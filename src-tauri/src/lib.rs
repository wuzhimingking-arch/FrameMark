mod db;
mod layout;
mod media;
mod models;

use crate::media::Tools;
use crate::models::*;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

struct RunningProcess {
    child: Arc<Mutex<Child>>,
    partial_output: PathBuf,
    temp_dir: PathBuf,
}

struct AppState {
    db_path: PathBuf,
    log_dir: PathBuf,
    tools: Tools,
    encoders: Vec<EncodingOption>,
    busy: AtomicBool,
    cancelled: AtomicBool,
    running: Mutex<Option<RunningProcess>>,
}

#[tauri::command]
fn bootstrap(state: State<'_, AppState>) -> Result<Bootstrap, String> {
    Ok(Bootstrap {
        identities: db::identities(&state.db_path)?,
        settings: db::settings(&state.db_path)?,
        histories: db::histories(&state.db_path)?,
        log_dir: state.log_dir.to_string_lossy().into_owned(),
        encoding_options: state.encoders.clone(),
    })
}

#[tauri::command]
fn inspect_video(state: State<'_, AppState>, path: String) -> Result<MediaInfo, String> {
    log(&state.log_dir, &format!("读取视频：{path}"));
    media::inspect(&state.tools, Path::new(&path)).map_err(|error| {
        log(&state.log_dir, &format!("读取失败：{error}"));
        error
    })
}

#[tauri::command]
fn save_identity(state: State<'_, AppState>, identity: Identity) -> Result<Identity, String> {
    db::save_identity(&state.db_path, identity)
}

#[tauri::command]
fn delete_identity(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    db::delete_identity(&state.db_path, id)
}

#[tauri::command]
fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    db::save_settings(&state.db_path, &settings)
}

#[tauri::command]
fn list_histories(state: State<'_, AppState>) -> Result<Vec<HistorySummary>, String> {
    db::histories(&state.db_path)
}

#[tauri::command]
fn history_detail(state: State<'_, AppState>, id: i64) -> Result<HistoryDetail, String> {
    db::history(&state.db_path, id)
}

#[tauri::command]
async fn process_video(app: AppHandle, request: ProcessRequest) -> Result<ProcessResult, String> {
    let state = app.state::<AppState>();
    if state.busy.swap(true, Ordering::AcqRel) {
        return Err("已有视频正在处理".into());
    }
    state.cancelled.store(false, Ordering::Release);
    let worker_app = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || run_process(&worker_app, request))
        .await
        .map_err(|e| format!("处理线程异常：{e}"));
    let state = app.state::<AppState>();
    state.busy.store(false, Ordering::Release);
    *state.running.lock().unwrap_or_else(|e| e.into_inner()) = None;
    result?
}

#[tauri::command]
fn cancel_process(state: State<'_, AppState>) -> Result<(), String> {
    state.cancelled.store(true, Ordering::Release);
    if let Some(process) = state
        .running
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_mut()
    {
        let _ = process
            .child
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .kill();
        let _ = fs::remove_file(&process.partial_output);
        let _ = fs::remove_dir_all(&process.temp_dir);
    }
    log(&state.log_dir, "用户取消处理任务");
    Ok(())
}

fn run_process(app: &AppHandle, request: ProcessRequest) -> Result<ProcessResult, String> {
    let state = app.state::<AppState>();
    let identity = db::identity(&state.db_path, request.identity_id)?;
    if !identity.enabled {
        return Err("所选平台身份已停用".into());
    }
    emit_progress(app, "正在读取真实视频帧……", 0.0, 0.0, 0.0);
    let seed = layout::new_seed();
    let job = media::prepare(&state.tools, &request, &identity.text, seed)?;
    if state.cancelled.load(Ordering::Acquire) {
        media::cleanup(&job);
        return Err("任务已取消".into());
    }
    log(
        &state.log_dir,
        &format!(
            "开始处理：{}，Seed {}，{} 个水印",
            request.input_path,
            job.seed_text,
            job.watermarks.len()
        ),
    );
    emit_progress(app, "正在生成版权标记……", 0.0, 0.0, job.media.duration);

    let attempts = media::encoding_attempts(&request.encoder, &state.encoders);
    let mut selected = None;
    let mut last_error = String::new();
    for encoder in attempts {
        emit_progress(
            app,
            &format!("视频编码：{}", encoder.label),
            0.0,
            0.0,
            job.media.duration,
        );
        match run_ffmpeg(app, &state, Path::new(&request.input_path), &job, encoder) {
            Ok(()) => {
                selected = Some(encoder);
                break;
            }
            Err(_) if state.cancelled.load(Ordering::Acquire) => {
                media::cleanup(&job);
                return Err("任务已取消".into());
            }
            Err(error) if encoder.id != "cpu" => {
                last_error = error;
                let _ = fs::remove_file(&job.partial_output);
                log(
                    &state.log_dir,
                    &format!("{} 不可用，回退 CPU：{}", encoder.label, last_error),
                );
                emit_progress(
                    app,
                    "硬件编码初始化失败，正在回退 CPU……",
                    0.0,
                    0.0,
                    job.media.duration,
                );
            }
            Err(error) => {
                media::cleanup(&job);
                return Err(format!("视频编码失败：{error}"));
            }
        }
    }
    let selected = selected.ok_or_else(|| format!("视频编码失败：{last_error}"))?;
    emit_progress(
        app,
        "正在校验输出文件……",
        99.0,
        job.media.duration,
        job.media.duration,
    );
    if let Err(error) = media::validate_output(&state.tools, &job.media, &job.partial_output) {
        media::cleanup(&job);
        return Err(error);
    }
    fs::rename(&job.partial_output, &job.output).map_err(|e| {
        media::cleanup(&job);
        format!("无法保存输出文件：{e}")
    })?;
    let _ = fs::remove_dir_all(&job.temp_dir);
    let history_id = db::insert_history(
        &state.db_path,
        &job.media,
        &job.output,
        &identity.platform,
        &identity.text,
        request.interval,
        &job.seed_text,
        &job.watermarks,
    )
    .map_err(|e| format!("视频已生成，但历史记录保存失败：{e}"))?;
    log(
        &state.log_dir,
        &format!(
            "处理完成：{}，编码 {}",
            job.output.display(),
            selected.label
        ),
    );
    emit_progress(
        app,
        "处理完成",
        100.0,
        job.media.duration,
        job.media.duration,
    );
    Ok(ProcessResult {
        history_id,
        output_path: job.output.to_string_lossy().into_owned(),
        watermark_count: job.watermarks.len(),
        seed: job.seed_text,
        encoder_label: selected.label.into(),
    })
}

fn run_ffmpeg(
    app: &AppHandle,
    state: &AppState,
    input: &Path,
    job: &media::PreparedJob,
    encoder: media::VideoEncoder,
) -> Result<(), String> {
    let mut command = media::ffmpeg_command_with_encoder(&state.tools, input, job, encoder);
    let mut child = command
        .spawn()
        .map_err(|e| format!("FFmpeg 启动失败：{e}"))?;
    let stdout = child.stdout.take().ok_or("无法读取 FFmpeg 进度")?;
    let mut stderr = child.stderr.take().ok_or("无法读取 FFmpeg 错误信息")?;
    let child = Arc::new(Mutex::new(child));
    *state.running.lock().unwrap_or_else(|e| e.into_inner()) = Some(RunningProcess {
        child: child.clone(),
        partial_output: job.partial_output.clone(),
        temp_dir: job.temp_dir.clone(),
    });
    let error_reader = std::thread::spawn(move || {
        let mut value = String::new();
        let _ = stderr.read_to_string(&mut value);
        value
    });

    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        if let Some(value) = line.strip_prefix("out_time=").and_then(parse_ffmpeg_time) {
            emit_progress(
                app,
                &format!("视频编码：{}", encoder.label),
                (value / job.media.duration * 100.0).clamp(0.0, 99.0),
                value,
                job.media.duration,
            );
        }
    }
    let status = child
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .wait()
        .map_err(|e| format!("等待 FFmpeg 失败：{e}"))?;
    let stderr = error_reader.join().unwrap_or_default();
    *state.running.lock().unwrap_or_else(|e| e.into_inner()) = None;
    if state.cancelled.load(Ordering::Acquire) {
        return Err("任务已取消".into());
    }
    if !status.success() {
        return Err(ffmpeg_error(&stderr));
    }
    Ok(())
}

fn emit_progress(app: &AppHandle, stage: &str, percent: f64, processed: f64, total: f64) {
    let _ = app.emit(
        "processing-progress",
        ProgressEvent {
            stage: stage.into(),
            percent,
            processed,
            total,
        },
    );
}

fn parse_ffmpeg_time(value: &str) -> Option<f64> {
    let mut parts = value.split(':');
    Some(
        parts.next()?.parse::<f64>().ok()? * 3600.0
            + parts.next()?.parse::<f64>().ok()? * 60.0
            + parts.next()?.parse::<f64>().ok()?,
    )
}

fn resolve_tools(app: &tauri::App) -> Result<Tools, String> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if cfg!(debug_assertions) {
        let triple = option_env!("TAURI_ENV_TARGET_TRIPLE").unwrap_or("x86_64-pc-windows-msvc");
        let suffix = if cfg!(windows) { ".exe" } else { "" };
        let bundled = |name: &str| {
            manifest
                .join("binaries")
                .join(format!("{name}-{triple}{suffix}"))
        };
        return Ok(Tools {
            ffmpeg: bundled("ffmpeg"),
            ffprobe: bundled("ffprobe"),
            font: manifest.join("resources/NotoSansSC-Regular.ttf"),
        });
    }
    let executable_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("无法确定程序目录")?
        .to_path_buf();
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    let font = [
        resource_dir.join("resources/NotoSansSC-Regular.ttf"),
        resource_dir.join("NotoSansSC-Regular.ttf"),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .ok_or("安装资源缺少 Noto Sans SC 字体")?;
    let exe = if cfg!(windows) { ".exe" } else { "" };
    Ok(Tools {
        ffmpeg: executable_dir.join(format!("ffmpeg{exe}")),
        ffprobe: executable_dir.join(format!("ffprobe{exe}")),
        font,
    })
}

fn log(dir: &Path, message: &str) {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("framemark.log"))
    {
        let _ = writeln!(file, "[{seconds}] v{} {message}", env!("CARGO_PKG_VERSION"));
    }
}

fn ffmpeg_error(stderr: &str) -> String {
    stderr
        .lines()
        .rev()
        .find(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with("frame=") && !line.starts_with("[libx264")
        })
        .unwrap_or("未知 FFmpeg 错误")
        .trim()
        .chars()
        .take(400)
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            let log_dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
            fs::create_dir_all(&data_dir)?;
            fs::create_dir_all(&log_dir)?;
            let db_path = data_dir.join("framemark.sqlite3");
            db::initialize(&db_path)?;
            let tools = resolve_tools(app)?;
            let encoders = media::detection_options(&tools);
            let detected = encoders
                .iter()
                .filter(|option| option.available && option.id != "auto" && option.id != "cpu")
                .map(|option| option.label.as_str())
                .collect::<Vec<_>>()
                .join("、");
            log(
                &log_dir,
                &format!(
                    "可用硬件编码：{}",
                    if detected.is_empty() {
                        "无"
                    } else {
                        &detected
                    }
                ),
            );
            app.manage(AppState {
                db_path,
                log_dir,
                tools,
                encoders,
                busy: AtomicBool::new(false),
                cancelled: AtomicBool::new(false),
                running: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            inspect_video,
            save_identity,
            delete_identity,
            save_settings,
            list_histories,
            history_detail,
            process_video,
            cancel_process
        ])
        .run(tauri::generate_context!())
        .expect("启动帧印失败");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_progress_timestamp() {
        assert_eq!(parse_ffmpeg_time("01:02:03.500000"), Some(3723.5));
    }
}
