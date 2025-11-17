use crate::JobSender;

pub(super) mod config;
pub(super) mod fix;
pub(super) mod queue_clear;
pub(super) mod queue_info;
pub(super) mod scan;
pub(super) mod scan_all;
pub(super) mod scan_log;
pub(super) mod scan_log_clear;

pub struct AppStateInner {
    pub job_sender: JobSender,
}
pub type AppState = std::sync::Arc<AppStateInner>;
