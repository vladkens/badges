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
  let url = format!("https://hackage.haskell.org/package/{name}/{name}.cabal");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.text().await?;

  let version = dat
    .lines()
    .find(|x| x.starts_with("version:"))
    .map(|x| x.split(':').nth(1).unwrap_or("unknown").trim())
    .unwrap_or("unknown")
    .to_string();

  let license = dat
    .lines()
    .find(|x| x.starts_with("license:"))
    .map(|x| x.split(':').nth(1).unwrap_or("unknown").trim())
    .unwrap_or("unknown")
    .to_string();

  Ok(Data { version, license })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
}

pub async fn handler(Path((kind, name)): Path<(Kind, String)>, Query(qs): Query<Dict>) -> BadgeRep {
  let rs = get_data(name).await?;
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "hackage", &rs.version)?),
    Kind::License => Ok(Badge::for_license(&qs, &rs.license)?),
  }
}
