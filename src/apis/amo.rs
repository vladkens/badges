use axum::{
  extract::{Path, Query},
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{
  badge::{Badge, DlPeriod},
  server::{Dict, Rep, Res},
};

use super::get_client;

#[derive(Debug)]
struct Data {
  version: String,
  license: String,
  dlw: u64, // weekly downloads
}

async fn get_data(name: &str) -> Res<Data> {
  let url = format!("https://addons.mozilla.org/api/v4/addons/addon/{name}/");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["current_version"]["version"].as_str().unwrap_or("unknown").to_string();
  let license = dat["current_version"]["license"]["url"].as_str().unwrap_or("unknown").to_string();
  let license = license.split('/').last().unwrap_or(&license).to_string();
  let license = license.strip_suffix(".html").unwrap_or(&license).to_string();
  let dlw = dat["weekly_downloads"].as_u64().unwrap_or(0);

  Ok(Data { version, license, dlw })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "dw")]
  Weekly,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> Rep<impl IntoResponse> {
  let rs = get_data(&name).await?;
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "mozilla add-on", &rs.version)?),
    Kind::License => Ok(Badge::for_license(&qs, &rs.license)?),
    Kind::Weekly => Ok(Badge::for_dl(&qs, DlPeriod::Weekly, rs.dlw)?),
  }
}
