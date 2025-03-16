use std::str;

use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, DlPeriod};
use crate::server::{BadgeRep, Dict, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
  dlt: u64, // total downloads
}

#[cached(time = 60, result = true)]
async fn get_dl(name: String) -> Res<Data> {
  let name = name.to_lowercase();
  let url = format!("https://azuresearch-usnc.nuget.org/query?q=packageid:{name}&prerelease=true");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  let dat = &dat["data"][0];

  let version = dat["version"].as_str().unwrap_or("unknown").to_string();
  let dlt = dat["totalDownloads"].as_u64().unwrap_or(0);

  Ok(Data { version, dlt })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "dt")]
  Total,
}

pub async fn handler(Path((kind, name)): Path<(Kind, String)>, Query(qs): Query<Dict>) -> BadgeRep {
  let rs = get_dl(name).await?;
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "nuget", &rs.version)?),
    Kind::Total => Ok(Badge::for_dl(&qs, DlPeriod::Total, rs.dlt)?),
  }
}
