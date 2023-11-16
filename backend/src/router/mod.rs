use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{routing, Router, Server};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;

use crate::JobSender;

#[tracing::instrument]
pub async fn start_server(job_sender: JobSender) -> eyre::Result<()> {
    #[derive(OpenApi)]
    #[openapi(paths(api::scan_all, api::scan), components(schemas(api::ScanRequest)))]
    struct ApiDoc;

    #[cfg(debug_assertions)]
    {
        let openapi = ApiDoc::openapi().to_pretty_json().unwrap();
        tokio::fs::write(
            format!("{}/../frontend/openapi.json", env!("CARGO_MANIFEST_DIR")),
            openapi,
        )
        .await?;
    }

    let store = Arc::new(api::AppState::new(job_sender));
    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/scan_all", routing::post(api::scan_all))
                .route("/scan", routing::post(api::scan))
                .with_state(store),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

mod api {
    use std::{str::FromStr, sync::Arc};

    use axum::{
        extract::State,
        response::{IntoResponse, Response},
        Json,
    };
    use reqwest::StatusCode;
    use utoipa::ToSchema;

    use crate::JobSender;

    pub(super) struct AppState {
        job_sender: JobSender,
    }
    impl AppState {
        pub fn new(job_sender: JobSender) -> Self {
            Self { job_sender }
        }
    }

    #[utoipa::path(
        post,
        path = "/api/scan_all",
        responses(
            (status = 200, description = "Scan all files in source folder", body = [()]),
            (status = 500, description = "Failed to send job", body = [()])
        )
    )]
    pub(super) async fn scan_all(State(store): State<Arc<AppState>>) -> Result<Json<()>, Response> {
        store
            .job_sender
            .send(crate::JobCommand::ScanAll)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to send job: {:?}", e),
                )
                    .into_response()
            })?;
        Ok(Json(()))
    }

    #[derive(serde::Deserialize, ToSchema)]
    pub struct ScanRequest {
        path: String,
    }
    #[utoipa::path(
        post,
        path = "/api/scan",
        request_body = ScanRequest,
        responses(
            (status = 200, description = "Scan file"),
            (status = 500, description = "Failed to send job"),
            (status = 400, description = "Invalid request")
        )
    )]
    pub(super) async fn scan(
        State(store): State<Arc<AppState>>,
        Json(req): Json<ScanRequest>,
    ) -> Result<Json<()>, Response> {
        store
            .job_sender
            .send(crate::JobCommand::Scan {
                path: std::path::PathBuf::from_str(&req.path).map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Invalid path: {:?}", e)).into_response()
                })?,
                retry_count: 0,
            })
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to send job: {:?}", e),
                )
                    .into_response()
            })?;
        Ok(Json(()))
    }
}
