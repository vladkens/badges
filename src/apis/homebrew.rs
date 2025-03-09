use axum::{
  extract::{Path, Query},
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, DlPeriod};
use crate::server::{Dict, Rep, Res};

#[derive(Debug)]
struct Data {
  version: String,
  dlm: u64,
  dly: u64,
}

async fn get_data(chan: &str, name: &str) -> Res<Data> {
  let url = format!("https://formulae.brew.sh/api/{chan}/{name}.json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let v1 = dat["versions"]["stable"].as_str();
  let v2 = dat["version"].as_str();
  let version = v1.or(v2).unwrap_or("unknown").to_string();
  let dlm = dat["analytics"]["install"]["30d"][name].as_u64().unwrap_or(0);
  let dly = dat["analytics"]["install"]["90d"][name].as_u64().unwrap_or(0);

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

async fn handler(qs: Dict, kind: Kind, chan: &str, name: String) -> Rep<impl IntoResponse> {
  let rs = get_data(chan, &name).await?;

  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "homebrew", &rs.version)?),
    Kind::Monthly => Ok(Badge::for_dl(&qs, DlPeriod::Monthly, rs.dlm)?),
    Kind::Yearly => Ok(Badge::for_dl(&qs, DlPeriod::Yearly, rs.dly)?),
  }
}

pub async fn formula_handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> Rep<impl IntoResponse> {
  handler(qs, kind, "formula", name).await
}

pub async fn cask_handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> Rep<impl IntoResponse> {
  handler(qs, kind, "cask", name).await
}
