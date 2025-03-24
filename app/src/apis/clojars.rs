use axum::extract::{Path, Query};
use badgelib::{Badge, Period};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
  license: String,
  dlt: u64,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  // also: https://cljdoc.org/badge/wing/wing
  let url = format!("https://clojars.org/api/artifacts/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["latest_version"].as_str().unwrap_or("unknown").into();
  let license = dat["licenses"]
    .as_array()
    .and_then(|x| {
      let items = x.iter().filter_map(|x| x["name"].as_str()).collect::<Vec<_>>();
      if items.is_empty() { None } else { Some(items.join(" | ")) }
    })
    .unwrap_or("unknown".into());

  let dlt = dat["downloads"].as_u64().unwrap_or(0);

  Ok(Data { version, license, dlt })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "dt")]
  DlTotal,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data(name.clone()).await?;

  match kind {
    Kind::Version => {
      // https://github.com/badges/shields/pull/431
      Ok(badge.for_version("clojars", &rs.version).value(&format!(r#"[{name} "{}"]"#, rs.version)))
    }
    Kind::License => Ok(badge.for_license(&rs.license)),
    Kind::DlTotal => Ok(badge.for_downloads(Period::Total, rs.dlt)),
  }
}
