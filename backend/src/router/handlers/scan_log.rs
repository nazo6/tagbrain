use rspc::Type;

use crate::interface::metadata::Metadata;
use crate::POOL;

use super::AppState;
#[derive(serde::Deserialize, Type)]
pub struct ScanLogRequest {
    limit: u32,
    page: u32,
}

pub struct ScanLogRaw {
    id: i64,
    created_at: chrono::NaiveDateTime,
    success: bool,
    message: Option<String>,
    old_metadata: Option<sqlx::types::Json<Metadata>>,
    new_metadata: Option<sqlx::types::Json<Metadata>>,
    source_path: String,
    target_path: Option<String>,
    acoustid_score: Option<f64>,
    retry_count: i64,
}
#[derive(serde::Serialize, Type)]
pub struct ScanLog {
    id: i32,
    created_at: i32,
    success: bool,
    message: Option<String>,
    old_metadata: Option<Metadata>,
    new_metadata: Option<Metadata>,
    source_path: String,
    target_path: Option<String>,
    acoustid_score: Option<f32>,
    retry_count: i32,
}
impl From<ScanLogRaw> for ScanLog {
    fn from(raw: ScanLogRaw) -> Self {
        Self {
            id: raw.id as i32,
            created_at: raw.created_at.timestamp() as i32,
            success: raw.success,
            message: raw.message,
            old_metadata: raw.old_metadata.map(|x| x.0),
            new_metadata: raw.new_metadata.map(|x| x.0),
            source_path: raw.source_path,
            target_path: raw.target_path,
            acoustid_score: raw.acoustid_score.map(|x| x as f32),
            retry_count: raw.retry_count as i32,
        }
    }
}

pub async fn scan_log(
    _ctx: AppState,
    req: ScanLogRequest,
) -> Result<(Vec<ScanLog>, i32), rspc::Error> {
    let offset = req.limit * req.page;
    let res = sqlx::query_as!(
        ScanLogRaw,
        r#"
            SELECT 
                id as "id: i64",
                created_at,
                success,
                message,
                old_metadata as "old_metadata?: sqlx::types::Json<Metadata>",
                new_metadata as "new_metadata?: sqlx::types::Json<Metadata>",
                source_path,
                target_path,
                acoustid_score,
                retry_count
            FROM log ORDER BY id DESC LIMIT ? OFFSET ?"#,
        req.limit,
        offset
    )
    .fetch_all(&*POOL)
    .await
    .map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("Failed to query db: {:?}", e),
        )
    })?;

    let total_items = sqlx::query!("SELECT COUNT(*) as count FROM log")
        .fetch_one(&*POOL)
        .await
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("Failed to query db: {:?}", e),
            )
        })?
        .count;

    Ok((res.into_iter().map(|x| x.into()).collect(), total_items))
}
