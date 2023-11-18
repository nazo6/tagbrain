use rspc::Type;

use crate::interface::log::{LogType, ScanLog, ScanLogRaw};
use crate::interface::metadata::Metadata;
use crate::POOL;

use super::AppState;
#[derive(serde::Deserialize, Type)]
pub struct ScanLogRequest {
    limit: u32,
    page: u32,
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
                type as "type: LogType",
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
