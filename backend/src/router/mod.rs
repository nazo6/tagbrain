use std::{
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::http::request::Parts;
use rspc::{Procedure, ProcedureBuilder, ResolverInput, ResolverOutput};
use tracing::info;

use crate::{router::handlers::AppState, JobSender};

#[cfg(not(debug_assertions))]
mod frontend;
mod handlers;

#[derive(Clone)]
pub struct Ctx {}

#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
#[serde(tag = "type", content = "error")]
pub enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("internal error: {0}")]
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

pub struct BaseProcedure<TErr = Error>(PhantomData<TErr>);
impl<TErr> BaseProcedure<TErr> {
    pub fn builder<TInput, TResult>(
    ) -> ProcedureBuilder<TErr, Ctx, Ctx, TInput, TInput, TResult, TResult>
    where
        TErr: rspc::Error,
        TInput: ResolverInput,
        TResult: ResolverOutput<TErr>,
    {
        Procedure::builder()
    }
}

#[tracing::instrument(skip(job_sender))]
pub async fn start_server(job_sender: JobSender) -> eyre::Result<()> {
    let router = rspc::Router::<Arc<AppState>>::new()
        .procedure(
            "scan",
            Procedure::builder().mutation(|ctx: Arc<AppState>, req| handlers::scan::scan(ctx, req)),
        )
        // .mutation("scan_all", |t| t(handlers::scan_all::scan_all))
        // .query("scan_log", |t| t(handlers::scan_log::scan_log))
        // .mutation("scan_log_clear", |t| {
        //     t(handlers::scan_log_clear::scan_log_clear)
        // })
        // .query("queue_info", |t| t(handlers::queue_info::queue_info))
        // .mutation("queue_clear", |t| t(handlers::queue_clear::queue_clear))
        // .query("config_read", |t| t(handlers::config::config_read))
        // .mutation("config_write", |t| t(handlers::config::config_write))
        // .mutation("fix", |t| t(handlers::fix::fix))
        // .mutation("fix_failed", |t| t(handlers::fix::fix_failed))
        ;
    let (procedures, types) = router.build().unwrap();

    #[cfg(debug_assertions)]
    rspc::Typescript::default()
        .export_to(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../frontend/src/lib/bindings.ts"),
            &types,
        )
        .unwrap();

    let app_state = Arc::new(AppState { job_sender });

    let app: axum::Router<()> = axum::Router::new().nest(
        "/rspc",
        rspc_axum::endpoint(procedures, move |parts: Parts| app_state.clone()),
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
