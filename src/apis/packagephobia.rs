use axum::extract::{Path, Query};
use badgelib::{Badge, Color};
use cached::macros::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::BadgeRep;

#[derive(Debug, Clone)]
struct Data {
  publish_pretty: String,
  publish_bytes: u64,
  install_pretty: String,
  install_bytes: u64,
}

fn size_color(bytes: u64) -> Color {
  match bytes {
    ..102_400 => Color::Green,                // < 100 KiB
    102_400..1_048_576 => Color::Lime,        // < 1 MiB
    1_048_576..5_242_880 => Color::Blue,      // < 5 MiB
    5_242_880..31_457_280 => Color::Yellow,   // < 30 MiB
    31_457_280..104_857_600 => Color::Orange, // < 100 MiB
    104_857_600..524_288_000 => Color::Red,   // < 500 MiB
    _ => Color::Hex("ec4899".into()),         // >= 500 MiB
  }
}

#[cached(ttl_secs = 3600)]
async fn get_data(name: String) -> anyhow::Result<Data> {
  let url = format!("https://packagephobia.com/v2/api.json?p={name}");
  let rep = get_client()
    .get(&url)
    .header("user-agent", "Mozilla/5.0 (compatible; Badges/1.0; +https://badges.ws)")
    .send()
    .await?
    .error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let publish_pretty = dat["publish"]["pretty"].as_str().unwrap_or("unknown").to_string();
  let publish_bytes = dat["publish"]["bytes"].as_u64().unwrap_or_default();

  let install_pretty = dat["install"]["pretty"].as_str().unwrap_or("unknown").to_string();
  let install_bytes = dat["install"]["bytes"].as_u64().unwrap_or_default();

  Ok(Data { publish_pretty, publish_bytes, install_pretty, install_bytes })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "publish")]
  Publish,
  #[serde(rename = "install")]
  Install,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  // Add '@' if it's a scoped package
  let name = if name.contains("/") && !name.starts_with('@') { format!("@{}", name) } else { name };
  let rs = get_data(name).await?;

  match kind {
    Kind::Publish => Ok(
      badge
        .label("publish size")
        .value(&rs.publish_pretty)
        .value_color(size_color(rs.publish_bytes)),
    ),
    Kind::Install => Ok(
      badge
        .label("install size")
        .value(&rs.install_pretty)
        .value_color(size_color(rs.install_bytes)),
    ),
  }
}
