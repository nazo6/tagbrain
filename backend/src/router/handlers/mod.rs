use std::sync::Arc;

use crate::JobSender;

pub(super) mod queue_info;
pub(super) mod scan;
pub(super) mod scan_all;
pub(super) mod scan_log;

pub struct AppState {
    pub job_sender: Arc<JobSender>,
}
