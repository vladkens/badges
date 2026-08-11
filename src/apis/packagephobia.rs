use axum::extract::{Path, Query};
use badgelib::{Badge, Color};
use cached::macros::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::BadgeRep;

#[derive(Debug, Clone)]
struct Data {
  publish_pretty: String,
  publish_color: Color,
  install_pretty: String,
  install_color: Color,
}

#[cached(ttl = 3600)]
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
  let publish_color =
    dat["publish"]["color"].as_str().and_then(|x| Color::try_from(x).ok()).unwrap_or_default();

  let install_pretty = dat["install"]["pretty"].as_str().unwrap_or("unknown").to_string();
  let install_color =
    dat["install"]["color"].as_str().and_then(|x| Color::try_from(x).ok()).unwrap_or_default();

  Ok(Data { publish_pretty, publish_color, install_pretty, install_color })
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
    Kind::Publish => {
      Ok(badge.label("publish size").value(&rs.publish_pretty).value_color(rs.publish_color))
    }
    Kind::Install => {
      Ok(badge.label("install size").value(&rs.install_pretty).value_color(rs.install_color))
    }
  }
}
