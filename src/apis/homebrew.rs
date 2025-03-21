use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, DlPeriod};
use crate::server::{BadgeRep, Dict, Res};

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

async fn handler(qs: Dict, kind: Kind, chan: String, name: String) -> BadgeRep {
  let rs = get_data(chan, name).await?;

  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "homebrew", &rs.version)?),
    Kind::Monthly => Ok(Badge::for_dl(&qs, DlPeriod::Monthly, rs.dlm)?),
    Kind::Yearly => Ok(Badge::for_dl(&qs, DlPeriod::Yearly, rs.dly)?),
  }
}

pub async fn formula_handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> BadgeRep {
  handler(qs, kind, "formula".into(), name).await
}

pub async fn cask_handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> BadgeRep {
  handler(qs, kind, "cask".into(), name).await
}
