use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, Color};
use crate::server::BadgeRep;
use crate::server::{Dict, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
  license: String,
  platforms: Vec<String>,
  runtime: String,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  // also: https://metrics.cocoapods.org/api/v1/pods/SwiftyJSON
  let url = format!("https://trunk.cocoapods.org/api/v1/pods/{name}/specs/latest");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["version"].as_str().unwrap_or("unknown").to_string();
  let license = dat["license"]["type"].as_str().unwrap_or("unknown").to_string();
  let platforms = dat["platforms"].as_object().unwrap().keys().map(|x| x.to_string()).collect();
  let runtime = dat["swift_version"].as_str().unwrap_or("unknown").to_string();

  Ok(Data { version, license, platforms, runtime })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display, PartialEq)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "p", alias = "platforms")]
  Platform,
}

pub async fn handler(Path((kind, name)): Path<(Kind, String)>, Query(qs): Query<Dict>) -> BadgeRep {
  let rs = get_data(name).await?;
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "pod", &rs.version)?),
    Kind::License => Ok(Badge::for_license(&qs, &rs.license)?),
    Kind::Platform => {
      Ok(Badge::from_qs_with(&qs, "platform", &rs.platforms.join(" | "), Color::DefaultValue)?)
    }
  }
}
