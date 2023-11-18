use super::AppState;

pub async fn queue_clear(ctx: AppState, _: ()) -> Result<(), rspc::Error> {
    ctx.job_sender
        .send(crate::JobCommand::ClearQueue)
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("Internal server error: failed to send job command: {}", e,),
            )
        })?;
    Ok(())
}
