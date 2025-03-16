use axum::routing::get;
use tracing_subscriber::layer::SubscriberExt;

mod apis;
mod badgelib;
mod pages;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let env_filter = tracing_subscriber::EnvFilter::builder()
    .with_default_directive(tracing::Level::INFO.into())
    .from_env_lossy();

  let logfmt = tracing_logfmt::builder()
    .with_target(false)
    .with_span_name(false)
    .with_span_path(false)
    .layer();

  tracing::dispatcher::set_global_default(tracing::Dispatch::new(
    tracing_subscriber::Registry::default().with(env_filter).with(logfmt),
  ))?;

  let brand = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
  tracing::info!("{}", brand);

  let app = axum::Router::new()
    .route("/pypi/{kind}/{name}", get(apis::pypi::handler))
    .route("/npm/{kind}/{*name}", get(apis::npm::handler)) // name can be scoped
    .route("/packagephobia/{kind}/{*name}", get(apis::packagephobia::handler)) // name can be scoped
    .route("/crates/{kind}/{name}", get(apis::crates::handler))
    .route("/packagist/{kind}/{*name}", get(apis::packagist::handler))
    .route("/gem/{kind}/{*name}", get(apis::gems::handler))
    .route("/pub/{kind}/{*name}", get(apis::dart_pub::handler))
    .route("/hackage/{kind}/{*name}", get(apis::hackage::handler))
    .route("/nuget/{kind}/{*name}", get(apis::nuget::handler))
    .route("/homebrew/{kind}/cask/{name}", get(apis::homebrew::cask_handler))
    .route("/homebrew/{kind}/{name}", get(apis::homebrew::formula_handler))
    .route("/vscode/{kind}/{name}", get(apis::vscode::handler))
    .route("/amo/{kind}/{name}", get(apis::amo::handler))
    .route("/cws/{kind}/{name}", get(apis::cws::handler))
    .route("/jetbrains/{kind}/{name}", get(apis::jetbrains::handler))
    .route("/github/{kind}/{*name}", get(apis::github::handler))
    .route("/badge", get(apis::fixed::handler1))
    .route("/badge/{config}", get(apis::fixed::handler2))
    .route("/badge/{label}/{value}/{color}", get(apis::fixed::handler3))
    .route("/", get(pages::index))
    .route("/debug", get(pages::debug));

  server::run_server(app).await
}
