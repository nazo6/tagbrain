use std::str::FromStr;

use rspc::Type;

use super::AppState;

#[derive(serde::Deserialize, Type)]
pub struct ScanRequest {
    path: String,
}
pub async fn scan(ctx: AppState, req: ScanRequest) -> Result<(), rspc::Error> {
    ctx.job_sender
        .send(crate::JobCommand::Scan {
            path: std::path::PathBuf::from_str(&req.path).map_err(|e| {
                rspc::Error::new(rspc::ErrorCode::BadRequest, format!("Invalid path: {}", e))
            })?,
            retry_count: 0,
        })
        .map_err(|e| {
            rspc::Error::new(rspc::ErrorCode::BadRequest, format!("Invalid path: {}", e))
        })?;
    Ok(())
}
