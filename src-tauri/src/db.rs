use crate::models::{
    HistoryDetail, HistorySummary, Identity, MediaInfo, Settings, Watermark, WatermarkPlan,
};
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
    migrate(&db)?;
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

fn migrate(db: &Connection) -> Result<(), String> {
    let version: i64 = db
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(error)?;
    if version < 1 {
        db.execute_batch(
            "BEGIN IMMEDIATE;
            ALTER TABLE identities ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
            UPDATE identities SET sort_order=id;
            ALTER TABLE histories ADD COLUMN plan_json TEXT;
            PRAGMA user_version=1;
            COMMIT;",
        )
        .map_err(error)?;
    }
    Ok(())
}

pub fn reorder_identities(path: &Path, ids: &[i64]) -> Result<(), String> {
    let mut db = connection(path)?;
    let tx = db.transaction().map_err(error)?;
    let all: Vec<i64> = {
        let mut stmt = tx.prepare("SELECT id FROM identities").map_err(error)?;
        let rows = stmt
            .query_map([], |r| r.get(0))
            .map_err(error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(error)?;
        rows
    };
    let unique: std::collections::BTreeSet<_> = ids.iter().copied().collect();
    if ids.len() != unique.len() || unique != all.iter().copied().collect() {
        return Err("水印顺序必须包含全部水印且不能重复".into());
    }
    for (i, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE identities SET sort_order=?1 WHERE id=?2",
            params![i, id],
        )
        .map_err(error)?;
    }
    tx.commit().map_err(error)
}

pub fn selected_texts(path: &Path, ids: &[i64]) -> Result<Vec<String>, String> {
    if ids.is_empty() {
        return Err("请至少选择一条水印".into());
    }
    ids.iter()
        .map(|id| {
            let item = identity(path, *id)?;
            crate::layout::validate_text(&item.text)?;
            Ok(item.text)
        })
        .collect()
}

fn connection(path: &Path) -> Result<Connection, String> {
    let db = Connection::open(path).map_err(error)?;
    db.execute_batch("PRAGMA foreign_keys=ON;").map_err(error)?;
    Ok(db)
}

pub fn identities(path: &Path) -> Result<Vec<Identity>, String> {
    let db = connection(path)?;
    let mut stmt = db
        .prepare("SELECT id,text,enabled FROM identities ORDER BY sort_order,id")
        .map_err(error)?;
    let result = stmt
        .query_map([], |row| {
            Ok(Identity {
                id: row.get(0)?,
                text: row.get(1)?,
                enabled: row.get(2)?,
            })
        })
        .map_err(error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(error);
    result
}

pub fn save_identity(path: &Path, identity: Identity) -> Result<Identity, String> {
    let text = identity.text.trim();
    crate::layout::validate_text(text)?;
    let db = connection(path)?;
    if identity.id == 0 {
        db.execute(
            "INSERT INTO identities(platform,text,enabled,sort_order) VALUES('',?1,?2,(SELECT COALESCE(MAX(sort_order),0)+1 FROM identities))",
            params![text, identity.enabled],
        )
        .map_err(error)?;
        Ok(Identity {
            id: db.last_insert_rowid(),
            text: text.into(),
            enabled: identity.enabled,
        })
    } else {
        if db
            .execute(
                "UPDATE identities SET text=?1,enabled=?2 WHERE id=?3",
                params![text, identity.enabled, identity.id],
            )
            .map_err(error)?
            == 0
        {
            return Err("水印信息不存在".into());
        }
        Ok(Identity {
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
    if !matches!(
        value.encoder.as_str(),
        "auto" | "nvenc" | "qsv" | "amf" | "videotoolbox" | "cpu"
    ) {
        return Err("视频编码加速设置无效".into());
    }
    Ok(())
}

pub fn identity(path: &Path, id: i64) -> Result<Identity, String> {
    connection(path)?
        .query_row(
            "SELECT id,text,enabled FROM identities WHERE id=?1",
            [id],
            |row| {
                Ok(Identity {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    enabled: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(error)?
        .ok_or_else(|| "请选择有效的水印信息".into())
}

pub fn histories(path: &Path) -> Result<Vec<HistorySummary>, String> {
    let db = connection(path)?;
    let mut stmt = db
        .prepare(
            "SELECT h.id,h.original_name,h.output_path,h.platform,COUNT(w.number),h.created_at,COUNT(DISTINCT w.frame_index)
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
                row.get::<_, usize>(6)?,
            ))
        })
        .map_err(error)?;
    rows.map(|row| {
        let (id, original_name, output_path, platform, watermark_count, created_at, event_count) =
            row.map_err(error)?;
        Ok(HistorySummary {
            id,
            original_name,
            file_exists: Path::new(&output_path).exists(),
            output_path,
            platform,
            watermark_count,
            event_count,
            created_at,
        })
    })
    .collect()
}

pub fn history(path: &Path, id: i64) -> Result<HistoryDetail, String> {
    let db = connection(path)?;
    let mut detail = db.query_row(
        "SELECT id,original_name,original_path,output_name,output_path,platform,watermark_text,interval,seed,width,height,duration,fps,video_codec,audio_codec,created_at,plan_json FROM histories WHERE id=?1",
        [id], |row| Ok(HistoryDetail {
            id: row.get(0)?, original_name: row.get(1)?, original_path: row.get(2)?, output_name: row.get(3)?, output_path: row.get(4)?, platform: row.get(5)?, watermark_text: row.get(6)?, interval: row.get(7)?, seed: row.get(8)?, width: row.get(9)?, height: row.get(10)?, duration: row.get(11)?, fps: row.get(12)?, video_codec: row.get(13)?, audio_codec: row.get(14)?, created_at: row.get(15)?, file_exists: false, watermarks: vec![], plan: WatermarkPlan { algorithm_version:1, mode:"simultaneous".into(), texts:vec![row.get(6)?],event_count:0,marks:vec![], config:None }
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
                event: row.get(0)?,
                width: 0,
                height: 0,
                rendered_text: row.get(8)?,
            })
        })
        .map_err(error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(error)?;
    let json: Option<String> = db
        .query_row("SELECT plan_json FROM histories WHERE id=?1", [id], |r| {
            r.get(0)
        })
        .map_err(error)?;
    if let Some(json) = json {
        detail.plan = serde_json::from_str(&json).map_err(error)?;
        detail.watermarks = detail.plan.marks.clone();
    } else {
        detail.plan.event_count = detail.watermarks.len();
        detail.plan.marks = detail.watermarks.clone();
    }
    Ok(detail)
}

pub fn insert_history(
    path: &Path,
    media: &MediaInfo,
    output: &Path,
    plan: &WatermarkPlan,
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
        params![media.file_name, media.path, output_name, output.to_string_lossy(), "", plan.texts.join("\n"), interval, seed, media.width, media.height, media.duration, media.fps, media.video_codec, media.audio_codec, created_at],
    ).map_err(error)?;
    let id = tx.last_insert_rowid();
    tx.execute(
        "UPDATE histories SET plan_json=?1 WHERE id=?2",
        params![serde_json::to_string(plan).map_err(error)?, id],
    )
    .map_err(error)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_crud_order_and_history_snapshots_survive_restart() {
        let dir = std::env::temp_dir().join(format!("framemark-db-{}", crate::layout::new_seed()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("test.sqlite3");
        initialize(&path).unwrap();
        let db = Connection::open(&path).unwrap();
        db.execute_batch("ALTER TABLE identities DROP COLUMN sort_order;
            ALTER TABLE histories DROP COLUMN plan_json; PRAGMA user_version=0;
            INSERT INTO identities VALUES(7,'Bilibili','UID:旧数据',1);
            INSERT INTO histories VALUES(1,'demo.mp4','missing.mp4','marked.mp4','missing-output.mp4','Bilibili','UID:旧数据',3,'ABCD',640,360,5,30,'h264','aac',1);
            INSERT INTO watermarks VALUES(1,1,1.2,36,36,50,60,16,0.45,'UID:旧数据');").unwrap();
        drop(db);
        initialize(&path).unwrap();
        initialize(&path).unwrap();
        assert_eq!(identities(&path).unwrap().len(), 1);
        assert_eq!(identities(&path).unwrap()[0].text, "UID:旧数据");
        let created = save_identity(
            &path,
            Identity {
                id: 0,
                text: "  © 公司\nhttps://example.test @name  ".into(),
                enabled: true,
            },
        )
        .unwrap();
        assert_eq!(created.text, "© 公司\nhttps://example.test @name");
        let copy = save_identity(
            &path,
            Identity {
                id: 0,
                ..created.clone()
            },
        )
        .unwrap();
        let edited = save_identity(
            &path,
            Identity {
                text: "自定义修改".into(),
                ..copy.clone()
            },
        )
        .unwrap();
        assert_eq!(edited.text, "自定义修改");
        reorder_identities(&path, &[copy.id, 7, created.id]).unwrap();
        assert!(reorder_identities(&path, &[7, 7, created.id]).is_err());
        let mut setting = settings(&path).unwrap();
        setting.selected_ids = vec![copy.id, created.id];
        save_settings(&path, &setting).unwrap();
        initialize(&path).unwrap();
        assert_eq!(
            settings(&path).unwrap().selected_ids,
            vec![copy.id, created.id]
        );
        assert_eq!(identities(&path).unwrap()[0].id, copy.id);
        delete_identity(&path, 7).unwrap();
        let old = history(&path, 1).unwrap();
        assert_eq!(old.watermarks[0].text, "UID:旧数据");
        assert_eq!(old.plan.algorithm_version, 1);
        assert!(!old.file_exists);
        assert_eq!(histories(&path).unwrap()[0].event_count, 1);
        assert!(save_identity(
            &path,
            Identity {
                id: 0,
                text: " ".into(),
                enabled: true
            }
        )
        .is_err());
        drop(old);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
