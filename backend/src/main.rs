use std::str::FromStr;

use job::JobCommand;
use once_cell::sync::Lazy;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing_error::ErrorLayer;

mod api;
mod config;
mod interface;
mod job;
mod router;
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

fn install_tracing() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false).pretty();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    install_tracing();

    dotenvy::dotenv().expect(".env file not found");

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

            Ok::<_, eyre::Report>(())
        })?;

    Ok(())
}
