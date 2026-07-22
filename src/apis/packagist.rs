use anyhow::anyhow;
use axum::extract::{Path, Query};
use badgelib::{Badge, Period};
use cached::macros::cached;
use semver::Version;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::BadgeRep;

#[derive(Debug, Clone)]
struct Data {
  version: String,
  license: String,
  dlw: u64,
  dlm: u64,
  dlt: u64,
  php_ver: String,
  stars: u64,
  dependents: u64,
}

fn parse_versions(
  obj: &serde_json::Map<String, serde_json::Value>,
) -> Vec<(Version, &serde_json::Value)> {
  let obj = obj.iter().filter_map(|(k, val)| {
    let ver = k.strip_prefix("v").unwrap_or(k);
    let ver = Version::parse(ver).ok();
    ver.map(|ver| (ver, val))
  });
  obj.collect::<Vec<_>>()
}

#[cached(ttl = 60)]
async fn get_data(name: String) -> anyhow::Result<Data> {
  let url = format!("https://packagist.org/packages/{name}.json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  let dat = dat.get("package").ok_or(anyhow!("package not found"))?;

  let dlt = dat["downloads"]["total"].as_u64().unwrap_or(0);
  let dld = dat["downloads"]["daily"].as_u64().unwrap_or(0);
  let dlm = dat["downloads"]["monthly"].as_u64().unwrap_or(0);

  let approx1: f64 = dld as f64 * 7.0;
  let approx2: f64 = dlm as f64 / 4.0;
  let dlw = ((approx1 + approx2) / 2.0) as u64;

  let vers = dat["versions"].as_object();
  let mut vers = vers.map(|x| parse_versions(x)).unwrap_or(vec![]);
  vers.sort_by(|(a, _), (b, _)| b.cmp(a)); // reverse sort by semver

  let stub = (Version::new(0, 0, 0), &serde_json::Value::Null);
  let latest = vers.first().unwrap_or(&stub).1;
  let version = latest["version"].as_str().unwrap_or("unknown").to_string();
  let license = latest["license"][0].as_str().unwrap_or("unknown").to_string();
  let php_ver = latest["require"]["php"].as_str().unwrap_or("unknown").to_string();

  let stars = dat["favers"].as_u64().unwrap_or(0);
  let dependents = dat["dependents"].as_u64().unwrap_or(0);

  Ok(Data { version, license, dlt, dlw, dlm, php_ver, stars, dependents })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
#[allow(clippy::upper_case_acronyms)]
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
  #[serde(rename = "php")]
  PHP,
  #[serde(rename = "stars")]
  Stars,
  #[serde(rename = "dependents")]
  Dependents,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data(name).await?;
  match kind {
    Kind::Version => Ok(badge.for_version("packagist", &rs.version)),
    Kind::License => Ok(badge.for_license(&rs.license)),
    Kind::DlWeek => Ok(badge.for_downloads(Period::Week, rs.dlw)),
    Kind::DlMonth => Ok(badge.for_downloads(Period::Month, rs.dlm)),
    Kind::DlTotal => Ok(badge.for_downloads(Period::Total, rs.dlt)),
    Kind::PHP => Ok(badge.label("php").value(&rs.php_ver)),
    Kind::Stars => Ok(badge.for_count("stars", rs.stars)),
    Kind::Dependents => Ok(badge.for_count("dependents", rs.dependents)),
  }
}
