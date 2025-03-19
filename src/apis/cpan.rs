use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::Badge;
use crate::server::{BadgeRep, Dict, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
  license: String,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  let url = format!("https://fastapi.metacpan.org/v1/release/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["version"].as_str().unwrap_or("unknown").to_string();
  let license = dat["license"][0].as_str().unwrap_or("unknown").to_string();

  Ok(Data { version, license })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display, PartialEq)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
}

pub async fn handler(Path((kind, name)): Path<(Kind, String)>, Query(qs): Query<Dict>) -> BadgeRep {
  let name = name.replace("/", "-");
  let rs = get_data(name).await?;
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "cpan", &rs.version)?),
    Kind::License => Ok(Badge::for_license(&qs, &rs.license)?),
  }
}
