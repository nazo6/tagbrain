use std::{path::PathBuf, str::FromStr};

use axum::{extract::WebSocketUpgrade, response::Html, routing::get, Router};
use dioxus::prelude::*;
use once_cell::sync::OnceCell;
use tracing::info;

use crate::{
    job::{JobCommand, QueueKind, ScanQueueItem},
    JobSender,
};

static JOB_SENDER: OnceCell<JobSender> = OnceCell::new();

pub async fn start_server(job_sender: JobSender) {
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 3070).into();
    JOB_SENDER.set(job_sender).unwrap();

    let view = dioxus_liveview::LiveViewPool::new();

    let app = Router::new()
        .route(
            "/",
            get(move || async move {
                Html(format!(
                    r#"
                <!DOCTYPE html>
                <html>
                <head> <title>Dioxus LiveView with Axum</title>  </head>
                <body> <div id="main"></div> </body>
                {glue}
                </html>
                "#,
                    // Create the glue code to connect to the WebSocket on the "/ws" route
                    glue = dioxus_liveview::interpreter_glue(&format!("ws://{addr}/ws"))
                ))
            }),
        )
        .route(
            "/ws",
            get(move |ws: WebSocketUpgrade| async move {
                ws.on_upgrade(move |socket| async move {
                    _ = view.launch(dioxus_liveview::axum_socket(socket), App).await;
                })
            }),
        );

    println!("Listening on http://{addr}");

    axum::Server::bind(&addr.to_string().parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap()
}

fn App(cx: Scope) -> Element {
    let file_input = use_state(cx, || "".to_string());

    cx.render(rsx! {
        input {
            r#type: "text",
            placeholder: "Enter file path",
            value: "{file_input}",
            oninput: move |e| {
                file_input.set(e.data.value.clone());
            },
        },
        button {
            onclick: move |_| {
                let Ok(path) = PathBuf::from_str(file_input) else {
                    info!("Invalid path");
                    return;
                };
                JOB_SENDER.get().unwrap().send(JobCommand::Enqueue(QueueKind::Scan(ScanQueueItem {
                    path,
                    retry_count: 0,
                }))).unwrap();
            },
            "Scan file",
        },
    })
}
