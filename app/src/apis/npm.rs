use anyhow::anyhow;
use axum::extract::{Path, Query};
use badgelib::{Badge, Period};
use cached::macros::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct NpmData {
  version: String,
  license: String,
}

#[cached(ttl = 60, result = true)]
async fn get_data(name: String) -> Res<NpmData> {
  let url = format!("https://unpkg.com/{name}@latest/package.json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["version"].as_str().unwrap_or("unknown").to_string();
  let license = dat["license"].as_str().unwrap_or("unknown").to_string();
  Ok(NpmData { version, license })
}

#[cached(ttl = 60, result = true)]
async fn get_downloads(name: String, kind: Kind) -> Res<u64> {
  let url = "https://api.npmjs.org/downloads";
  let url = match kind {
    Kind::DlWeek => format!("{url}/range/last-week/{name}"),
    Kind::DlMonth => format!("{url}/range/last-month/{name}"),
    Kind::DlTotal => format!("{url}/range/2005-01-01:2030-01-01/{name}"),
    _ => unreachable!(),
  };

  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dls = dat["downloads"].as_array().ok_or(anyhow!("no downloads"))?;
  let dls = dls.iter().filter_map(|x| x["downloads"].as_u64());
  let dls = dls.sum::<u64>();
  Ok(dls)
}

#[derive(
  Debug, Deserialize, Serialize, strum::EnumIter, strum::Display, Hash, Clone, PartialEq, Eq,
)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "dw")]
  DlWeek,
  #[serde(rename = "dm")]
  DlMonth,
  #[serde(rename = "dt")]
  DlTotal,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  // Add '@' if it's a scoped package
  let name = if name.contains("/") && !name.starts_with('@') { format!("@{}", name) } else { name };

  match kind {
    Kind::Version => Ok(badge.for_version("npm", &get_data(name).await?.version)),
    Kind::License => Ok(badge.for_license(&get_data(name).await?.license)),
    Kind::DlWeek => Ok(badge.for_downloads(Period::Week, get_downloads(name, kind).await?)),
    Kind::DlMonth => Ok(badge.for_downloads(Period::Month, get_downloads(name, kind).await?)),
    Kind::DlTotal => Ok(badge.for_downloads(Period::Total, get_downloads(name, kind).await?)),
  }
}
