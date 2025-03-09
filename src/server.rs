use std::collections::HashMap;

use axum::{
  Json, ServiceExt,
  extract::Request,
  http::{StatusCode, header},
  response::{IntoResponse, Response},
  routing::get,
};
use tokio::net::TcpListener;
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;
use tracing::Level;

pub type Res<T = ()> = anyhow::Result<T>;
pub type Rep<T> = std::result::Result<T, AppError>;
pub type Dict = HashMap<String, String>;

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("Something went wrong: {}", self.0)).into_response()
  }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
  fn from(err: E) -> Self {
    Self(err.into())
  }
}

// https://github.com/tokio-rs/axum/discussions/1894
async fn shutdown_signal() {
  use tokio::signal;

  let ctrl_c = async {
    signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
  };

  let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
      .expect("failed to install signal handler")
      .recv()
      .await;
  };

  tokio::select! {
      _ = ctrl_c => {},
      _ = terminate => {},
  }
}

async fn health() -> impl IntoResponse {
  let msg = serde_json::json!({ "status": "ok", "ver": env!("CARGO_PKG_VERSION") });
  (StatusCode::OK, axum::response::Json(msg))
}

async fn not_found() -> impl IntoResponse {
  let msg = serde_json::json!({ "code": 404, "message": "not found" });
  (StatusCode::NOT_FOUND, Json(msg))
}

async fn favicon() -> impl IntoResponse {
  let bytes = include_bytes!("../assets/favicon.ico");
  (StatusCode::OK, [(header::CONTENT_TYPE, "image/x-icon")], bytes)
}

pub async fn run_server(app: axum::Router) -> Result<(), Box<dyn std::error::Error>> {
  let app = app
    .layer(
      tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(Level::INFO))
        .on_response(tower_http::trace::DefaultOnResponse::new().level(Level::INFO)),
    )
    .route("/health", get(health))
    .route("/favicon.ico", get(favicon))
    .fallback_service(get(not_found));

  let app = NormalizePathLayer::trim_trailing_slash().layer(app);
  let app = ServiceExt::<Request>::into_make_service(app);

  let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
  let port = std::env::var("PORT").unwrap_or("8080".to_string());
  let addr = format!("{}:{}", host, port);

  let listener = TcpListener::bind(&addr).await?;
  tracing::info!("listening on http://{}", addr);
  axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
  Ok(())
}
