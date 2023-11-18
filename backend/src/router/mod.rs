use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use tracing::info;

use crate::{router::handlers::AppState, JobSender};

#[cfg(not(debug_assertions))]
mod frontend;
mod handlers;

#[tracing::instrument(skip(job_sender))]
pub async fn start_server(job_sender: JobSender) -> eyre::Result<()> {
    let router = rspc::Router::<AppState>::new()
        .mutation("scan", |t| t(handlers::scan::scan))
        .mutation("scan_all", |t| t(handlers::scan_all::scan_all))
        .query("scan_log", |t| t(handlers::scan_log::scan_log))
        .query("queue_info", |t| t(handlers::queue_info::queue_info))
        .mutation("queue_clear", |t| t(handlers::queue_clear::queue_clear))
        .query("config_read", |t| t(handlers::config::config_read))
        .mutation("config_write", |t| t(handlers::config::config_write))
        .build()
        .arced();

    #[cfg(debug_assertions)]
    router.export_ts("../frontend/src/lib/bindings.ts").unwrap();

    let app: axum::Router<()> = axum::Router::new().nest(
        "/rspc",
        router
            .endpoint(move || AppState {
                job_sender: Arc::new(job_sender),
            })
            .axum(),
    );

    #[cfg(not(debug_assertions))]
    let app = app.fallback(frontend::static_handler);

    let addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 3080);

    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
