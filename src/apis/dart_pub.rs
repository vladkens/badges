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
struct Data {
  version: String,
}

#[derive(Debug)]
struct Score {
  dlm: u64,
  likes: u64,
  license: String,
}

async fn get_data(name: &str) -> Res<Data> {
  let url = format!("https://pub.dev/api/packages/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["latest"]["version"].as_str().unwrap_or("unknown").to_string();

  Ok(Data { version })
}

async fn get_score(name: &str) -> Res<Score> {
  let url = format!("https://pub.dev/api/packages/{name}/score");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dlm = dat["downloadCount30Days"].as_u64().unwrap_or(0);
  let likes = dat["likeCount"].as_u64().unwrap_or(0);

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

  let license = tags.iter().next().unwrap_or(&"unknown".to_string()).to_string();

  Ok(Score { dlm, likes, license })
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
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> Rep<impl IntoResponse> {
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "pub", &get_data(&name).await?.version)?),
    Kind::License => Ok(Badge::for_license(&qs, &get_score(&name).await?.license)?),
    Kind::Weekly => Ok(Badge::for_dl(&qs, DlPeriod::Weekly, get_score(&name).await?.dlm / 4)?),
    Kind::Monthly => Ok(Badge::for_dl(&qs, DlPeriod::Monthly, get_score(&name).await?.dlm)?),
  }
}
