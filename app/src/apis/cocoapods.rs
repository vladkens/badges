use axum::extract::{Path, Query};
use badgelib::Badge;
use cached::macros::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
  license: String,
  platforms: Vec<String>,
}

#[cached(ttl = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  // also: https://metrics.cocoapods.org/api/v1/pods/SwiftyJSON
  let url = format!("https://trunk.cocoapods.org/api/v1/pods/{name}/specs/latest");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["version"].as_str().unwrap_or("unknown").to_string();
  let license = dat["license"]["type"].as_str().unwrap_or("unknown").to_string();
  let platforms = dat["platforms"].as_object().unwrap().keys().map(|x| x.to_string()).collect();
  // let runtime = dat["swift_version"].as_str().unwrap_or("unknown").to_string();

  Ok(Data { version, license, platforms })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "p", alias = "platforms")]
  Platform,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data(name).await?;
  match kind {
    Kind::Version => Ok(badge.for_version("pod", &rs.version)),
    Kind::License => Ok(badge.for_license(&rs.license)),
    Kind::Platform => Ok(badge.label("platform").value(&rs.platforms.join(" | "))),
  }
}
