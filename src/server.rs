use std::collections::HashMap;

use axum::{
  Json, ServiceExt,
  extract::Request,
  http::{StatusCode, Uri, header},
  response::{IntoResponse, Response},
  routing::get,
};
use rust_embed::Embed;
use tokio::net::TcpListener;
use tower_layer::Layer;
use tracing::Level;

use crate::badgelib::{Badge, Color};

pub type Res<T = ()> = anyhow::Result<T>;
pub type Dict = HashMap<String, String>;

// MARK: AppError

pub struct AppError(anyhow::Error);
pub type AnyRep<T> = std::result::Result<T, AppError>;

impl<E: Into<anyhow::Error>> From<E> for AppError {
  fn from(err: E) -> Self {
    Self(err.into())
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("Something went wrong: {}", self.0)).into_response()
  }
}

// MARK: BadgeError

pub struct BadgeError(anyhow::Error);
pub type BadgeRep = std::result::Result<Badge, BadgeError>;

impl<E: Into<anyhow::Error>> From<E> for BadgeError {
  fn from(err: E) -> Self {
    Self(err.into())
  }
}

impl IntoResponse for BadgeError {
  fn into_response(self) -> Response {
    tracing::error!("error: {:?}", self.0);

    let e = self.0;
    if e.downcast_ref::<reqwest::Error>().is_some() {
      let e = e.downcast_ref::<reqwest::Error>().unwrap();
      let value = e.status().map(|s| s.to_string()).unwrap_or("api error".into());
      return Badge::new("error", &value, Color::Red).into_response();
    }

    Badge::new("error", "unknown", Color::Red).into_response()
  }
}

// Other functions

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

#[derive(Embed)]
#[folder = "assets"]
struct Asset;

async fn static_handler(uri: Uri) -> impl IntoResponse {
  let mut path = uri.path().trim_start_matches('/').to_string();
  if path.starts_with("assets/") {
    path = path.replace("assets/", "");
  }

  match Asset::get(path.as_str()) {
    Some(content) => {
      let mime = mime_guess::from_path(path).first_or_octet_stream();
      ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
    }
    None => not_found().await.into_response(),
  }
}

fn rewrite_request_uri<B>(mut req: Request<B>) -> Request<B> {
  let uri = req.uri().clone();
  let path = uri.path();

  let mut path = if path == "/" { path } else { path.trim_end_matches('/') };
  let mut qs = req.uri().query().unwrap_or_default().to_string();

  let has_ext = path.ends_with(".svg") || path.ends_with(".json");
  if !path.starts_with("/assets/") && has_ext {
    let ext = path.split('.').last().unwrap();
    path = path.trim_end_matches(&format!(".{}", ext));
    qs = if qs.is_empty() { format!("format={}", ext) } else { format!("{}&format={}", qs, ext) };
  }

  qs = if qs.is_empty() { qs } else { format!("?{}", qs) };
  *req.uri_mut() = format!("{}{}", path, qs).parse().unwrap();
  req
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
    .route("/assets/{*file}", get(static_handler))
    .fallback_service(get(not_found));

  // https://docs.rs/axum/latest/axum/middleware/index.html#rewriting-request-uri-in-middleware
  // waiting for answer: https://github.com/tokio-rs/axum/discussions/3270
  let app = tower::util::MapRequestLayer::new(rewrite_request_uri).layer(app).into_make_service();

  // let app = NormalizePathLayer::trim_trailing_slash().layer(app);
  // let app = ServiceExt::<Request>::into_make_service(app);

  let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
  let port = std::env::var("PORT").unwrap_or("8080".to_string());
  let addr = format!("{}:{}", host, port);

  let listener = TcpListener::bind(&addr).await?;
  tracing::info!("listening on http://{}", addr);
  axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
  Ok(())
}
