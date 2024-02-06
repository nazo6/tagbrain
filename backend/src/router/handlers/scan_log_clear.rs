use rspc::Type;

use crate::POOL;

use super::AppState;
#[derive(serde::Deserialize, Type, Debug)]
pub struct ScanLogClearRequest {
    clear_failed: bool,
}

#[tracing::instrument(err, skip(_ctx))]
pub async fn scan_log_clear(_ctx: AppState, req: ScanLogClearRequest) -> Result<(), rspc::Error> {
    sqlx::query!(
        r#"
            DELETE FROM log
            WHERE success = CASE WHEN ? THEN true ELSE success END"#,
        req.clear_failed
    )
    .execute(&*POOL)
    .await
    .map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("Failed to delete logs: {:?}", e),
        )
    })?;

    Ok(())
}
