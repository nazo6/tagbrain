use crate::job::JobTask;

use super::AppState;

#[derive(Debug, rspc::Type, serde::Serialize)]
pub struct QueueInfo {
    pub tasks: Vec<JobTask>,
    pub running_count: u32,
}

pub async fn queue_info(ctx: AppState, _: ()) -> Result<QueueInfo, rspc::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    ctx.job_sender
        .send(crate::JobCommand::GetQueueInfo { sender: tx })
        .map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("Internal server error: failed to send job command: {}", e,),
            )
        })?;
    let info = rx.await.map_err(|e| {
        rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!(
                "Internal server error: failed to receive queue count: {}",
                e,
            ),
        )
    })?;
    Ok(QueueInfo {
        tasks: info.tasks,
        running_count: info.running_count as u32,
    })
}
