use job::JobCommand;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

mod api;
#[allow(non_snake_case)]
mod app;
mod config;
mod job;
mod watcher;

type JobSender = UnboundedSender<JobCommand>;
type JobReceiver = UnboundedReceiver<JobCommand>;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    dotenvy::dotenv().expect(".env file not found");

    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .init();

    {
        let _c = config::CONFIG.read();
    }

    let (job_sender, job_receiver) = tokio::sync::mpsc::unbounded_channel::<JobCommand>();

    let _ = tokio::join!(
        app::start_server(job_sender.clone()),
        watcher::start_watcher(job_sender),
        job::start_job(job_receiver)
    );
}
