use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::get_client;
use crate::badgelib::{Badge, Color, utils::millify};
use crate::server::{BadgeRep, Dict, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
  installs: u64,
  downloads: u64,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  let url = "https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery";
  let dat = json!({
    "filters": [{ "criteria": [{ "filterType": 7, "value": name }] }],
    "flags": 914
  });

  let rep = get_client().post(url).query(&[("api-version", "7.2-preview.1")]).json(&dat);
  let rep = rep.send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  let dat = &dat["results"][0]["extensions"][0];

  let get_stat = |name: &str| -> u64 {
    let val = dat["statistics"].as_array();
    let val = val.and_then(|arr| arr.iter().find(|x| x["statisticName"] == name));
    let val = val.and_then(|x| x["value"].as_f64());
    val.map(|x| x as u64).unwrap_or(0)
  };

  let installs = get_stat("install");
  let downloads = get_stat("updateCount") + installs;
  let version = dat["versions"][0]["version"].as_str().unwrap().to_string();

  Ok(Data { version, installs, downloads })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "i")]
  Installs,
  #[serde(rename = "d")]
  Downloads,
}

pub async fn handler(Path((kind, name)): Path<(Kind, String)>, Query(qs): Query<Dict>) -> BadgeRep {
  let rs = get_data(name).await?;
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "vscode", &rs.version)?),
    Kind::Installs => Ok(Badge::new("installs", &millify(rs.installs), Color::Green)),
    Kind::Downloads => Ok(Badge::new("downloads", &millify(rs.downloads), Color::Green)),
  }
}
