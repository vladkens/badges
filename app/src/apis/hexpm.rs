use axum::extract::{Path, Query};
use badgelib::{Badge, Period};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
  license: String,
  dlt: u64,
  dlw: u64,
  dlm: u64,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  let url = format!("https://hex.pm/api/packages/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["latest_stable_version"]
    .as_str()
    .or_else(|| dat["latest_version"].as_str())
    .unwrap_or("unknown")
    .to_string();

  let license = dat["meta"]["licenses"]
    .as_array()
    .map(|arr| {
      arr.iter().map(|lic| lic.as_str().unwrap_or("unknown")).collect::<Vec<_>>().join(" | ")
    })
    .unwrap_or("unknown".to_string());

  let dlt = dat["downloads"]["all"].as_u64().unwrap_or(0);
  let dlw = dat["downloads"]["week"].as_u64().unwrap_or(0);
  let dlm = (dlw as f64 / 7.0 * 30.4375) as u64;

  Ok(Data { version, license, dlt, dlw, dlm })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display, PartialEq)]
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
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data(name).await?;
  match kind {
    Kind::Version => Ok(badge.for_version("hex", &rs.version)),
    Kind::License => Ok(badge.for_license(&rs.license)),
    Kind::Weekly => Ok(badge.for_downloads(Period::Week, rs.dlw)),
    Kind::Monthly => Ok(badge.for_downloads(Period::Month, rs.dlm)),
    Kind::Total => Ok(badge.for_downloads(Period::Total, rs.dlt)),
  }
}
