use axum::{
  extract::{Path, Query},
  response::IntoResponse,
};
use cached::proc_macro::once;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, DlPeriod};
use crate::server::{Dict, Rep, Res};

#[derive(Debug, Clone)]
struct Base {
  license: String,
  stars: u64,
  forks: u64,
}

#[derive(Debug, Clone)]
struct Release {
  version: String,
  dlt: u64,
}

#[once(time = 60, sync_writes = true, result = true)]
async fn get_data(name: &str) -> Res<Base> {
  let url = format!("https://api.github.com/repos/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let license = dat["license"]["spdx_id"].as_str().unwrap_or("unknown").to_string();
  let stars = dat["stargazers_count"].as_u64().unwrap_or(0);
  let forks = dat["forks_count"].as_u64().unwrap_or(0);

  Ok(Base { license, stars, forks })
}

#[once(time = 60, sync_writes = true, result = true)]
async fn get_release(name: &str) -> Res<Release> {
  let url = format!("https://api.github.com/repos/{name}/releases/latest");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["tag_name"].as_str().unwrap_or("unknown").to_string();
  let dlt = dat["assets"]
    .as_array()
    .map(|p| p.iter().filter_map(|x| x["download_count"].as_u64()).sum::<u64>())
    .unwrap_or(0);

  Ok(Release { version, dlt })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "release")]
  Release,
  #[serde(rename = "assets-dl")]
  AssetsDl,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "stars")]
  Stars,
  #[serde(rename = "forks")]
  Forks,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> Rep<impl IntoResponse> {
  match kind {
    Kind::Release => Ok(Badge::for_version(&qs, "release", &get_release(&name).await?.version)?),
    Kind::AssetsDl => Ok(Badge::for_dl(&qs, DlPeriod::Total, get_release(&name).await?.dlt)?),
    Kind::License => Ok(Badge::for_license(&qs, &get_data(&name).await?.license)?),
    Kind::Stars => Ok(Badge::for_count(&qs, "stars", get_data(&name).await?.stars)?),
    Kind::Forks => Ok(Badge::for_count(&qs, "forks", get_data(&name).await?.forks)?),
  }
}
