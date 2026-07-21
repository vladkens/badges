use axum::extract::{Path, Query};
use badgelib::{Badge, Period};
use cached::macros::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::BadgeRep;

#[cached(ttl = 60)]
async fn get_version(name: String) -> anyhow::Result<String> {
  let url = format!("https://plugins.jetbrains.com/api/plugins/{name}/updates");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  Ok(dat[0]["version"].as_str().unwrap_or("unknown").to_string())
}

#[cached(ttl = 60)]
async fn get_dlt(name: String) -> anyhow::Result<u64> {
  let url = format!("https://plugins.jetbrains.com/api/plugins/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  Ok(dat["downloads"].as_u64().unwrap_or(0))
}

#[cached(ttl = 60)]
async fn get_score(name: String) -> anyhow::Result<f64> {
  let url = format!("https://plugins.jetbrains.com/api/plugins/{name}/rating");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  Ok(dat["meanRating"].as_f64().unwrap_or(0.0))
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "dt")]
  DlTotal,
  #[serde(rename = "score")]
  Score, // todo: rating?
  #[serde(rename = "stars")]
  Stars,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  match kind {
    Kind::Version => Ok(badge.for_version("jetbrain plugin", &get_version(name).await?)),
    Kind::DlTotal => Ok(badge.for_downloads(Period::Total, get_dlt(name).await?)),
    Kind::Score => Ok(badge.for_rating("rating", get_score(name).await?, 5.0)),
    Kind::Stars => Ok(badge.for_stars("stars", get_score(name).await?, 5.0)),
  }
}
