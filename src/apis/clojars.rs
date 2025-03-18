use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, Color, DlPeriod};
use crate::server::BadgeRep;
use crate::server::{Dict, Res};

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

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display, PartialEq)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "dt")]
  Total,
}

pub async fn handler(Path((kind, name)): Path<(Kind, String)>, Query(qs): Query<Dict>) -> BadgeRep {
  let rs = get_data(name.clone()).await?;

  match kind {
    Kind::Version => {
      // https://github.com/badges/shields/pull/431
      let value = &format!(r#"[{name} "{}"]"#, rs.version);
      let color = Color::from_version(&rs.version);
      Ok(Badge::from_qs_with(&qs, "clojars", value, color)?)
    }
    Kind::License => Ok(Badge::for_license(&qs, &rs.license)?),
    Kind::Total => Ok(Badge::for_dl(&qs, DlPeriod::Total, rs.dlt)?),
  }
}
