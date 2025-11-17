use crate::router::Error;

use super::AppState;

pub async fn queue_clear(ctx: AppState, _: ()) -> Result<(), Error> {
    ctx.job_sender
        .send(crate::JobCommand::ClearQueue)
        .map_err(|e| {
            Error::Internal(format!(
                "Internal server error: failed to send job command: {}",
                e,
            ))
        })?;
    Ok(())
}
