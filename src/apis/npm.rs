use anyhow::anyhow;
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
struct NpmData {
  version: String,
  license: String,
}

async fn get_data(name: &str) -> Res<NpmData> {
  let url = format!("https://unpkg.com/{name}@latest/package.json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["version"].as_str().unwrap_or("unknown").to_string();
  let license = dat["license"].as_str().unwrap_or("unknown").to_string();
  Ok(NpmData { version, license })
}

async fn get_downloads(name: &str, kind: &Kind) -> Res<u64> {
  let url = "https://api.npmjs.org/downloads";
  let url = match kind {
    Kind::Weekly => format!("{url}/range/last-week/{name}"),
    Kind::Monthly => format!("{url}/range/last-month/{name}"),
    Kind::Total => format!("{url}/range/2005-01-01:2030-01-01/{name}"),
    _ => unreachable!(),
  };

  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dls = dat["downloads"].as_array().ok_or(anyhow!("no downloads"))?;
  let dls = dls.iter().filter_map(|x| x["downloads"].as_u64());
  let dls = dls.sum::<u64>();
  Ok(dls)
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
  // Add '@' if it's a scoped package
  let name = if name.contains("/") && !name.starts_with('@') { format!("@{}", name) } else { name };

  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "npm", &get_data(&name).await?.version)?),
    Kind::License => Ok(Badge::for_license(&qs, &get_data(&name).await?.license)?),
    Kind::Weekly => Ok(Badge::for_dl(&qs, DlPeriod::Weekly, get_downloads(&name, &kind).await?)?),
    Kind::Monthly => Ok(Badge::for_dl(&qs, DlPeriod::Monthly, get_downloads(&name, &kind).await?)?),
    Kind::Total => Ok(Badge::for_dl(&qs, DlPeriod::Total, get_downloads(&name, &kind).await?)?),
  }
}
