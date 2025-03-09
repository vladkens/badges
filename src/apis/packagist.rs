use anyhow::anyhow;
use axum::{
  extract::{Path, Query},
  response::IntoResponse,
};
use semver::Version;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, DlPeriod};
use crate::server::{Dict, Rep, Res};

#[derive(Debug)]
struct PackageData {
  version: String,
  license: String,
  dlt: u64,
  dld: u64,
  dlm: u64,
  php_ver: String,
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

async fn get_data(name: &str) -> Res<PackageData> {
  let url = format!("https://packagist.org/packages/{name}.json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;
  let dat = dat.get("package").ok_or(anyhow!("package not found"))?;

  let dlt = dat["downloads"]["total"].as_u64().unwrap_or(0);
  let dld = dat["downloads"]["daily"].as_u64().unwrap_or(0);
  let dlm = dat["downloads"]["monthly"].as_u64().unwrap_or(0);

  let vers = dat["versions"].as_object();
  let mut vers = vers.map(|x| parse_versions(x)).unwrap_or(vec![]);
  vers.sort_by(|(a, _), (b, _)| b.cmp(a)); // reverse sort by semver

  let stub = (Version::new(0, 0, 0), &serde_json::Value::Null);
  let latest = vers.first().unwrap_or(&stub).1;
  let version = latest["version"].as_str().unwrap_or("unknown").to_string();
  let license = latest["license"][0].as_str().unwrap_or("unknown").to_string();
  let php_ver = latest["require"]["php"].as_str().unwrap_or("unknown").to_string();

  Ok(PackageData { version, license, dlt, dld, dlm, php_ver })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "dw")]
  Weekly,
  #[serde(rename = "dm")]
  Monthly,
  #[serde(rename = "dt")]
  Total,
  #[serde(rename = "php")]
  PHP,
}

pub async fn handler(
  Path((kind, project)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> Rep<impl IntoResponse> {
  let rs = get_data(&project).await?;
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "packagist", &rs.version)?),
    Kind::License => Ok(Badge::for_license(&qs, &rs.license)?),
    Kind::Weekly => {
      let approx1: f64 = rs.dld as f64 * 7.0;
      let approx2: f64 = rs.dlm as f64 / 4.0;
      let approx = ((approx1 + approx2) / 2.0) as u64;
      Ok(Badge::for_dl(&qs, DlPeriod::Weekly, approx)?)
    }
    Kind::Monthly => Ok(Badge::for_dl(&qs, DlPeriod::Monthly, rs.dlm)?),
    Kind::Total => Ok(Badge::for_dl(&qs, DlPeriod::Total, rs.dlt)?),
    Kind::PHP => Ok(Badge::for_min_ver(&qs, "php", &rs.php_ver)?),
  }
}
