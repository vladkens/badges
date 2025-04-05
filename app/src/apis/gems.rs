use anyhow::anyhow;
use axum::extract::{Path, Query};
use badgelib::{Badge, Period};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct GemData {
  version: String,
  license: String,
  dlt: u64, // total downloads
  ruby_ver: String,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<GemData> {
  // let url = format!("https://rubygems.org/api/v1/gems/{name}.json");
  let url = format!("https://rubygems.org/api/v1/versions/{name}.json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let vers = dat.as_array().ok_or(anyhow!("no data"))?;
  let stable = vers.iter().find(|x| !x["prerelease"].as_bool().unwrap_or(false));
  let latest = stable.or(vers.first()).ok_or(anyhow!("no version"))?;

  let version = latest["number"].as_str().unwrap_or("unknown").to_string();
  let license = latest["licenses"][0].as_str().unwrap_or("unknown").to_string();

  let dlt = vers.iter().map(|x| x["downloads_count"].as_u64().unwrap_or(0)).sum();
  let ruby_ver = latest["ruby_version"].as_str().unwrap_or("unknown").to_string();

  Ok(GemData { version, license, dlt, ruby_ver })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "dt")]
  DlTotal,
  #[serde(rename = "ruby")]
  Ruby,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data(name).await?;
  match kind {
    Kind::Version => Ok(badge.for_version("gem", &rs.version)),
    Kind::License => Ok(badge.for_license(&rs.license)),
    Kind::DlTotal => Ok(badge.for_downloads(Period::Total, rs.dlt)),
    Kind::Ruby => Ok(badge.label("ruby").value(&rs.ruby_ver)),
  }
}
