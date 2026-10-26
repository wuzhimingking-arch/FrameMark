use crate::layout::{build_plan, choose_frames, FramePoint, SplitMix64};
use crate::models::{EncodingOption, MediaInfo, ProcessRequest, Watermark, WatermarkPlan};
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Copy)]
pub struct VideoEncoder {
    pub id: &'static str,
    pub label: &'static str,
}

const CPU_ENCODER: VideoEncoder = VideoEncoder {
    id: "cpu",
    label: "CPU（libx264）",
};
const GPU_ENCODERS: [VideoEncoder; 4] = [
    VideoEncoder {
        id: "nvenc",
        label: "NVIDIA NVENC",
    },
    VideoEncoder {
        id: "qsv",
        label: "Intel Quick Sync",
    },
    VideoEncoder {
        id: "amf",
        label: "AMD AMF",
    },
    VideoEncoder {
        id: "videotoolbox",
        label: "Apple VideoToolbox",
    },
];

#[derive(Clone)]
pub struct Tools {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
    pub font: PathBuf,
}

pub struct PreparedJob {
    pub media: MediaInfo,
    pub watermarks: Vec<Watermark>,
    pub output: PathBuf,
    pub partial_output: PathBuf,
    pub temp_dir: PathBuf,
    pub filter_script: PathBuf,
    pub seed_text: String,
    pub copy_audio: bool,
    pub plan: WatermarkPlan,
}

pub fn detection_options(tools: &Tools) -> Vec<EncodingOption> {
    let listed = new_command(&tools.ffmpeg)
        .args(["-hide_banner", "-encoders"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_default();
    let mut options = vec![EncodingOption {
        id: "auto".into(),
        label: "自动（优先 GPU）".into(),
        available: true,
    }];
    for encoder in GPU_ENCODERS {
        let available = listed.contains(h264_name(encoder.id)) && encoder_works(tools, encoder);
        options.push(EncodingOption {
            id: encoder.id.into(),
            label: encoder.label.into(),
            available,
        });
    }
    options.push(EncodingOption {
        id: CPU_ENCODER.id.into(),
        label: "CPU（软件编码）".into(),
        available: true,
    });
    options
}

pub fn encoding_attempts(requested: &str, options: &[EncodingOption]) -> Vec<VideoEncoder> {
    let available = |id: &str| {
        options
            .iter()
            .any(|option| option.id == id && option.available)
    };
    let selected = if requested == "auto" {
        GPU_ENCODERS
            .into_iter()
            .find(|encoder| available(encoder.id))
    } else {
        GPU_ENCODERS
            .into_iter()
            .find(|encoder| encoder.id == requested && available(encoder.id))
    };
    selected
        .into_iter()
        .chain(std::iter::once(CPU_ENCODER))
        .collect()
}

fn h264_name(id: &str) -> &'static str {
    match id {
        "nvenc" => "h264_nvenc",
        "qsv" => "h264_qsv",
        "amf" => "h264_amf",
        "videotoolbox" => "h264_videotoolbox",
        _ => "libx264",
    }
}

fn encoder_works(tools: &Tools, encoder: VideoEncoder) -> bool {
    new_command(&tools.ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=320x240:r=1",
            "-frames:v",
            "1",
            "-c:v",
            h264_name(encoder.id),
            "-f",
            "null",
            "-",
        ])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn new_command(program: &Path) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

pub fn inspect(tools: &Tools, input: &Path) -> Result<MediaInfo, String> {
    if !input.is_file() {
        return Err("视频文件不存在或已被移动".into());
    }
    let extension = input
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "mp4" | "mov" | "mkv" | "webm") {
        return Err("当前仅支持 MP4、MOV、MKV 和 WebM 文件".into());
    }
    let output = new_command(&tools.ffprobe)
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(input)
        .output()
        .map_err(|e| format!("ffprobe 启动失败：{e}"))?;
    if !output.status.success() {
        return Err(format!(
            "无法读取视频：{}",
            concise(&String::from_utf8_lossy(&output.stderr))
        ));
    }
    let json: Value =
        serde_json::from_slice(&output.stdout).map_err(|_| "ffprobe 返回了无法解析的信息")?;
    let streams = json["streams"]
        .as_array()
        .ok_or("视频中没有可读取的媒体流")?;
    let video = streams
        .iter()
        .find(|s| s["codec_type"] == "video")
        .ok_or("文件中没有视频流")?;
    let codec = text(video, "codec_name");
    if !matches!(codec.as_str(), "h264" | "hevc") {
        return Err(format!(
            "当前视频编码 {codec} 暂不支持，请转换为 H.264/H.265 后重试"
        ));
    }
    let audio = streams.iter().find(|s| s["codec_type"] == "audio");
    let rotation = video["side_data_list"]
        .as_array()
        .and_then(|items| items.iter().find_map(|item| item["rotation"].as_i64()))
        .or_else(|| {
            video["tags"]["rotate"]
                .as_str()
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(0) as i32;
    let raw_width = video["width"].as_u64().unwrap_or(0) as u32;
    let raw_height = video["height"].as_u64().unwrap_or(0) as u32;
    let (width, height) = if rotation.abs() % 180 == 90 {
        (raw_height, raw_width)
    } else {
        (raw_width, raw_height)
    };
    let fps_text = text(video, "avg_frame_rate");
    let fps = rational(&fps_text)
        .or_else(|| rational(&text(video, "r_frame_rate")))
        .unwrap_or(0.0);
    let duration = json["format"]["duration"]
        .as_str()
        .and_then(|v| v.parse().ok())
        .or_else(|| video["duration"].as_str().and_then(|v| v.parse().ok()))
        .unwrap_or(0.0);
    if duration <= 0.0 || width == 0 || height == 0 {
        return Err("视频时长或分辨率无效，文件可能已损坏".into());
    }
    let transfer = text(video, "color_transfer");
    let primaries = text(video, "color_primaries");
    let profile = text(video, "profile");
    let hdr = matches!(transfer.as_str(), "smpte2084" | "arib-std-b67")
        || primaries == "bt2020"
        || profile.to_ascii_lowercase().contains("dolby");
    let hdr_description = if hdr {
        format!("{} {} {}", primaries, transfer, profile)
            .trim()
            .to_string()
    } else {
        String::new()
    };
    let metadata = fs::metadata(input).map_err(|e| format!("无法读取文件信息：{e}"))?;
    Ok(MediaInfo {
        path: input.to_string_lossy().into_owned(),
        file_name: input
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .into(),
        size: metadata.len(),
        duration,
        width,
        height,
        fps,
        fps_text,
        video_codec: codec,
        audio_codec: audio.map(|v| text(v, "codec_name")).unwrap_or_default(),
        has_audio: audio.is_some(),
        rotation,
        hdr,
        hdr_description,
    })
}

pub fn prepare(
    tools: &Tools,
    request: &ProcessRequest,
    texts: &[String],
    seed: u64,
) -> Result<PreparedJob, String> {
    validate_request(request)?;
    let input = Path::new(&request.input_path);
    let media = inspect(tools, input)?;
    let output_dir = if request.output_dir.trim().is_empty() {
        input.parent().ok_or("无法确定原视频目录")?.to_path_buf()
    } else {
        PathBuf::from(&request.output_dir)
    };
    if !output_dir.is_dir() {
        return Err("输出目录不存在或已被移动".into());
    }
    let available =
        fs2::available_space(&output_dir).map_err(|e| format!("无法检查磁盘空间：{e}"))?;
    let required = media.size.saturating_mul(6) / 5 + 256 * 1024 * 1024;
    if available < required {
        return Err(format!(
            "输出盘空间不足：建议至少保留 {:.1} GB",
            required as f64 / 1_073_741_824.0
        ));
    }

    let groups = scan_frames(tools, input, media.duration, request.interval)?;
    let full = (media.duration / request.interval).floor() as usize;
    let mut rng = SplitMix64::new(seed);
    let chosen = choose_frames(&groups, full, &mut rng)?;
    let output = unique_output(&output_dir, input)?;
    let partial_output = output.with_file_name(format!(
        ".{}.framemark-part.mp4",
        output
            .file_stem()
            .and_then(|v| v.to_str())
            .unwrap_or("output")
    ));
    let seed_text = format!("{seed:016X}");
    let temp_dir = (0..100)
        .map(|suffix| {
            std::env::temp_dir().join(if suffix == 0 {
                format!("framemark-{seed_text}")
            } else {
                format!("framemark-{seed_text}-{suffix}")
            })
        })
        .find(|path| fs::create_dir(path).is_ok())
        .ok_or("无法创建独立临时目录")?;
    let planned = make_plan(
        tools,
        request,
        &chosen,
        media.width,
        media.height,
        texts,
        seed,
        &temp_dir,
    );
    let plan = match planned {
        Ok(plan) => plan,
        Err(error) => {
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(error);
        }
    };
    let watermarks = plan.marks.clone();
    let filter_script = temp_dir.join("filter.txt");
    if let Err(error) = write_filter(tools, &temp_dir, &filter_script, &watermarks, request) {
        let _ = fs::remove_dir_all(&temp_dir);
        return Err(error);
    }
    let copy_audio = matches!(
        media.audio_codec.as_str(),
        "aac" | "mp3" | "ac3" | "eac3" | "alac"
    );
    Ok(PreparedJob {
        media,
        watermarks,
        output,
        partial_output,
        temp_dir,
        filter_script,
        seed_text,
        copy_audio,
        plan,
    })
}

#[cfg(test)]
pub fn ffmpeg_command(tools: &Tools, input: &Path, job: &PreparedJob) -> Command {
    ffmpeg_command_with_encoder(tools, input, job, CPU_ENCODER)
}

pub fn ffmpeg_command_with_encoder(
    tools: &Tools,
    input: &Path,
    job: &PreparedJob,
    encoder: VideoEncoder,
) -> Command {
    let mut command = new_command(&tools.ffmpeg);
    command
        .args(["-y", "-hide_banner", "-nostdin", "-i"])
        .arg(input)
        .args(["-/filter_complex"])
        .arg(&job.filter_script)
        .args([
            "-map",
            "[marked]",
            "-map",
            "0:a?",
            "-map",
            "0:s?",
            "-c:s",
            "mov_text",
            "-map_metadata",
            "0",
            "-map_chapters",
            "0",
            "-metadata:s:v:0",
            "rotate=0",
            "-pix_fmt",
            "yuv420p",
            "-fps_mode",
            "passthrough",
        ]);
    configure_encoder(&mut command, encoder);
    if job.copy_audio {
        command.args(["-c:a", "copy"]);
    } else if job.media.has_audio {
        command.args(["-c:a", "aac", "-b:a", "256k"]);
    }
    command
        .args(["-movflags", "+faststart", "-progress", "pipe:1", "-nostats"])
        .arg(&job.partial_output)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

fn configure_encoder(command: &mut Command, encoder: VideoEncoder) {
    match encoder.id {
        "nvenc" => {
            command.args([
                "-c:v",
                "h264_nvenc",
                "-preset",
                "p5",
                "-tune",
                "hq",
                "-rc",
                "vbr",
                "-cq",
                "19",
                "-b:v",
                "0",
            ]);
        }
        "qsv" => {
            command.args([
                "-c:v",
                "h264_qsv",
                "-preset",
                "medium",
                "-global_quality",
                "20",
            ]);
        }
        "amf" => {
            command.args([
                "-c:v", "h264_amf", "-quality", "quality", "-rc", "cqp", "-qp_i", "20", "-qp_p",
                "22",
            ]);
        }
        "videotoolbox" => {
            command.args(["-c:v", "h264_videotoolbox", "-q:v", "60"]);
        }
        _ => {
            command.args(["-c:v", "libx264", "-preset", "slow", "-crf", "17"]);
        }
    }
}

pub fn validate_output(tools: &Tools, input: &MediaInfo, output: &Path) -> Result<(), String> {
    if !output.is_file() || fs::metadata(output).map_err(|e| e.to_string())?.len() < 1024 {
        return Err("输出文件不存在或不完整".into());
    }
    let result = inspect(tools, output)?;
    if result.width != input.width || result.height != input.height {
        return Err("输出校验失败：分辨率发生异常变化".into());
    }
    if (result.duration - input.duration).abs() > 0.25_f64.max(2.0 / input.fps.max(1.0)) {
        return Err("输出校验失败：视频时长不一致".into());
    }
    if input.has_audio && !result.has_audio {
        return Err("输出校验失败：音频流丢失".into());
    }
    Ok(())
}

pub fn cleanup(job: &PreparedJob) {
    let _ = fs::remove_file(&job.partial_output);
    let _ = fs::remove_dir_all(&job.temp_dir);
}

fn scan_frames(
    tools: &Tools,
    input: &Path,
    duration: f64,
    interval: f64,
) -> Result<Vec<Vec<FramePoint>>, String> {
    let count = (duration / interval).ceil().max(1.0) as usize;
    let mut groups = vec![Vec::new(); count];
    let mut child = new_command(&tools.ffprobe)
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_frames",
            "-show_entries",
            "frame=pts,best_effort_timestamp_time",
            "-of",
            "csv=p=0",
        ])
        .arg(input)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("ffprobe 帧扫描启动失败：{e}"))?;
    let stdout = child.stdout.take().ok_or("无法读取 ffprobe 帧信息")?;
    let mut first_time = None;
    let mut index = 0_u64;
    for line in BufReader::new(stdout).lines() {
        let line = line.map_err(|e| format!("读取视频帧失败：{e}"))?;
        let fields = line.split(',').collect::<Vec<_>>();
        if fields.len() < 2 {
            continue;
        }
        let pts = fields[0].parse::<i64>().ok();
        let raw_time = fields[1]
            .parse::<f64>()
            .ok()
            .or_else(|| fields[0].parse::<f64>().ok());
        if let Some(raw_time) = raw_time {
            let origin = *first_time.get_or_insert(raw_time);
            let timestamp = (raw_time - origin).max(0.0);
            let group = ((timestamp / interval).floor() as usize).min(count - 1);
            groups[group].push(FramePoint {
                timestamp,
                pts,
                index,
            });
            index += 1;
        }
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("等待 ffprobe 失败：{e}"))?;
    if !output.status.success() {
        return Err(format!(
            "读取有效视频帧失败：{}",
            concise(&String::from_utf8_lossy(&output.stderr))
        ));
    }
    if index == 0 {
        return Err("视频中没有可处理的有效帧".into());
    }
    Ok(groups)
}

fn padding(size: u32) -> u32 {
    (size / 5).max(4) + 3
}

fn make_plan(
    tools: &Tools,
    request: &ProcessRequest,
    frames: &[FramePoint],
    width: u32,
    height: u32,
    texts: &[String],
    seed: u64,
    temp: &Path,
) -> Result<WatermarkPlan, String> {
    let mut plan = build_plan(
        frames,
        width,
        height,
        texts,
        &request.mode,
        &request.font_size,
        request.opacity,
        request.safe_margin,
        request.random_opacity,
        request.random_size,
        seed ^ 0xA0761D6478BD642F,
        |text, size| measure_text(tools, temp, text, size, width, height),
    )?;
    plan.config = Some(request.clone());
    Ok(plan)
}

fn measure_text(
    tools: &Tools,
    temp: &Path,
    text: &str,
    size: u32,
    width: u32,
    height: u32,
) -> Result<(u32, u32), String> {
    let text_file = temp.join("measure.txt");
    fs::write(&text_file, text).map_err(|e| e.to_string())?;
    let pad = padding(size);
    let filter = format!("drawtext=fontfile='{}':textfile='{}':expansion=none:fontsize={}:line_spacing={}:fontcolor=white:borderw={}:bordercolor=white:x={}:y={}",
        filter_path(&tools.font),filter_path(&text_file),size,size/4,(size/24).max(1),pad,pad);
    let output = new_command(&tools.ffmpeg)
        .args(["-v", "error", "-f", "lavfi", "-i"])
        .arg(format!("color=c=black:s={width}x{height}:r=1"))
        .args([
            "-vf",
            &filter,
            "-frames:v",
            "1",
            "-pix_fmt",
            "gray",
            "-f",
            "rawvideo",
            "pipe:1",
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "测量水印失败：{}",
            concise(&String::from_utf8_lossy(&output.stderr))
        ));
    }
    let mut right = 0;
    let mut bottom = 0;
    for (i, pixel) in output.stdout.iter().enumerate() {
        if *pixel > 32 {
            right = right.max(i as u32 % width + 1);
            bottom = bottom.max(i as u32 / width + 1);
        }
    }
    if right == 0 {
        return Err("水印没有可显示字符".into());
    }
    if right + pad >= width || bottom + pad >= height {
        return Ok((width + 1, height + 1));
    }
    Ok((right + pad, bottom + pad))
}

fn write_filter(
    tools: &Tools,
    temp: &Path,
    script: &Path,
    marks: &[Watermark],
    request: &ProcessRequest,
) -> Result<(), String> {
    let mut groups: std::collections::BTreeMap<(String, u32, usize), Vec<&Watermark>> =
        std::collections::BTreeMap::new();
    for m in marks {
        groups
            .entry((
                m.rendered_text.clone(),
                m.font_size,
                (m.number - 1) % request.watermark_ids.len(),
            ))
            .or_default()
            .push(m);
    }
    let (color, border) = if request.color == "black" {
        ("black", "white@0.70")
    } else {
        ("white", "black@0.70")
    };
    let mut filters = Vec::new();
    for (index, ((text, size, _slot), batch)) in groups.into_iter().enumerate() {
        let text_file = temp.join(format!("text-{index}.txt"));
        fs::write(&text_file, text).map_err(|e| e.to_string())?;
        let expr = |value: fn(&Watermark) -> f64| {
            batch
                .iter()
                .map(|m| format!("{:.3}*eq(n\\,{})", value(m), m.frame_index))
                .collect::<Vec<_>>()
                .join("+")
        };
        let enable = batch
            .iter()
            .map(|m| format!("eq(n\\,{})", m.frame_index))
            .collect::<Vec<_>>()
            .join("+");
        let x = expr(|m| (m.x + padding(m.font_size)) as f64);
        let y = expr(|m| (m.y + padding(m.font_size)) as f64);
        let alpha = expr(|m| m.opacity);
        filters.push(format!("drawtext=fontfile='{}':textfile='{}':expansion=none:fontsize={}:line_spacing={}:fontcolor={}:borderw={}:bordercolor={}:box={}:boxcolor=black@0.28:boxborderw={}:x='{}':y='{}':alpha='{}':enable='{}'",
            filter_path(&tools.font),filter_path(&text_file),size,size/4,color,(size/24).max(1),border,if request.background {1}else{0},size/5,x,y,alpha,enable));
    }
    fs::write(script, format!("[0:v:0]{}[marked]", filters.join(","))).map_err(|e| e.to_string())
}

pub fn preview(
    tools: &Tools,
    request: &ProcessRequest,
    texts: &[String],
    next: usize,
    seed: u64,
) -> Result<Vec<u8>, String> {
    validate_request(request)?;
    let info = inspect(tools, Path::new(&request.input_path))?;
    let temp = std::env::temp_dir().join(format!("framemark-preview-{seed}"));
    fs::create_dir(&temp).map_err(|e| e.to_string())?;
    let result = (|| {
        let active = if request.mode == "sequential" {
            vec![texts.get(next % texts.len()).ok_or("请选择水印")?.clone()]
        } else {
            texts.to_vec()
        };
        let frame = FramePoint {
            timestamp: 0.0,
            pts: None,
            index: 0,
        };
        let plan = make_plan(
            tools,
            request,
            &[frame],
            info.width,
            info.height,
            &active,
            seed,
            &temp,
        )?;
        let script = temp.join("preview-filter.txt");
        write_filter(tools, &temp, &script, &plan.marks, request)?;
        let out = new_command(&tools.ffmpeg)
            .args(["-v", "error", "-ss", "0", "-i"])
            .arg(&request.input_path)
            .arg("-/filter_complex")
            .arg(&script)
            .args([
                "-map",
                "[marked]",
                "-frames:v",
                "1",
                "-an",
                "-c:v",
                "png",
                "-f",
                "image2pipe",
                "pipe:1",
            ])
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(concise(&String::from_utf8_lossy(&out.stderr)));
        }
        Ok(out.stdout)
    })();
    let _ = fs::remove_dir_all(temp);
    result
}

fn unique_output(dir: &Path, input: &Path) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .and_then(|v| v.to_str())
        .ok_or("输入文件名无效")?;
    for suffix in 1..10_000 {
        let name = if suffix == 1 {
            format!("{stem}_marked.mp4")
        } else {
            format!("{stem}_marked_{suffix}.mp4")
        };
        let candidate = dir.join(name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("无法生成不重复的输出文件名".into())
}

fn validate_request(request: &ProcessRequest) -> Result<(), String> {
    if request.watermark_ids.is_empty()
        || !matches!(request.mode.as_str(), "simultaneous" | "sequential")
    {
        return Err("请至少选择一条水印，并选择有效出现方式".into());
    }
    if request
        .watermark_ids
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        != request.watermark_ids.len()
    {
        return Err("不能重复选择同一条水印".into());
    }
    if !(0.5..=60.0).contains(&request.interval) {
        return Err("水印间隔必须在 0.5 到 60 秒之间".into());
    }
    if !(0.2..=1.0).contains(&request.opacity) {
        return Err("透明度必须在 20% 到 100% 之间".into());
    }
    if !(0.02..=0.20).contains(&request.safe_margin) {
        return Err("安全边距必须在 2% 到 20% 之间".into());
    }
    if !matches!(request.font_size.as_str(), "small" | "medium" | "large") {
        return Err("字号设置无效".into());
    }
    if !matches!(request.color.as_str(), "white" | "black") {
        return Err("文字颜色设置无效".into());
    }
    if !matches!(
        request.encoder.as_str(),
        "auto" | "nvenc" | "qsv" | "amf" | "videotoolbox" | "cpu"
    ) {
        return Err("视频编码加速设置无效".into());
    }
    Ok(())
}

fn text(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap_or_default().to_string()
}
fn rational(value: &str) -> Option<f64> {
    let (a, b) = value.split_once('/')?;
    let b = b.parse::<f64>().ok()?;
    (b != 0.0)
        .then(|| a.parse::<f64>().ok().map(|a| a / b))
        .flatten()
}
fn filter_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .replace(':', "\\:")
        .replace('\'', "\\'")
}
fn concise(value: &str) -> String {
    value
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("未知 FFmpeg 错误")
        .trim()
        .chars()
        .take(300)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn output_names_never_overwrite() {
        let temp =
            std::env::temp_dir().join(format!("framemark-output-test-{}", std::process::id()));
        fs::create_dir_all(&temp).unwrap();
        let input = temp.join("北京 旅行.mp4");
        fs::write(&input, b"x").unwrap();
        assert_eq!(
            unique_output(&temp, &input).unwrap().file_name().unwrap(),
            "北京 旅行_marked.mp4"
        );
        fs::write(temp.join("北京 旅行_marked.mp4"), b"x").unwrap();
        assert_eq!(
            unique_output(&temp, &input).unwrap().file_name().unwrap(),
            "北京 旅行_marked_2.mp4"
        );
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn real_ffmpeg_is_strictly_single_frame_and_keeps_audio() {
        let tools = test_tools();
        let temp = std::env::temp_dir().join(format!(
            "帧印 真实测试-{}-{}",
            std::process::id(),
            crate::layout::new_seed()
        ));
        fs::create_dir_all(&temp).unwrap();
        let input = temp.join("北京旅行 测试01.mp4");
        let generated = new_command(&tools.ffmpeg)
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=black:s=320x240:r=10:d=5",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000:duration=5",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-pix_fmt",
                "yuv420p",
                "-c:a",
                "aac",
                "-shortest",
            ])
            .arg(&input)
            .output()
            .unwrap();
        assert!(
            generated.status.success(),
            "status={:?} stdout={} stderr={}",
            generated.status,
            String::from_utf8_lossy(&generated.stdout),
            String::from_utf8_lossy(&generated.stderr)
        );

        let request = ProcessRequest {
            input_path: input.to_string_lossy().into_owned(),
            output_dir: temp.to_string_lossy().into_owned(),
            watermark_ids: vec![1],
            mode: "simultaneous".into(),
            interval: 3.0,
            opacity: 1.0,
            font_size: "medium".into(),
            color: "white".into(),
            background: false,
            random_opacity: false,
            random_size: false,
            safe_margin: 0.05,
            encoder: "cpu".into(),
        };
        let job = prepare(&tools, &request, &["UID:12345678".into()], 0xA42F72E19B30).unwrap();
        assert_eq!(job.watermarks.len(), 2);
        let output = ffmpeg_command(&tools, &input, &job).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        validate_output(&tools, &job.media, &job.partial_output).unwrap();

        let analyzed = new_command(&tools.ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(&job.partial_output)
            .args([
                "-vf",
                "signalstats,metadata=print:file=-",
                "-an",
                "-f",
                "null",
                "-",
            ])
            .output()
            .unwrap();
        assert!(analyzed.status.success());
        let detected = bright_frames(&String::from_utf8_lossy(&analyzed.stdout));
        let expected = job
            .watermarks
            .iter()
            .map(|mark| mark.frame_index)
            .collect::<BTreeSet<_>>();
        assert_eq!(detected, expected, "只有目标帧的 YMAX 应超过 100");
        for index in expected {
            assert!(!detected.contains(&index.saturating_sub(1)) || index == 0);
            assert!(!detected.contains(&(index + 1)));
        }
        cleanup(&job);
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn hardware_encoder_keeps_single_frames_when_available() {
        let tools = test_tools();
        let option = detection_options(&tools).into_iter().find(|option| {
            option.available
                && matches!(option.id.as_str(), "nvenc" | "qsv" | "amf" | "videotoolbox")
        });
        let Some(option) = option else { return };
        let encoder = encoding_attempts(&option.id, &[option.clone()])[0];
        let temp =
            std::env::temp_dir().join(format!("framemark-gpu-test-{}", crate::layout::new_seed()));
        fs::create_dir_all(&temp).unwrap();
        let input = temp.join("gpu-input.mp4");
        assert!(new_command(&tools.ffmpeg)
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=black:s=320x240:r=10:d=5",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000:duration=5",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-c:a",
                "aac",
                "-shortest"
            ])
            .arg(&input)
            .output()
            .unwrap()
            .status
            .success());
        let request = ProcessRequest {
            input_path: input.to_string_lossy().into_owned(),
            output_dir: temp.to_string_lossy().into_owned(),
            watermark_ids: vec![1],
            mode: "simultaneous".into(),
            interval: 3.0,
            opacity: 1.0,
            font_size: "medium".into(),
            color: "white".into(),
            background: false,
            random_opacity: false,
            random_size: false,
            safe_margin: 0.05,
            encoder: option.id,
        };
        let job = prepare(&tools, &request, &["GPU-UID".into()], 0xA42F72E19B30).unwrap();
        let output = ffmpeg_command_with_encoder(&tools, &input, &job, encoder)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        validate_output(&tools, &job.media, &job.partial_output).unwrap();
        let input_times = frame_times(&tools, &input);
        let output_times = frame_times(&tools, &job.partial_output);
        assert_eq!(input_times.len(), output_times.len());
        assert!(input_times
            .iter()
            .zip(output_times)
            .all(|(a, b)| (a - b).abs() < 0.002));
        let analyzed = new_command(&tools.ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(&job.partial_output)
            .args([
                "-vf",
                "signalstats,metadata=print:file=-",
                "-an",
                "-f",
                "null",
                "-",
            ])
            .output()
            .unwrap();
        assert!(analyzed.status.success());
        let detected = bright_frames(&String::from_utf8_lossy(&analyzed.stdout));
        let expected = job
            .watermarks
            .iter()
            .map(|mark| mark.frame_index)
            .collect::<BTreeSet<_>>();
        assert_eq!(detected, expected, "{} 路径必须严格单帧", encoder.label);
        assert!(job.media.has_audio && inspect(&tools, &job.partial_output).unwrap().has_audio);
        cleanup(&job);
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn unavailable_hardware_selection_falls_back_to_cpu() {
        let options = vec![
            EncodingOption {
                id: "auto".into(),
                label: "自动".into(),
                available: true,
            },
            EncodingOption {
                id: "nvenc".into(),
                label: "NVIDIA NVENC".into(),
                available: false,
            },
            EncodingOption {
                id: "cpu".into(),
                label: "CPU".into(),
                available: true,
            },
        ];
        assert_eq!(encoding_attempts("nvenc", &options)[0].id, "cpu");
    }

    #[test]
    fn multi_watermarks_cpu_gpu_exact_events_and_history() {
        let tools = test_tools();
        let dir =
            std::env::temp_dir().join(format!("帧印 多水印测试-{}", crate::layout::new_seed()));
        fs::create_dir(&dir).unwrap();
        let input = dir.join("中文 空格.mp4");
        assert!(new_command(&tools.ffmpeg)
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=black:s=640x360:r=20:d=5",
                "-f",
                "lavfi",
                "-i",
                "sine=duration=5",
                "-vf",
                "select=\'if(lt(t,2.5),not(mod(n,2)),1)\'",
                "-fps_mode",
                "vfr",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-c:a",
                "aac",
                "-shortest"
            ])
            .arg(&input)
            .output()
            .unwrap()
            .status
            .success());
        let texts = vec![
            "A @创作者".into(),
            "B © 2026\nexample.test".into(),
            "C 100% ID".into(),
        ];
        let mut encoders = vec![CPU_ENCODER];
        if let Some(gpu) = encoding_attempts("auto", &detection_options(&tools))
            .into_iter()
            .find(|e| e.id != "cpu")
        {
            encoders.push(gpu);
        }
        let database = dir.join("history.sqlite3");
        crate::db::initialize(&database).unwrap();
        for mode in ["simultaneous", "sequential"] {
            for encoder in &encoders {
                let request = ProcessRequest {
                    input_path: input.to_string_lossy().into(),
                    output_dir: dir.to_string_lossy().into(),
                    watermark_ids: vec![1, 2, 3],
                    mode: mode.into(),
                    interval: 0.5,
                    opacity: 1.0,
                    font_size: "medium".into(),
                    color: "white".into(),
                    background: true,
                    random_opacity: false,
                    random_size: true,
                    safe_margin: 0.05,
                    encoder: encoder.id.into(),
                };
                let job = prepare(&tools, &request, &texts, 4242).unwrap();
                assert_eq!(job.plan.event_count, 10);
                assert_eq!(
                    job.watermarks.len(),
                    if mode == "simultaneous" { 30 } else { 10 }
                );
                let result = ffmpeg_command_with_encoder(&tools, &input, &job, *encoder)
                    .output()
                    .unwrap();
                assert!(
                    result.status.success(),
                    "{}",
                    String::from_utf8_lossy(&result.stderr)
                );
                validate_output(&tools, &job.media, &job.partial_output).unwrap();
                let audio_hash = |path: &Path| {
                    let out = new_command(&tools.ffmpeg)
                        .args(["-v", "error", "-i"])
                        .arg(path)
                        .args([
                            "-map", "0:a:0", "-c:a", "copy", "-f", "hash", "-hash", "md5", "pipe:1",
                        ])
                        .output()
                        .unwrap();
                    assert!(out.status.success());
                    out.stdout
                };
                assert_eq!(
                    audio_hash(&input),
                    audio_hash(&job.partial_output),
                    "音频压缩流必须保持完全一致"
                );
                let image = preview(&tools, &request, &texts, 1, 555).unwrap();
                assert_eq!(&image[..8], b"\x89PNG\r\n\x1a\n");
                let decoded = new_command(&tools.ffmpeg)
                    .args(["-v", "error", "-i"])
                    .arg(&job.partial_output)
                    .args([
                        "-an",
                        "-fps_mode",
                        "passthrough",
                        "-pix_fmt",
                        "gray",
                        "-f",
                        "rawvideo",
                        "pipe:1",
                    ])
                    .output()
                    .unwrap();
                assert!(decoded.status.success());
                let frame_size = 640 * 360;
                assert_eq!(decoded.stdout.len(), 75 * frame_size);
                let mut detected = std::collections::BTreeSet::new();
                for (index, pixels) in decoded.stdout.chunks(frame_size).enumerate() {
                    if pixels.iter().any(|v| *v > 100) {
                        detected.insert(index as u64);
                    }
                }
                assert_eq!(
                    detected,
                    job.watermarks.iter().map(|m| m.frame_index).collect()
                );
                for (i, m) in job.watermarks.iter().enumerate() {
                    if mode == "sequential" {
                        assert_eq!(m.text, texts[i % 3]);
                    }
                    let pixels = &decoded.stdout[m.frame_index as usize * frame_size
                        ..(m.frame_index as usize + 1) * frame_size];
                    let mut visible = false;
                    for y in m.y..m.y + m.height {
                        if pixels[(y * 640 + m.x) as usize..(y * 640 + m.x + m.width) as usize]
                            .iter()
                            .any(|v| *v > 100)
                        {
                            visible = true;
                            break;
                        }
                    }
                    assert!(visible, "每条水印在所记录区域都必须可见");
                }
                let times = frame_times(&tools, &job.partial_output);
                let original_times = frame_times(&tools, &input);
                assert_eq!(times.len(), original_times.len());
                assert!(times
                    .iter()
                    .zip(original_times)
                    .all(|(a, b)| (a - b).abs() < 0.002));
                for m in &job.watermarks {
                    assert!((times[m.frame_index as usize] - m.timestamp).abs() < 0.002);
                }
                let id = crate::db::insert_history(
                    &database,
                    &job.media,
                    &job.output,
                    &job.plan,
                    request.interval,
                    &job.seed_text,
                    &job.watermarks,
                )
                .unwrap();
                let saved = crate::db::history(&database, id).unwrap();
                assert_eq!(saved.plan.texts, texts);
                assert_eq!(saved.watermarks, job.watermarks);
                assert_eq!(saved.plan.mode, mode);
                eprintln!(
                    "validated mode={mode} encoder={} events=10 draws={} frames=75 VFR=true audio=aac",
                    encoder.label,
                    job.watermarks.len()
                );
                cleanup(&job);
            }
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn cancellation_removes_partial_output() {
        let tools = test_tools();
        let temp =
            std::env::temp_dir().join(format!("framemark-cancel-{}", crate::layout::new_seed()));
        fs::create_dir_all(&temp).unwrap();
        let input = temp.join("cancel.mp4");
        assert!(new_command(&tools.ffmpeg)
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=black:s=640x360:r=30:d=20",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast"
            ])
            .arg(&input)
            .output()
            .unwrap()
            .status
            .success());
        let request = ProcessRequest {
            input_path: input.to_string_lossy().into_owned(),
            output_dir: temp.to_string_lossy().into_owned(),
            watermark_ids: vec![1],
            mode: "simultaneous".into(),
            interval: 3.0,
            opacity: 0.45,
            font_size: "medium".into(),
            color: "white".into(),
            background: false,
            random_opacity: false,
            random_size: false,
            safe_margin: 0.05,
            encoder: "cpu".into(),
        };
        let job = prepare(&tools, &request, &["@CancelTest".into()], 123).unwrap();
        let mut child = ffmpeg_command(&tools, &input, &job).spawn().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = child.kill();
        let _ = child.wait();
        cleanup(&job);
        assert!(!job.partial_output.exists());
        assert!(!job.temp_dir.exists());
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn duplicate_texts_keep_independent_same_frame_positions() {
        let tools = test_tools();
        let temp =
            std::env::temp_dir().join(format!("framemark-duplicate-{}", crate::layout::new_seed()));
        fs::create_dir(&temp).unwrap();
        let request: ProcessRequest = serde_json::from_value(serde_json::json!({
            "inputPath":"","outputDir":"","watermarkIds":[1,2],"mode":"simultaneous",
            "interval":3,"opacity":1,"fontSize":"medium","color":"white","background":false,
            "randomOpacity":false,"randomSize":false,"safeMargin":0.05
        }))
        .unwrap();
        let frames = [FramePoint {
            timestamp: 0.0,
            pts: Some(0),
            index: 0,
        }];
        let plan = make_plan(
            &tools,
            &request,
            &frames,
            640,
            360,
            &["COPY".into(), "COPY".into()],
            101,
            &temp,
        )
        .unwrap();
        let script = temp.join("filter.txt");
        write_filter(&tools, &temp, &script, &plan.marks, &request).unwrap();
        assert_eq!(
            fs::read_to_string(&script)
                .unwrap()
                .matches("drawtext=")
                .count(),
            2
        );
        assert_ne!(
            (plan.marks[0].x, plan.marks[0].y),
            (plan.marks[1].x, plan.marks[1].y)
        );
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn synthetic_video_matrix_keeps_geometry_vfr_rotation_and_long_duration() {
        let tools = test_tools();
        let temp =
            std::env::temp_dir().join(format!("framemark-matrix-{}", crate::layout::new_seed()));
        fs::create_dir_all(&temp).unwrap();
        let cases = [
            (
                "1080p30.mp4",
                "color=c=navy:s=1920x1080:r=30:d=1",
                1920,
                1080,
                0.5,
            ),
            (
                "4k60.mp4",
                "color=c=navy:s=3840x2160:r=60:d=0.25",
                3840,
                2160,
                0.5,
            ),
            (
                "vertical.mp4",
                "color=c=navy:s=1080x1920:r=30:d=1",
                1080,
                1920,
                0.5,
            ),
            (
                "long-30min.mp4",
                "color=c=navy:s=160x96:r=1:d=1800",
                160,
                96,
                60.0,
            ),
        ];
        for (name, source, width, height, interval) in cases {
            let input = temp.join(name);
            assert!(new_command(&tools.ffmpeg)
                .args([
                    "-y",
                    "-f",
                    "lavfi",
                    "-i",
                    source,
                    "-c:v",
                    "libx264",
                    "-preset",
                    "ultrafast",
                    "-pix_fmt",
                    "yuv420p"
                ])
                .arg(&input)
                .output()
                .unwrap()
                .status
                .success());
            let job = process_test_case(&tools, &input, &temp, interval, 900 + width as u64);
            let output_info = inspect(&tools, &job.partial_output).unwrap();
            assert_eq!((output_info.width, output_info.height), (width, height));
            assert!((output_info.duration - job.media.duration).abs() < 0.25);
            cleanup(&job);
        }

        let vfr = temp.join("vfr.mp4");
        assert!(new_command(&tools.ffmpeg)
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=s=640x360:r=30:d=2",
                "-vf",
                "select='if(lt(t,1),not(mod(n,3)),1)'",
                "-fps_mode",
                "vfr",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast"
            ])
            .arg(&vfr)
            .output()
            .unwrap()
            .status
            .success());
        let input_times = frame_times(&tools, &vfr);
        let vfr_job = process_test_case(&tools, &vfr, &temp, 0.5, 8080);
        let output_times = frame_times(&tools, &vfr_job.partial_output);
        assert_eq!(input_times.len(), output_times.len());
        assert!(input_times
            .iter()
            .zip(output_times)
            .all(|(a, b)| (a - b).abs() < 0.002));
        cleanup(&vfr_job);

        let base = temp.join("rotation-base.mp4");
        let rotated = temp.join("rotation-90.mp4");
        assert!(new_command(&tools.ffmpeg)
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=navy:s=640x360:r=30:d=1",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast"
            ])
            .arg(&base)
            .output()
            .unwrap()
            .status
            .success());
        assert!(new_command(&tools.ffmpeg)
            .args(["-y", "-display_rotation", "90", "-i"])
            .arg(&base)
            .args(["-c", "copy"])
            .arg(&rotated)
            .output()
            .unwrap()
            .status
            .success());
        let rotation_info = inspect(&tools, &rotated).unwrap();
        assert_eq!(rotation_info.rotation.abs(), 90);
        assert_eq!((rotation_info.width, rotation_info.height), (360, 640));
        let rotation_job = process_test_case(&tools, &rotated, &temp, 0.5, 9090);
        let rotation_output = inspect(&tools, &rotation_job.partial_output).unwrap();
        assert_eq!(
            (
                rotation_output.width,
                rotation_output.height,
                rotation_output.rotation
            ),
            (360, 640, 0)
        );
        cleanup(&rotation_job);
        fs::remove_dir_all(temp).unwrap();
    }

    fn process_test_case(
        tools: &Tools,
        input: &Path,
        output_dir: &Path,
        interval: f64,
        seed: u64,
    ) -> PreparedJob {
        let request = ProcessRequest {
            input_path: input.to_string_lossy().into_owned(),
            output_dir: output_dir.to_string_lossy().into_owned(),
            watermark_ids: vec![1],
            mode: "simultaneous".into(),
            interval,
            opacity: 1.0,
            font_size: "small".into(),
            color: "white".into(),
            background: false,
            random_opacity: false,
            random_size: false,
            safe_margin: 0.02,
            encoder: "cpu".into(),
        };
        let job = prepare(tools, &request, &["ID1".into()], seed).unwrap();
        let result = ffmpeg_command(tools, input, &job).output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        validate_output(tools, &job.media, &job.partial_output).unwrap();
        job
    }

    fn frame_times(tools: &Tools, input: &Path) -> Vec<f64> {
        let output = new_command(&tools.ffprobe)
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_frames",
                "-show_entries",
                "frame=best_effort_timestamp_time",
                "-of",
                "csv=p=0",
            ])
            .arg(input)
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.split(',').next()?.parse().ok())
            .collect()
    }

    fn test_tools() -> Tools {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let triple = option_env!("TAURI_ENV_TARGET_TRIPLE").unwrap_or("x86_64-pc-windows-msvc");
        Tools {
            ffmpeg: manifest
                .join("binaries")
                .join(format!("ffmpeg-{triple}.exe")),
            ffprobe: manifest
                .join("binaries")
                .join(format!("ffprobe-{triple}.exe")),
            font: manifest.join("resources/NotoSansSC-Regular.ttf"),
        }
    }

    fn bright_frames(metadata: &str) -> BTreeSet<u64> {
        let mut frame = None;
        let mut result = BTreeSet::new();
        for line in metadata.lines() {
            if let Some(value) = line.strip_prefix("frame:") {
                frame = value
                    .split_whitespace()
                    .next()
                    .and_then(|value| value.parse::<u64>().ok());
            } else if let Some(ymax) = line
                .strip_prefix("lavfi.signalstats.YMAX=")
                .and_then(|value| value.parse::<u32>().ok())
            {
                if ymax > 100 {
                    if let Some(frame) = frame {
                        result.insert(frame);
                    }
                }
            }
        }
        result
    }
}
