use specta::Type;

use crate::{router::Error, POOL};

use super::AppState;
#[derive(serde::Deserialize, Type, Debug)]
pub struct ScanLogClearRequest {
    clear_failed: bool,
}

#[tracing::instrument(err, skip(_ctx))]
pub async fn scan_log_clear(_ctx: AppState, req: ScanLogClearRequest) -> Result<(), Error> {
    sqlx::query!(
        r#"
            DELETE FROM log
            WHERE success = CASE WHEN ? THEN success ELSE true END"#,
        req.clear_failed
    )
    .execute(&*POOL)
    .await
    .map_err(|e| Error::Internal(format!("Failed to delete logs: {:?}", e)))?;

    Ok(())
}
