use axum::{extract::WebSocketUpgrade, response::Html, routing::get, Router};
use dioxus::prelude::*;

use crate::JobSender;

pub async fn start_server(job_sender: JobSender) {
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 3070).into();

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
    cx.render(rsx! {
        div {
            "Hello world!"
        }
    })
}
