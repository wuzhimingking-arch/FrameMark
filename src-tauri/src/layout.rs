use crate::models::Watermark;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FramePoint {
    pub timestamp: f64,
    pub pts: Option<i64>,
    pub index: u64,
}

#[derive(Clone)]
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn usize(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            0
        } else {
            (self.next() % upper as u64) as usize
        }
    }

    fn u32_between(&mut self, min: u32, max: u32) -> u32 {
        min + (self.next() % (u64::from(max - min) + 1)) as u32
    }

    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1_u64 << 53) as f64
    }
}

static SEED_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn new_seed() -> u64 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mixed = time ^ u64::from(std::process::id()) ^ SEED_COUNTER.fetch_add(1, Ordering::Relaxed);
    SplitMix64::new(mixed).next()
}

pub fn choose_frames(
    intervals: &[Vec<FramePoint>],
    full_intervals: usize,
    rng: &mut SplitMix64,
) -> Result<Vec<FramePoint>, String> {
    intervals
        .iter()
        .enumerate()
        .map(|(i, frames)| {
            if frames.is_empty() {
                return Err(format!("第 {} 个时间区间没有有效视频帧", i + 1));
            }
            let (start, end) = if i < full_intervals && frames.len() >= 20 {
                let edge = (frames.len() as f64 * 0.05).ceil() as usize;
                (edge, frames.len() - edge)
            } else {
                (0, frames.len())
            };
            Ok(frames[start + rng.usize(end - start)])
        })
        .collect()
}

pub const ALGORITHM_VERSION: u32 = 2;
const CAPACITY_ERROR: &str = "当前水印数量或文字长度超出画面可显示范围，请减少水印或调整字号。";

pub fn validate_text(text: &str) -> Result<(), String> {
    if text.trim().is_empty() || text.chars().count() > 240 || text.lines().count() > 8 {
        return Err("水印内容不能为空，最多 240 个字符、8 行".into());
    }
    if text
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
    {
        return Err("水印包含不支持的控制字符".into());
    }
    Ok(())
}

fn wrap_text(text: &str, columns: usize) -> String {
    text.split('\n')
        .map(|line| {
            let mut out = String::new();
            for (i, ch) in line.chars().enumerate() {
                if i > 0 && i % columns == 0 {
                    out.push('\n');
                }
                out.push(ch);
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(clippy::too_many_arguments)]
pub fn build_plan(
    frames: &[FramePoint],
    width: u32,
    height: u32,
    texts: &[String],
    mode: &str,
    size_name: &str,
    opacity: f64,
    margin: f64,
    random_opacity: bool,
    random_size: bool,
    seed: u64,
    mut measure: impl FnMut(&str, u32) -> Result<(u32, u32), String>,
) -> Result<crate::models::WatermarkPlan, String> {
    if texts.is_empty() || !matches!(mode, "simultaneous" | "sequential") {
        return Err("请至少选择一条水印，并选择有效出现方式".into());
    }
    for text in texts {
        validate_text(text)?;
    }
    let mx = (width as f64 * margin).ceil() as u32;
    let my = (height as f64 * margin).ceil() as u32;
    let safe_w = width.saturating_sub(mx * 2);
    let safe_h = height.saturating_sub(my * 2);
    let ratio = match size_name {
        "small" => 0.035,
        "large" => 0.055,
        _ => 0.045,
    };
    let base = (width.min(height) as f64 * ratio).round().max(14.0) as u32;
    let mut rng = SplitMix64::new(seed);
    let mut marks: Vec<Watermark> = Vec::new();
    let mut previous = None::<(u32, u32)>;
    let mut cache = std::collections::BTreeMap::new();
    for (event, frame) in frames.iter().enumerate() {
        let active: Vec<&String> = if mode == "sequential" {
            vec![&texts[event % texts.len()]]
        } else {
            texts.iter().collect()
        };
        let mut placed = None;
        // A bounded retry keeps coordinates random; shrink at most 25%, then report capacity.
        for shrink in [1.0, 0.9, 0.75] {
            let mut batch: Vec<Watermark> = Vec::new();
            for text in &active {
                let scale = if random_size {
                    [0.875, 1.0, 1.125][rng.usize(3)]
                } else {
                    1.0
                };
                let font_size = (base as f64 * shrink * scale).round().max(12.0) as u32;
                let columns = (safe_w.saturating_sub(font_size) / font_size).max(1) as usize;
                let rendered = wrap_text(text, columns);
                let key = (rendered.clone(), font_size);
                let (w, h) = if let Some(value) = cache.get(&key) {
                    *value
                } else {
                    let value = measure(&rendered, font_size)?;
                    cache.insert(key, value);
                    value
                };
                if w > safe_w || h > safe_h {
                    break;
                }
                let mut position = None;
                for attempt in 0..512 {
                    let x = rng.u32_between(mx, width - mx - w);
                    let y = rng.u32_between(my, height - my - h);
                    let intersects = batch.iter().any(|other| {
                        x < other.x + other.width
                            && x + w > other.x
                            && y < other.y + other.height
                            && y + h > other.y
                    });
                    let near = previous.is_some_and(|(px, py)| {
                        (x as f64 - px as f64).hypot(y as f64 - py as f64)
                            < width.min(height) as f64 * 0.15
                    });
                    if !intersects && (!near || attempt >= 12) {
                        position = Some((x, y));
                        break;
                    }
                }
                let Some((x, y)) = position else {
                    break;
                };
                let alpha = if random_opacity {
                    (opacity + (rng.unit() - 0.5) * 0.2).clamp(0.2, 1.0)
                } else {
                    opacity
                };
                batch.push(Watermark {
                    number: marks.len() + batch.len() + 1,
                    event: event + 1,
                    timestamp: frame.timestamp,
                    pts: frame.pts,
                    frame_index: frame.index,
                    x,
                    y,
                    width: w,
                    height: h,
                    font_size,
                    opacity: (alpha * 1000.0).round() / 1000.0,
                    text: (*text).clone(),
                    rendered_text: rendered,
                });
            }
            if batch.len() == active.len() {
                placed = Some(batch);
                break;
            }
        }
        let batch = placed.ok_or(CAPACITY_ERROR)?;
        previous = batch.last().map(|m| (m.x, m.y));
        marks.extend(batch);
    }
    Ok(crate::models::WatermarkPlan {
        algorithm_version: ALGORITHM_VERSION,
        mode: mode.into(),
        texts: texts.to_vec(),
        event_count: frames.len(),
        marks,
        config: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frames() -> Vec<FramePoint> {
        (0..1000)
            .map(|i| FramePoint {
                timestamp: i as f64 * 3.0 + 1.0,
                pts: Some(i * 90 + 30),
                index: (i * 90 + 30) as u64,
            })
            .collect()
    }
    fn plan(seed: u64, mode: &str) -> crate::models::WatermarkPlan {
        build_plan(
            &frames(),
            1920,
            1080,
            &["甲".into(), "B".into(), "©".into()],
            mode,
            "medium",
            0.45,
            0.05,
            true,
            true,
            seed,
            |text, size| Ok((text.chars().count() as u32 * size + 8, size * 2)),
        )
        .unwrap()
    }
    #[test]
    fn deterministic_multi_plan_bounds_order_and_non_overlap() {
        let first = plan(42, "simultaneous");
        assert_eq!(first.marks, plan(42, "simultaneous").marks);
        assert_ne!(first.marks, plan(43, "simultaneous").marks);
        assert_eq!(first.event_count, 1000);
        assert_eq!(first.marks.len(), 3000);
        for batch in first.marks.chunks(3) {
            assert!(batch.iter().all(|m| m.frame_index == batch[0].frame_index
                && m.x >= 96
                && m.y >= 54
                && m.x + m.width <= 1824
                && m.y + m.height <= 1026));
            for (i, a) in batch.iter().enumerate() {
                for b in &batch[i + 1..] {
                    assert!(
                        !(a.x < b.x + b.width
                            && a.x + a.width > b.x
                            && a.y < b.y + b.height
                            && a.y + a.height > b.y)
                    );
                }
            }
        }
        let seq = plan(42, "sequential");
        assert_eq!(seq.marks.len(), 1000);
        for (i, m) in seq.marks.iter().enumerate() {
            assert_eq!(m.text, ["甲", "B", "©"][i % 3]);
        }
    }
    #[test]
    fn interval_tail_and_capacity_errors() {
        let groups = (0..21)
            .map(|i| {
                vec![FramePoint {
                    timestamp: i as f64 * 3.0 + 0.5,
                    pts: None,
                    index: i,
                }]
            })
            .collect::<Vec<_>>();
        assert_eq!(
            choose_frames(&groups, 20, &mut SplitMix64::new(1))
                .unwrap()
                .len(),
            21
        );
        assert!(build_plan(
            &frames()[..1],
            100,
            100,
            &["long".into()],
            "simultaneous",
            "large",
            0.45,
            0.2,
            false,
            false,
            1,
            |_, _| Ok((1000, 1000))
        )
        .is_err());
        assert!(validate_text("").is_err());
        assert!(validate_text(&"a".repeat(241)).is_err());
        assert!(validate_text("© URL https://example.test\n繁體中文 @user").is_ok());
    }
}
