use std::path::PathBuf;

use serde::Deserialize;
use specta::Type;

use crate::router::Error;

use super::AppState;

#[derive(Deserialize, Type)]
pub struct FixRequest {
    pub target_path: String,
    pub release_id: String,
    pub recording_id: String,
}
pub async fn fix(ctx: AppState, req: FixRequest) -> Result<(), Error> {
    ctx.job_sender
        .send(crate::JobCommand::Fix {
            target_path: PathBuf::from(req.target_path),
            release_id: req.release_id,
            recording_id: req.recording_id,
        })
        .map_err(|e| {
            Error::Internal(format!(
                "Internal server error: failed to send job command: {}",
                e,
            ))
        })?;
    Ok(())
}

#[derive(Deserialize, Type)]
pub struct FixFailedRequest {
    pub source_path: String,
    pub release_id: String,
    pub recording_id: String,
}
pub async fn fix_failed(ctx: AppState, req: FixFailedRequest) -> Result<(), Error> {
    ctx.job_sender
        .send(crate::JobCommand::FixFailed {
            source_path: PathBuf::from(req.source_path),
            release_id: req.release_id,
            recording_id: req.recording_id,
        })
        .map_err(|e| {
            Error::Internal(format!(
                "Internal server error: failed to send job command: {}",
                e,
            ))
        })?;
    Ok(())
}
