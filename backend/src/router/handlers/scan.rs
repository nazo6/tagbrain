use std::{str::FromStr, sync::Arc};

use specta::Type;

use crate::router::Error;

use super::AppState;

#[derive(serde::Deserialize, Type)]
pub struct ScanRequest {
    path: String,
}
pub async fn scan(ctx: Arc<AppState>, req: ScanRequest) -> Result<(), Error> {
    ctx.job_sender
        .send(crate::JobCommand::Scan {
            path: std::path::PathBuf::from_str(&req.path)
                .map_err(|e| Error::BadRequest(format!("Invalid path: {}", e)))?,
            retry_count: 0,
        })
        .map_err(|e| Error::Internal(format!("Failed to send scan job: {}", e)))?;
    Ok(())
}
