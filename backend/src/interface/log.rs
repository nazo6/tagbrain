use super::metadata::Metadata;

#[derive(sqlx::Type, specta::Type, serde::Serialize)]
#[repr(i32)]
pub enum LogType {
    Scan = 0,
    Fix = 1,
}

pub struct ScanLogRaw {
    pub id: i64,
    pub r#type: LogType,
    pub created_at: chrono::NaiveDateTime,
    pub success: bool,
    pub message: Option<String>,
    pub old_metadata: Option<sqlx::types::Json<Metadata>>,
    pub new_metadata: Option<sqlx::types::Json<Metadata>>,
    pub source_path: String,
    pub target_path: Option<String>,
    pub acoustid_score: Option<f64>,
    pub retry_count: Option<i64>,
}

#[derive(serde::Serialize, specta::Type)]
pub struct ScanLog {
    pub id: i32,
    pub r#type: LogType,
    pub created_at: i32,
    pub success: bool,
    pub message: Option<String>,
    pub old_metadata: Option<Metadata>,
    pub new_metadata: Option<Metadata>,
    pub source_path: String,
    pub target_path: Option<String>,
    pub acoustid_score: Option<f32>,
    pub retry_count: Option<i32>,
}
impl From<ScanLogRaw> for ScanLog {
    fn from(raw: ScanLogRaw) -> Self {
        Self {
            id: raw.id as i32,
            r#type: raw.r#type,
            created_at: raw.created_at.timestamp() as i32,
            success: raw.success,
            message: raw.message,
            old_metadata: raw.old_metadata.map(|x| x.0),
            new_metadata: raw.new_metadata.map(|x| x.0),
            source_path: raw.source_path,
            target_path: raw.target_path,
            acoustid_score: raw.acoustid_score.map(|x| x as f32),
            retry_count: raw.retry_count.map(|x| x as i32),
        }
    }
}
