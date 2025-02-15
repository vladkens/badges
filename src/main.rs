use axum::{response::IntoResponse, routing::get};
use colors::Color;
use maud::{html, Markup};
use serde_variant::to_variant_name;
use server::Rep;
use strum::IntoEnumIterator;
use tracing_subscriber::layer::SubscriberExt;

mod apis;
mod badge;
mod colors;
mod icons;
mod server;
mod utils;
mod width;

fn base(title: &str, node: Markup) -> Markup {
  html!(html {
    head {
      meta charset="utf-8" {}
      meta name="viewport" content="width=device-width, initial-scale=1" {}
      title { (title) }

      link rel="preconnect" href="https://unpkg.com" {}
      link rel="stylesheet" href="https://unpkg.com/normalize.css" media="screen" {}
      link rel="stylesheet" href="https://unpkg.com/sakura.css/css/sakura.css" media="screen" {}
      link rel="stylesheet" href="https://unpkg.com/sakura.css/css/sakura-dark.css" media="screen and (prefers-color-scheme: dark)" {}
      script src="https://unpkg.com/@twind/cdn" {}
    }
    body style="min-width: 900px; padding-bottom: 30px;" { (node) }
  })
}

fn render_tbox<T: Into<String>>(name: &str, items: Vec<(T, T)>) -> maud::Markup {
  let items: Vec<(String, String)> = items.into_iter().map(|(a, b)| (a.into(), b.into())).collect();
  let anchor = name.to_lowercase().replace(" ", "-").replace(".", "");

  html!({
    tbody style="border-top: 1px solid #ccc" id=(anchor) {
      tr { th colspan="3" { a href=(format!("#{anchor}")) { (name) } } }
      @for (path, desc) in items {
        tr {
          td { (desc) }
          td { code { (path) } }
          td style="min-width: 220px" {
            img style="margin-bottom: 0; height: 20px" src=(path) alt=(desc) {}
          }
        }
      }
    }
  })
}

fn render_enum<T: IntoEnumIterator + std::fmt::Display + serde::Serialize>(
  name: &str,
  path: &str,
) -> maud::Markup {
  let items: Vec<(String, String)> = T::iter()
    .map(|x| {
      let path = path.replace("{}", to_variant_name(&x).unwrap());
      // let desc = format!("{} {}", name, x);
      let desc = x.to_string();
      (path, desc)
    })
    .collect();

  render_tbox(name, items)
}

async fn index() -> Rep<impl IntoResponse> {
  let colors = Color::iter().filter_map(|x| x.to_name());

  #[rustfmt::skip]
  let icons = vec![
    "git", "github", "gitlab", "gitea", "bitbucket", "githubsponsors",
    "circleci", "travisci", "appveyor", "jenkins", "drone", "codecov", "coveralls",
    "bitcoin", "ethereum", "litecoin", "dogecoin", "monero", "ripple", "tether",
    "buymeacoffee", "patreon", "paypal", "liberapay", "opencollective", "kofi",
    "discord", "slack", "telegram", "whatsapp", "signal", "messenger", "line",
    "reddit", "x", "medium", "devto", "hashnode", "ghost", "rss",
    "docker", "kubernetes", "helm", "ansible", "terraform", "vagrant", "puppet",
    "rust", "python", "ruby", "php", "llvm", "javascript", "typescript", "go",
  ];

  let options = vec![
    ("label", "Label text"),
    ("label_color", "Label color"),
    ("value", "Value text"),
    ("value_color", "Value color"),
    ("icon", "Icon name"),
    ("icon_color", "Icon color"),
  ];

  let static_examples = vec![
    ("/badge/label-message-ff0000", "Fixed badge"),
    ("/badge/label--message-f00", "Fixed badge with dash"),
    ("/badge/label__message-red", "Fixed badge with underscore"),
  ];

  let html = html!({
    h1 class="text-center" { a href="/" { { "badges" span class="text-rose-600" { ".ws" } "!" } } }
    p class="text-center" { "A simple badge generator." }

    h5 { "Integrations" }
    table {
      thead {
        tr {
          th { "Description" }
          th { "Path" }
          th { "Badge" }
        }
      }

      (render_tbox("Static", static_examples))
      (render_enum::<apis::pypi::Kind>("PyPI", "/pypi/{}/twscrape"))
      (render_enum::<apis::npm::Kind>("npm", "/npm/{}/apigen-ts"))
      (render_enum::<apis::packagephobia::Kind>("packagephobia", "/packagephobia/{}/apigen-ts"))
      (render_enum::<apis::crates::Kind>("crates.io", "/crates/{}/tokio"))
      (render_enum::<apis::packagist::Kind>("Packagist", "/packagist/{}/laravel/laravel"))
      (render_enum::<apis::gems::Kind>("Ruby Gems", "/gem/{}/rails"))
      (render_enum::<apis::dart_pub::Kind>("Dart Pub", "/pub/{}/dio"))
      (render_enum::<apis::hackage::Kind>("Hackage", "/hackage/{}/servant"))
      (render_enum::<apis::nuget::Kind>("NuGet", "/nuget/{}/Newtonsoft.Json"))
      (render_enum::<apis::homebrew::Kind>("Homebrew", "/homebrew/{}/macmon"))
      (render_enum::<apis::homebrew::Kind>("Homebrew Cask", "/homebrew/{}/cask/firefox"))
      (render_enum::<apis::vscode::Kind>("VS Code", "/vscode/{}/esbenp.prettier-vscode"))
      (render_enum::<apis::amo::Kind>("Mozilla Add-ons", "/amo/{}/privacy-badger17"))
      (render_enum::<apis::cws::Kind>("Chrome Web Store", "/cws/{}/epcnnfbjfcgphgdmggkamkmgojdagdnn"))
      (render_enum::<apis::jetbrains::Kind>("JetBrains Plugin", "/jetbrains/{}/22282"))
      (render_enum::<apis::github::Kind>("GitHub", "/github/{}/vladkens/macmon"))
    }

    div class="flex flex-col gap-2.5" {
      h5 class="mb-0" { "Colors" }
      div { "You can use any of the following color names or hex values with " code { "?color=value" } }
      div class="flex flex-row gap-2" {
        @for color in colors {
          img style="margin-bottom: 0; height: 20px" src=(format!("/badge/?value={color}&color={color}&label=color")) alt=(color) {}
        }
      }
    }

    div class="flex flex-col gap-2.5" {
      h5 class="mb-0" { "Icons" }
      div { "You can add icons to your badge with " code { "?icon=name&iconColor=hex" } }
      div class="flex flex-row gap-2 flex-wrap" {
        @for icon in icons {
          img style="margin-bottom: 0; height: 20px" src=(format!("/badge/?icon={icon}&value={icon}")) alt=(icon) {}
        }
      }
      div { "All icons names can be found " a href="https://simpleicons.org/" target="_blank" { "here" } "." }
    }

    div class="flex flex-col gap-2.5" {
      h5 class="mb-0" { "Options" }
      ol {
        @for (name, desc) in options {
          li { code { (name) } { " - " (desc) } }
        }
      }
    }
  });

  Ok(base("badges.ws", html))
}

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

  let app = axum::Router::new() //
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
    .route("/", get(index));

  Ok(server::run_server(app).await?)
}
