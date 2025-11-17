use crate::router::Error;

use super::AppState;

pub async fn scan_all(ctx: AppState, _: ()) -> Result<(), Error> {
    ctx.job_sender
        .send(crate::JobCommand::ScanAll)
        .map_err(|e| {
            Error::Internal(format!(
                "Internal server error: failed to send job command: {}",
                e,
            ))
        })?;
    Ok(())
}
