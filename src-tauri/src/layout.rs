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

#[allow(clippy::too_many_arguments)]
pub fn build_layout(
    frames: &[FramePoint],
    width: u32,
    height: u32,
    text: &str,
    size_name: &str,
    opacity: f64,
    margin_ratio: f64,
    random_opacity: bool,
    random_size: bool,
    seed: u64,
) -> Result<Vec<Watermark>, String> {
    let mut rng = SplitMix64::new(seed);
    let short = width.min(height) as f64;
    let size_ratio = match size_name {
        "small" => 0.035,
        "large" => 0.055,
        _ => 0.045,
    };
    let base_size = (short * size_ratio).round().max(14.0) as u32;
    let margin_x = (width as f64 * margin_ratio).round() as u32;
    let margin_y = (height as f64 * margin_ratio).round() as u32;
    let safe_right = width.saturating_sub(margin_x);
    let safe_bottom = height.saturating_sub(margin_y);
    let min_distance = short * 0.15;
    let mut previous: Option<(f64, f64)> = None;

    frames
        .iter()
        .enumerate()
        .map(|(number, frame)| {
            let scale = if random_size {
                0.875 + rng.unit() * 0.25
            } else {
                1.0
            };
            let font_size = (base_size as f64 * scale).round().max(12.0) as u32;
            // Conservative full-em estimate keeps CJK and border inside the safe area.
            let text_width = (text.chars().count() as f64 * font_size as f64
                + font_size as f64 * 0.5)
                .ceil() as u32;
            let text_height = (font_size as f64 * 1.45).ceil() as u32;
            let max_x = safe_right
                .checked_sub(text_width)
                .ok_or("水印文字过宽，请减小字号或安全边距")?;
            let max_y = safe_bottom
                .checked_sub(text_height)
                .ok_or("水印文字过高，请减小字号或安全边距")?;
            if max_x < margin_x || max_y < margin_y {
                return Err("安全区域无法容纳水印，请减小字号、文字长度或安全边距".into());
            }

            let mut position = (margin_x, margin_y);
            for _ in 0..12 {
                position = (
                    rng.u32_between(margin_x, max_x),
                    rng.u32_between(margin_y, max_y),
                );
                let center = (
                    position.0 as f64 + text_width as f64 / 2.0,
                    position.1 as f64 + text_height as f64 / 2.0,
                );
                if previous.map_or(true, |last| {
                    (center.0 - last.0).hypot(center.1 - last.1) >= min_distance
                }) {
                    previous = Some(center);
                    break;
                }
            }

            let actual_opacity = if random_opacity {
                (opacity + (rng.unit() - 0.5) * 0.20).clamp(0.20, 1.0)
            } else {
                opacity
            };
            Ok(Watermark {
                number: number + 1,
                timestamp: frame.timestamp,
                pts: frame.pts,
                frame_index: frame.index,
                x: position.0,
                y: position.1,
                font_size,
                opacity: (actual_opacity * 1000.0).round() / 1000.0,
                text: text.into(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(duration: f64, fps: u32, interval: f64) -> Vec<Vec<FramePoint>> {
        let mut groups = vec![Vec::new(); (duration / interval).ceil() as usize];
        for i in 0..(duration * fps as f64).ceil() as u64 {
            let timestamp = i as f64 / fps as f64;
            let group = ((timestamp / interval).floor() as usize).min(groups.len() - 1);
            groups[group].push(FramePoint {
                timestamp,
                pts: Some(i as i64),
                index: i,
            });
        }
        groups
    }

    #[test]
    fn deterministic_interval_and_bounds() {
        let groups = frames(61.0, 30, 3.0);
        let mut first_rng = SplitMix64::new(42);
        let mut second_rng = SplitMix64::new(42);
        let picked = choose_frames(&groups, 20, &mut first_rng).unwrap();
        assert_eq!(picked, choose_frames(&groups, 20, &mut second_rng).unwrap());
        assert_eq!(picked.len(), 21);
        for (i, point) in picked.iter().enumerate() {
            assert!(point.timestamp >= i as f64 * 3.0);
            assert!(point.timestamp < ((i + 1) as f64 * 3.0).min(61.0));
        }

        let layout = build_layout(
            &picked,
            1920,
            1080,
            "UID:12345678",
            "medium",
            0.45,
            0.05,
            true,
            true,
            99,
        )
        .unwrap();
        let same = build_layout(
            &picked,
            1920,
            1080,
            "UID:12345678",
            "medium",
            0.45,
            0.05,
            true,
            true,
            99,
        )
        .unwrap();
        assert_eq!(layout, same);
        assert_ne!(
            layout,
            build_layout(
                &picked,
                1920,
                1080,
                "UID:12345678",
                "medium",
                0.45,
                0.05,
                true,
                true,
                100
            )
            .unwrap()
        );
        for mark in &layout {
            assert!(mark.x >= 96 && mark.y >= 54);
            let estimated_width =
                "UID:12345678".chars().count() as u32 * mark.font_size + mark.font_size / 2;
            let estimated_height = (mark.font_size as f64 * 1.45).ceil() as u32;
            assert!(mark.x + estimated_width <= 1824);
            assert!(mark.y + estimated_height <= 1026);
        }
    }

    #[test]
    fn one_thousand_positions_never_escape_safe_area() {
        let points = (0..1000)
            .map(|i| FramePoint {
                timestamp: i as f64,
                pts: Some(i),
                index: i as u64,
            })
            .collect::<Vec<_>>();
        let marks = build_layout(
            &points,
            1920,
            1080,
            "@PeakEdge",
            "medium",
            0.45,
            0.05,
            false,
            false,
            7,
        )
        .unwrap();
        assert_eq!(marks.len(), 1000);
        assert!(marks
            .windows(2)
            .all(|pair| pair[0].x != pair[1].x || pair[0].y != pair[1].y));
        assert!(marks
            .iter()
            .all(|mark| mark.x >= 96 && mark.y >= 54 && mark.x < 1824 && mark.y < 1026));
    }
}
