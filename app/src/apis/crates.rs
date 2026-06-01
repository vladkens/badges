use axum::extract::{Path, Query};
use badgelib::{Badge, Color, Period};
use cached::macros::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
  license: String,
  dlt: u64,
  dlq: u64,     // Downloads in 90 days
  msrv: String, // Minimum Supported Rust Version
  size: u64,
}

#[cached(ttl = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  let url = format!("https://crates.io/api/v1/crates/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let v1 = dat["crate"]["max_stable_version"].as_str();
  let v2 = dat["crate"]["max_version"].as_str();
  let version = v1.or(v2).unwrap_or("unknown").to_string();

  let dlt = dat["crate"]["downloads"].as_u64().unwrap_or(0);
  let dlq = dat["crate"]["recent_downloads"].as_u64().unwrap_or(0);

  let ver_data = dat["versions"]
    .as_array()
    .ok_or(anyhow::anyhow!("versions not found"))?
    .iter()
    .find(|x| x["num"].as_str() == Some(&version))
    .ok_or(anyhow::anyhow!("version not found"))?;
  let license = ver_data["license"].as_str().unwrap_or("unknown").to_string();
  let msrv = ver_data["rust_version"].as_str().unwrap_or("unknown").to_string();
  let size = ver_data["crate_size"].as_u64().unwrap_or(0);

  Ok(Data { version, license, dlt, dlq, msrv, size })
}

#[cached(ttl = 60, result = true)]
async fn get_docs(name: String) -> Res<bool> {
  let url = format!("https://docs.rs/crate/{name}/latest/status.json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  Ok(dat["doc_status"].as_bool().unwrap_or(false))
}

#[cached(ttl = 60, result = true)]
async fn get_deps_status(name: String) -> Res<(String, Color)> {
  let url = format!("https://deps.rs/crate/{name}/latest/status.svg?style=flat-square");
  let rep = get_client().get(&url).send().await?.error_for_status()?;

  // https://github.com/deps-rs/deps.rs/blob/9308a0cdbcd63fec2949124abc7f82d3fc5a33f6/src/server/views/badge.rs#L12
  let dat = rep.text().await?;
  let msg = dat
    .split("dependencies")
    .last()
    .and_then(|x| x.split(r#"">"#).last())
    .and_then(|x| x.split("</text>").next())
    .unwrap_or("unknown");

  match msg {
    "none" => Ok(("none".to_string(), Color::Green)),
    "up to date" => Ok(("up to date".to_string(), Color::Green)),
    "maybe insecure" => Ok(("maybe insecure".to_string(), Color::Lime)),
    "insecure" => Ok(("maybe insecure".to_string(), Color::Red)),
    x if x.contains("outdated") => Ok((x.to_string(), Color::Yellow)),
    _ => Ok(("unknown".to_string(), Color::Gray)),
  }
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display, PartialEq)]
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
  #[serde(rename = "msrv")]
  Msrv,
  #[serde(rename = "docs")]
  Docs,
  #[serde(rename = "size")]
  Size,
  #[serde(rename = "deps")]
  Deps,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  if kind == Kind::Docs {
    let status = get_docs(name).await?;
    return Ok(badge.for_ci_status("docs", status));
  }

  if kind == Kind::Deps {
    let (value, color) = get_deps_status(name).await?;
    return Ok(badge.label("dependencies").value(&value).value_color(color));
  }

  let rs = get_data(name).await?;
  match kind {
    Kind::Docs | Kind::Deps => unreachable!(),
    Kind::Version => Ok(badge.for_version("crates.io", &rs.version)),
    Kind::License => Ok(badge.for_license(&rs.license)),
    Kind::DlTotal => Ok(badge.for_downloads(Period::Total, rs.dlt)),
    Kind::DlWeek => Ok(badge.for_downloads(Period::Week, rs.dlq / 12)), // 12 weeks in 90 days
    Kind::DlMonth => Ok(badge.for_downloads(Period::Month, rs.dlq / 3)), // 3 months in 90 days
    Kind::Msrv => Ok(badge.for_version("msrv", &rs.msrv)),
    Kind::Size => Ok(badge.for_size("size", rs.size)),
  }
}
