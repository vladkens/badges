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
struct CrateData {
  version: String,
  license: String,
  dlt: u64, // total downloads
  dlq: u64, // quarterly downloads
}

async fn get_data(name: &str) -> Res<CrateData> {
  let url = format!("https://crates.io/api/v1/crates/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let v1 = dat["crate"]["max_stable_version"].as_str();
  let v2 = dat["crate"]["max_version"].as_str();
  let version = v1.or(v2).unwrap_or("unknown").to_string();

  let dlt = dat["crate"]["downloads"].as_u64().unwrap_or(0);
  let dlq = dat["crate"]["recent_downloads"].as_u64().unwrap_or(0);

  let ver_data = dat["versions"]
    .as_array()
    .ok_or(anyhow::anyhow!("versions not found"))?
    .iter()
    .find(|x| x["num"].as_str() == Some(&version))
    .ok_or(anyhow::anyhow!("version not found"))?;
  let license = ver_data["license"].as_str().unwrap_or("unknown").to_string();

  Ok(CrateData { version, license, dlt, dlq })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "dw")]
  Weekly,
  #[serde(rename = "dm")]
  Monthly,
  #[serde(rename = "dt")]
  Total,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> Rep<impl IntoResponse> {
  let rs = get_data(&name).await?;
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "crates.io", &rs.version)?),
    Kind::License => Ok(Badge::for_license(&qs, &rs.license)?),
    Kind::Total => Ok(Badge::for_dl(&qs, DlPeriod::Total, rs.dlt)?), // 12 weeks in 90 days
    Kind::Weekly => Ok(Badge::for_dl(&qs, DlPeriod::Weekly, rs.dlq / 12)?), // 12 weeks in 90 days
    Kind::Monthly => Ok(Badge::for_dl(&qs, DlPeriod::Monthly, rs.dlq / 3)?), // 3 months in 90 days
  }
}
