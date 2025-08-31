use std::time::Duration;

use axum::extract::{Path, Query};
use badgelib::Badge;
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

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

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data(name.replace("/", "-")).await?;
  match kind {
    Kind::Version => Ok(badge.for_version("cpan", &rs.version)),
    Kind::License => Ok(badge.for_license(&rs.license)),
  }
}
