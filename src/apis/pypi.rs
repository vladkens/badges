use anyhow::anyhow;
use axum::{
  extract::{Path, Query},
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, Color, DlPeriod, utils::to_ver_label};
use crate::server::{Dict, Rep, Res};

#[derive(Debug)]
struct PyPiData {
  version: String,
  license: String,
  pythons: Vec<String>,
}

async fn get_data(name: &str) -> Res<PyPiData> {
  let url = format!("https://pypi.org/pypi/{name}/json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  let dat = dat.get("info").expect("info not found");

  let version = dat["version"].as_str().unwrap_or("unknown").to_string();
  let license = dat["license"].as_str().unwrap_or("unknown").to_string();
  let pythons = dat["classifiers"]
    .as_array()
    .unwrap_or(&vec![])
    .iter()
    .filter(|v| v.as_str().unwrap_or("").starts_with("Programming Language :: Python :: "))
    .map(|v| v.as_str().unwrap_or("").replace("Programming Language :: Python :: ", ""))
    .collect::<Vec<String>>();

  Ok(PyPiData { version, license, pythons })
}

async fn get_dl_granular(name: &str) -> Res<(u64, u64)> {
  // doc: https://pypistats.org/api/
  let url = format!("https://pypistats.org/api/packages/{}/recent", name);
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dlw = dat["data"]["last_week"].as_u64().unwrap_or(0);
  let dlm = dat["data"]["last_month"].as_u64().unwrap_or(0);
  Ok((dlw, dlm))
}

async fn get_dl_total(name: &str) -> Res<u64> {
  let url = format!("https://pypistats.org/api/packages/{}/overall?mirrors=true", name);
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dlt = dat["data"].as_array().ok_or(anyhow!("no data"))?;
  let dlt = dlt.iter().filter_map(|x| x["downloads"].as_u64());
  let dlt = dlt.sum::<u64>();
  Ok(dlt)
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
  #[serde(rename = "python")]
  Python,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> Rep<impl IntoResponse> {
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "pypi", &get_data(&name).await?.version)?),
    Kind::License => Ok(Badge::for_license(&qs, &get_data(&name).await?.license)?),
    Kind::Weekly => Ok(Badge::for_dl(&qs, DlPeriod::Weekly, get_dl_granular(&name).await?.0)?),
    Kind::Monthly => Ok(Badge::for_dl(&qs, DlPeriod::Monthly, get_dl_granular(&name).await?.1)?),
    Kind::Total => Ok(Badge::for_dl(&qs, DlPeriod::Total, get_dl_total(&name).await?)?),

    Kind::Python => {
      let rs = get_data(&name).await?;
      Ok(Badge::new("python", &to_ver_label(rs.pythons), Color::Blue))
    }
  }
}
