use axum::{
  extract::{Path, Query},
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{
  badge::{Badge, DlPeriod},
  colors::Color,
  server::{Dict, Rep, Res},
  utils::render_stars,
};

use super::get_client;

async fn get_version(name: &str) -> Res<String> {
  let url = format!("https://plugins.jetbrains.com/api/plugins/{name}/updates");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  Ok(dat[0]["version"].as_str().unwrap_or("unknown").to_string())
}

async fn get_dlt(name: &str) -> Res<u64> {
  let url = format!("https://plugins.jetbrains.com/api/plugins/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  Ok(dat["downloads"].as_u64().unwrap_or(0))
}

async fn get_score(name: &str) -> Res<f64> {
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
  Total,
  #[serde(rename = "score")]
  Score,
  #[serde(rename = "stars")]
  Stars,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> Rep<impl IntoResponse> {
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "jetbrain plugin", &get_version(&name).await?)?),
    Kind::Total => Ok(Badge::for_dl(&qs, DlPeriod::Total, get_dlt(&name).await?)?),
    Kind::Score => {
      Ok(Badge::new("score", &format!("{:.1}/5", get_score(&name).await?), Color::Default))
    }
    Kind::Stars => {
      Ok(Badge::new("stars", &render_stars(get_score(&name).await?, 5.0), Color::Default))
    }
  }
}
