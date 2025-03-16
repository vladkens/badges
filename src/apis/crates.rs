use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, Color, DlPeriod};
use crate::server::{BadgeRep, Dict, Res};

#[derive(Debug, Clone)]
struct CrateData {
  version: String,
  license: String,
  dlt: u64,     // total downloads
  dlq: u64,     // quarterly downloads
  msrv: String, // minimum supported rust version
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<CrateData> {
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
  let msrv = ver_data["rust_version"].as_str().unwrap_or("unknown").to_string();

  Ok(CrateData { version, license, dlt, dlq, msrv })
}

#[cached(time = 60, result = true)]
async fn get_docs(name: String) -> Res<bool> {
  let url = format!("https://docs.rs/crate/{name}/latest/status.json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  Ok(dat["doc_status"].as_bool().unwrap_or(false))
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
  #[serde(rename = "msrv")]
  Msrv,
  #[serde(rename = "docs")]
  Docs,
}

pub async fn handler(Path((kind, name)): Path<(Kind, String)>, Query(qs): Query<Dict>) -> BadgeRep {
  if kind == Kind::Docs {
    let status = get_docs(name).await?;
    let value = if status { "passing" } else { "failing" };
    let color = if status { Color::Green } else { Color::Red };
    return Ok(Badge::from_qs_with(&qs, "docs", value, color)?);
  }

  let rs = get_data(name).await?;
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "crates.io", &rs.version)?),
    Kind::License => Ok(Badge::for_license(&qs, &rs.license)?),
    Kind::Total => Ok(Badge::for_dl(&qs, DlPeriod::Total, rs.dlt)?), // 12 weeks in 90 days
    Kind::Weekly => Ok(Badge::for_dl(&qs, DlPeriod::Weekly, rs.dlq / 12)?), // 12 weeks in 90 days
    Kind::Monthly => Ok(Badge::for_dl(&qs, DlPeriod::Monthly, rs.dlq / 3)?), // 3 months in 90 days
    Kind::Msrv => Ok(Badge::for_version(&qs, "msrv", &rs.msrv)?),
    Kind::Docs => unreachable!(),
  }
}
