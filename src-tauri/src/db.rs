use crate::models::{HistoryDetail, HistorySummary, Identity, MediaInfo, Settings, Watermark};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub fn initialize(path: &Path) -> Result<(), String> {
    let db = Connection::open(path).map_err(error)?;
    db.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;
         CREATE TABLE IF NOT EXISTS identities (
           id INTEGER PRIMARY KEY, platform TEXT NOT NULL, text TEXT NOT NULL, enabled INTEGER NOT NULL DEFAULT 1
         );
         CREATE TABLE IF NOT EXISTS settings (
           id INTEGER PRIMARY KEY CHECK (id=1), json TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS histories (
           id INTEGER PRIMARY KEY, original_name TEXT NOT NULL, original_path TEXT NOT NULL,
           output_name TEXT NOT NULL, output_path TEXT NOT NULL, platform TEXT NOT NULL,
           watermark_text TEXT NOT NULL, interval REAL NOT NULL, seed TEXT NOT NULL,
           width INTEGER NOT NULL, height INTEGER NOT NULL, duration REAL NOT NULL, fps REAL NOT NULL,
           video_codec TEXT NOT NULL, audio_codec TEXT NOT NULL, created_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS watermarks (
           history_id INTEGER NOT NULL REFERENCES histories(id) ON DELETE CASCADE,
           number INTEGER NOT NULL, timestamp REAL NOT NULL, pts INTEGER, frame_index INTEGER NOT NULL,
           x INTEGER NOT NULL, y INTEGER NOT NULL, font_size INTEGER NOT NULL, opacity REAL NOT NULL,
           text TEXT NOT NULL, PRIMARY KEY(history_id, number)
         );",
    )
    .map_err(error)?;
    if db
        .query_row("SELECT 1 FROM settings WHERE id=1", [], |_| Ok(()))
        .optional()
        .map_err(error)?
        .is_none()
    {
        db.execute(
            "INSERT INTO settings(id,json) VALUES(1,?1)",
            [serde_json::to_string(&Settings::default()).map_err(error)?],
        )
        .map_err(error)?;
    }
    Ok(())
}

fn connection(path: &Path) -> Result<Connection, String> {
    let db = Connection::open(path).map_err(error)?;
    db.execute_batch("PRAGMA foreign_keys=ON;").map_err(error)?;
    Ok(db)
}

pub fn identities(path: &Path) -> Result<Vec<Identity>, String> {
    let db = connection(path)?;
    let mut stmt = db
        .prepare("SELECT id,platform,text,enabled FROM identities ORDER BY id")
        .map_err(error)?;
    let result = stmt
        .query_map([], |row| {
            Ok(Identity {
                id: row.get(0)?,
                platform: row.get(1)?,
                text: row.get(2)?,
                enabled: row.get(3)?,
            })
        })
        .map_err(error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(error);
    result
}

pub fn save_identity(path: &Path, identity: Identity) -> Result<Identity, String> {
    let platform = identity.platform.trim();
    let text = identity.text.trim();
    if platform.is_empty() || text.is_empty() {
        return Err("平台名称和水印文字不能为空".into());
    }
    let db = connection(path)?;
    if identity.id == 0 {
        db.execute(
            "INSERT INTO identities(platform,text,enabled) VALUES(?1,?2,?3)",
            params![platform, text, identity.enabled],
        )
        .map_err(error)?;
        Ok(Identity {
            id: db.last_insert_rowid(),
            platform: platform.into(),
            text: text.into(),
            enabled: identity.enabled,
        })
    } else {
        if db
            .execute(
                "UPDATE identities SET platform=?1,text=?2,enabled=?3 WHERE id=?4",
                params![platform, text, identity.enabled, identity.id],
            )
            .map_err(error)?
            == 0
        {
            return Err("平台身份不存在".into());
        }
        Ok(Identity {
            platform: platform.into(),
            text: text.into(),
            ..identity
        })
    }
}

pub fn delete_identity(path: &Path, id: i64) -> Result<(), String> {
    connection(path)?
        .execute("DELETE FROM identities WHERE id=?1", [id])
        .map_err(error)?;
    Ok(())
}

pub fn settings(path: &Path) -> Result<Settings, String> {
    let db = connection(path)?;
    let json: String = db
        .query_row("SELECT json FROM settings WHERE id=1", [], |row| row.get(0))
        .map_err(error)?;
    serde_json::from_str(&json).map_err(error)
}

pub fn save_settings(path: &Path, value: &Settings) -> Result<(), String> {
    validate_settings(value)?;
    connection(path)?
        .execute(
            "UPDATE settings SET json=?1 WHERE id=1",
            [serde_json::to_string(value).map_err(error)?],
        )
        .map_err(error)?;
    Ok(())
}

pub fn validate_settings(value: &Settings) -> Result<(), String> {
    if !(0.5..=60.0).contains(&value.interval) {
        return Err("水印间隔必须在 0.5 到 60 秒之间".into());
    }
    if !(0.2..=1.0).contains(&value.opacity) {
        return Err("透明度必须在 20% 到 100% 之间".into());
    }
    if !(0.02..=0.20).contains(&value.safe_margin) {
        return Err("安全边距必须在 2% 到 20% 之间".into());
    }
    if !matches!(value.font_size.as_str(), "small" | "medium" | "large") {
        return Err("字号设置无效".into());
    }
    Ok(())
}

pub fn identity(path: &Path, id: i64) -> Result<Identity, String> {
    connection(path)?
        .query_row(
            "SELECT id,platform,text,enabled FROM identities WHERE id=?1",
            [id],
            |row| {
                Ok(Identity {
                    id: row.get(0)?,
                    platform: row.get(1)?,
                    text: row.get(2)?,
                    enabled: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(error)?
        .ok_or_else(|| "请选择有效的平台身份".into())
}

pub fn histories(path: &Path) -> Result<Vec<HistorySummary>, String> {
    let db = connection(path)?;
    let mut stmt = db
        .prepare(
            "SELECT h.id,h.original_name,h.output_path,h.platform,COUNT(w.number),h.created_at
         FROM histories h LEFT JOIN watermarks w ON w.history_id=h.id
         GROUP BY h.id ORDER BY h.created_at DESC",
        )
        .map_err(error)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, usize>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .map_err(error)?;
    rows.map(|row| {
        let (id, original_name, output_path, platform, watermark_count, created_at) =
            row.map_err(error)?;
        Ok(HistorySummary {
            id,
            original_name,
            file_exists: Path::new(&output_path).exists(),
            output_path,
            platform,
            watermark_count,
            created_at,
        })
    })
    .collect()
}

pub fn history(path: &Path, id: i64) -> Result<HistoryDetail, String> {
    let db = connection(path)?;
    let mut detail = db.query_row(
        "SELECT id,original_name,original_path,output_name,output_path,platform,watermark_text,interval,seed,width,height,duration,fps,video_codec,audio_codec,created_at FROM histories WHERE id=?1",
        [id], |row| Ok(HistoryDetail {
            id: row.get(0)?, original_name: row.get(1)?, original_path: row.get(2)?, output_name: row.get(3)?, output_path: row.get(4)?, platform: row.get(5)?, watermark_text: row.get(6)?, interval: row.get(7)?, seed: row.get(8)?, width: row.get(9)?, height: row.get(10)?, duration: row.get(11)?, fps: row.get(12)?, video_codec: row.get(13)?, audio_codec: row.get(14)?, created_at: row.get(15)?, file_exists: false, watermarks: vec![]
        }),
    ).optional().map_err(error)?.ok_or_else(|| "历史记录不存在".to_string())?;
    detail.file_exists = Path::new(&detail.output_path).exists();
    let mut stmt = db.prepare("SELECT number,timestamp,pts,frame_index,x,y,font_size,opacity,text FROM watermarks WHERE history_id=?1 ORDER BY number").map_err(error)?;
    detail.watermarks = stmt
        .query_map([id], |row| {
            Ok(Watermark {
                number: row.get(0)?,
                timestamp: row.get(1)?,
                pts: row.get(2)?,
                frame_index: row.get(3)?,
                x: row.get(4)?,
                y: row.get(5)?,
                font_size: row.get(6)?,
                opacity: row.get(7)?,
                text: row.get(8)?,
            })
        })
        .map_err(error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(error)?;
    Ok(detail)
}

pub fn insert_history(
    path: &Path,
    media: &MediaInfo,
    output: &Path,
    platform: &str,
    text: &str,
    interval: f64,
    seed: &str,
    watermarks: &[Watermark],
) -> Result<i64, String> {
    let mut db = connection(path)?;
    let tx = db.transaction().map_err(error)?;
    let output_name = output
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    tx.execute(
        "INSERT INTO histories(original_name,original_path,output_name,output_path,platform,watermark_text,interval,seed,width,height,duration,fps,video_codec,audio_codec,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        params![media.file_name, media.path, output_name, output.to_string_lossy(), platform, text, interval, seed, media.width, media.height, media.duration, media.fps, media.video_codec, media.audio_codec, created_at],
    ).map_err(error)?;
    let id = tx.last_insert_rowid();
    {
        let mut stmt = tx.prepare("INSERT INTO watermarks(history_id,number,timestamp,pts,frame_index,x,y,font_size,opacity,text) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)").map_err(error)?;
        for mark in watermarks {
            stmt.execute(params![
                id,
                mark.number,
                mark.timestamp,
                mark.pts,
                mark.frame_index,
                mark.x,
                mark.y,
                mark.font_size,
                mark.opacity,
                mark.text
            ])
            .map_err(error)?;
        }
    }
    tx.commit().map_err(error)?;
    Ok(id)
}

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
