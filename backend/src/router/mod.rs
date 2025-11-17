use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use rspc::Procedure;
use tracing::info;

use crate::{
    router::handlers::{AppState, AppStateInner},
    JobSender,
};

#[cfg(not(debug_assertions))]
mod frontend;
mod handlers;

#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
#[serde(tag = "type", content = "error")]
pub enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Unexpected error: {0}")]
    #[serde(skip)]
    Any(#[from] eyre::Error),
}

impl rspc::Error for Error {
    fn into_procedure_error(self) -> rspc::ProcedureError {
        // rspc::ResolverError::new(self.to_string(), Some(self)) // TODO: Typesafe way to achieve this
        rspc::ResolverError::new(
            self,
            None::<std::io::Error>, // TODO: `Some(self)` but `anyhow::Error` is not `Clone`
        )
        .into()
    }
}

#[tracing::instrument(skip(job_sender))]
pub async fn start_server(job_sender: JobSender) -> eyre::Result<()> {
    let router = rspc::Router::<AppState>::new()
        .procedure("scan", Procedure::builder().mutation(handlers::scan::scan))
        .procedure(
            "scan_all",
            Procedure::builder().mutation(handlers::scan_all::scan_all),
        )
        .procedure(
            "scan_log",
            Procedure::builder().query(handlers::scan_log::scan_log),
        )
        .procedure(
            "scan_log_clear",
            Procedure::builder().mutation(handlers::scan_log_clear::scan_log_clear),
        )
        .procedure(
            "queue_info",
            Procedure::builder().query(handlers::queue_info::queue_info),
        )
        .procedure(
            "queue_clear",
            Procedure::builder().mutation(handlers::queue_clear::queue_clear),
        )
        .procedure(
            "config_read",
            Procedure::builder().query(handlers::config::config_read),
        )
        .procedure(
            "config_write",
            Procedure::builder().mutation(handlers::config::config_write),
        )
        .procedure("fix", Procedure::builder().mutation(handlers::fix::fix))
        .procedure(
            "fix_failed",
            Procedure::builder().mutation(handlers::fix::fix_failed),
        );
    let (procedures, types) = router.build().unwrap();

    #[cfg(debug_assertions)]
    rspc::Typescript::default()
        .export_to(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../frontend/src/lib/bindings.ts"),
            &types,
        )
        .unwrap();

    let app_state = Arc::new(AppStateInner { job_sender });

    let app: axum::Router<()> = axum::Router::new().nest(
        "/rspc",
        rspc_axum::endpoint(procedures, move || app_state.clone()),
    );

    #[cfg(not(debug_assertions))]
    let app = app.fallback(frontend::static_handler);

    let addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 3080);

    info!("Listening on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();

    Ok(())
}
