use axum::extract::{Path, Query};
use badgelib::{Badge, Period};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
  dlm: u64,
  dly: u64,
}

#[cached(time = 60, result = true)]
async fn get_data(chan: String, name: String) -> Res<Data> {
  let url = format!("https://formulae.brew.sh/api/{chan}/{name}.json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let v1 = dat["versions"]["stable"].as_str();
  let v2 = dat["version"].as_str();
  let version = v1.or(v2).unwrap_or("unknown").to_string();
  let dlm = dat["analytics"]["install"]["30d"][&name].as_u64().unwrap_or(0);
  let dly = dat["analytics"]["install"]["90d"][&name].as_u64().unwrap_or(0);

  Ok(Data { version, dlm, dly })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "dm")]
  Monthly,
  #[serde(rename = "dy")]
  Yearly,
}

pub async fn formula_handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data("formula".into(), name).await?;
  match kind {
    Kind::Version => Ok(badge.for_version("homebrew", &rs.version)),
    Kind::Monthly => Ok(badge.for_downloads(Period::Month, rs.dlm)),
    Kind::Yearly => Ok(badge.for_downloads(Period::Year, rs.dly)),
  }
}

pub async fn cask_handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data("cask".into(), name).await?;
  match kind {
    Kind::Version => Ok(badge.for_version("homebrew", &rs.version)),
    Kind::Monthly => Ok(badge.for_downloads(Period::Month, rs.dlm)),
    Kind::Yearly => Ok(badge.for_downloads(Period::Year, rs.dly)),
  }
}
