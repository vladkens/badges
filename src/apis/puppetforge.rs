use axum::extract::{Path, Query};
use badgelib::{Badge, Period};
use cached::macros::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::BadgeRep;

#[derive(Debug, Clone)]
struct Data {
  ver: String,
  dlt: u64,
  score: u64,
}

#[cached(ttl = 60)]
async fn get_data(name: String) -> anyhow::Result<Data> {
  let url = format!("https://forgeapi.puppetlabs.com/v3/modules/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let ver = dat["current_release"]["version"].as_str().unwrap_or("unknown").into();
  let dlt = dat["downloads"].as_u64().unwrap_or(0);
  let score = dat["current_release"]["validation_score"].as_u64().unwrap_or_default();

  Ok(Data { ver, dlt, score })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "version", alias = "v")]
  Version,
  #[serde(rename = "dt")]
  DlTotal,
  #[serde(rename = "score", alias = "f")]
  Score,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data(name.replace("/", "-")).await?;
  match kind {
    Kind::Version => Ok(badge.for_version("puppetforge", &rs.ver)),
    Kind::DlTotal => Ok(badge.for_downloads(Period::Total, rs.dlt)),
    Kind::Score => Ok(badge.label("quality score").value(&format!("{}%", rs.score))),
  }
}
