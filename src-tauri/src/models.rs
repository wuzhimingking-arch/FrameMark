use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub id: i64,
    pub platform: String,
    pub text: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub interval: f64,
    pub opacity: f64,
    pub font_size: String,
    pub output_dir: String,
    pub random_opacity: bool,
    pub random_size: bool,
    pub safe_margin: f64,
    #[serde(default = "default_encoder")]
    pub encoder: String,
}

fn default_encoder() -> String {
    "auto".into()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            interval: 3.0,
            opacity: 0.45,
            font_size: "medium".into(),
            output_dir: String::new(),
            random_opacity: false,
            random_size: false,
            safe_margin: 0.05,
            encoder: default_encoder(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub path: String,
    pub file_name: String,
    pub size: u64,
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub fps_text: String,
    pub video_codec: String,
    pub audio_codec: String,
    pub has_audio: bool,
    pub rotation: i32,
    pub hdr: bool,
    pub hdr_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRequest {
    pub input_path: String,
    pub output_dir: String,
    pub identity_id: i64,
    pub interval: f64,
    pub opacity: f64,
    pub font_size: String,
    pub color: String,
    pub background: bool,
    pub random_opacity: bool,
    pub random_size: bool,
    pub safe_margin: f64,
    #[serde(default = "default_encoder")]
    pub encoder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Watermark {
    pub number: usize,
    pub timestamp: f64,
    pub pts: Option<i64>,
    pub frame_index: u64,
    pub x: u32,
    pub y: u32,
    pub font_size: u32,
    pub opacity: f64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySummary {
    pub id: i64,
    pub original_name: String,
    pub output_path: String,
    pub platform: String,
    pub watermark_count: usize,
    pub created_at: i64,
    pub file_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryDetail {
    pub id: i64,
    pub original_name: String,
    pub original_path: String,
    pub output_name: String,
    pub output_path: String,
    pub platform: String,
    pub watermark_text: String,
    pub interval: f64,
    pub seed: String,
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub fps: f64,
    pub video_codec: String,
    pub audio_codec: String,
    pub created_at: i64,
    pub file_exists: bool,
    pub watermarks: Vec<Watermark>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub identities: Vec<Identity>,
    pub settings: Settings,
    pub histories: Vec<HistorySummary>,
    pub log_dir: String,
    pub encoding_options: Vec<EncodingOption>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodingOption {
    pub id: String,
    pub label: String,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub stage: String,
    pub percent: f64,
    pub processed: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessResult {
    pub history_id: i64,
    pub output_path: String,
    pub watermark_count: usize,
    pub seed: String,
    pub encoder_label: String,
}
