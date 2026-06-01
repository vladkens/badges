use axum::extract::{Path, Query};
use badgelib::{Badge, Period};
use cached::macros::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
}

#[derive(Debug, Clone)]
struct Score {
  dlm: u64,
  license: String,
}

#[cached(ttl = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  let url = format!("https://pub.dev/api/packages/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["latest"]["version"].as_str().unwrap_or("unknown").to_string();

  Ok(Data { version })
}

#[cached(ttl = 60, result = true)]
async fn get_score(name: String) -> Res<Score> {
  let url = format!("https://pub.dev/api/packages/{name}/score");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dlm = dat["downloadCount30Days"].as_u64().unwrap_or(0);
  // let likes = dat["likeCount"].as_u64().unwrap_or(0);

  let default_vec = vec![];
  let tags = dat["tags"].as_array().unwrap_or(&default_vec);
  let tags = tags
    .iter()
    .filter_map(|x| x.as_str())
    .filter(|x| {
      x.starts_with("license:") && *x != "license:fsf-libre" && *x != "license:osi-approved"
    })
    .map(|x| x[8..].to_string())
    .collect::<Vec<String>>();

  let license = tags.first().unwrap_or(&"unknown".to_string()).to_string();

  Ok(Score { dlm, license })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "dw")]
  DlWeek,
  #[serde(rename = "dm")]
  DlMonth,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  match kind {
    Kind::Version => Ok(badge.for_version("pub", &get_data(name).await?.version)),
    Kind::License => Ok(badge.for_license(&get_score(name).await?.license)),
    Kind::DlWeek => Ok(badge.for_downloads(Period::Week, get_score(name).await?.dlm / 4)),
    Kind::DlMonth => Ok(badge.for_downloads(Period::Month, get_score(name).await?.dlm)),
  }
}
