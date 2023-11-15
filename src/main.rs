use std::str::FromStr;

use job::JobCommand;
use once_cell::sync::Lazy;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

mod api;
#[allow(non_snake_case)]
mod app;
mod config;
mod interface;
mod job;
mod watcher;

type JobSender = UnboundedSender<JobCommand>;
type JobReceiver = UnboundedReceiver<JobCommand>;

static POOL: Lazy<SqlitePool> = Lazy::new(|| {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        SqlitePool::connect_with(
            SqliteConnectOptions::from_str("sqlite://./data/db.sqlite").unwrap(),
        )
        .await
        .unwrap()
    })
});

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().expect(".env file not found");

    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .init();

    {
        let _c = &*config::CONFIG;
        let _pool = &*POOL;
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(async {
            sqlx::migrate!().run(&*POOL).await?;

            let (job_sender, job_receiver) = tokio::sync::mpsc::unbounded_channel::<JobCommand>();

            let _ = tokio::join!(
                app::start_server(job_sender.clone()),
                watcher::start_watcher(job_sender),
                job::start_job(job_receiver)
            );

            Ok::<_, anyhow::Error>(())
        })?;

    Ok(())
}
