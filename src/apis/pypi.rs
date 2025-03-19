use anyhow::anyhow;
use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, Color, DlPeriod, utils::to_ver_label};
use crate::server::{BadgeRep, Dict, Res};

#[derive(Debug, Clone)]
struct PyPiData {
  version: String,
  license: String,
  pythons: Vec<String>,
  status: String,
  wheel: bool,
  implementation: String,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<PyPiData> {
  // https://pypi.org/pypi?%3Aaction=list_classifiers
  let url = format!("https://pypi.org/pypi/{name}/json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["info"]["version"].as_str().unwrap_or("unknown").into();
  let license = dat["info"]["license"].as_str().unwrap_or("unknown").into();

  let classifiers = dat["info"]["classifiers"]
    .as_array()
    .map(|v| v.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>())
    .unwrap_or_default();

  let pythons = classifiers
    .iter()
    .filter(|v| v.starts_with("Programming Language :: Python :: "))
    .map(|v| v.replace("Programming Language :: Python :: ", ""))
    .collect::<Vec<String>>();

  let status = classifiers
    .iter()
    .find(|v| v.starts_with("Development Status :: "))
    .and_then(|v| v.split(" - ").last())
    .unwrap_or("unknown")
    .to_lowercase()
    .replace("production/stable", "stable");

  let formats = dat["releases"][&version]
    .as_array()
    .map(|v| v.iter().filter_map(|x| x["packagetype"].as_str()).collect::<Vec<_>>())
    .unwrap_or_default();

  // let egg = formats.iter().any(|x| *x == "bdist_egg" || *x == "egg");
  let wheel = formats.iter().any(|x| *x == "bdist_wheel" || *x == "wheel");

  let implementation = classifiers
    .iter()
    .filter(|x| x.starts_with("Programming Language :: Python :: Implementation :: "))
    .map(|x| x.replace("Programming Language :: Python :: Implementation :: ", "").to_lowercase())
    .collect::<Vec<_>>()
    .join(" | ");

  let implementation =
    if implementation.is_empty() { "cpython".to_string() } else { implementation };

  Ok(PyPiData { version, license, pythons, wheel, status, implementation })
}

#[cached(time = 60, result = true)]
async fn get_dl_granular(name: String) -> Res<(u64, u64)> {
  // doc: https://pypistats.org/api/
  let url = format!("https://pypistats.org/api/packages/{}/recent", name);
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dlw = dat["data"]["last_week"].as_u64().unwrap_or(0);
  let dlm = dat["data"]["last_month"].as_u64().unwrap_or(0);
  Ok((dlw, dlm))
}

#[cached(time = 60, result = true)]
async fn get_dl_total(name: String) -> Res<u64> {
  let url = format!("https://pypistats.org/api/packages/{}/overall?mirrors=true", name);
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dlt = dat["data"].as_array().ok_or(anyhow!("no data"))?;
  let dlt = dlt.iter().filter_map(|x| x["downloads"].as_u64());
  let dlt = dlt.sum::<u64>();
  Ok(dlt)
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
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
  #[serde(rename = "python")]
  Python,
  #[serde(rename = "wheel")]
  Wheel,
  #[serde(rename = "status")]
  Status,
  #[serde(rename = "implementation")]
  Implementation,
}

pub async fn handler(Path((kind, name)): Path<(Kind, String)>, Query(qs): Query<Dict>) -> BadgeRep {
  match kind {
    Kind::Version
    | Kind::License
    | Kind::Python
    | Kind::Wheel
    | Kind::Status
    | Kind::Implementation => {
      let rs = get_data(name.clone()).await?;
      match kind {
        Kind::Version => Ok(Badge::for_version(&qs, "pypi", &rs.version)?),
        Kind::License => Ok(Badge::for_license(&qs, &rs.license)?),
        Kind::Python => {
          let value = to_ver_label(rs.pythons);
          Ok(Badge::from_qs_with(&qs, "python", &value, Color::DefaultValue)?)
        }
        Kind::Wheel => {
          let value = if rs.wheel { "yes" } else { "no" };
          // let color = if rs.has_wheel { Color::BrightGreen } else { Color::Red };
          let color = if rs.wheel { Color::DefaultValue } else { Color::Red };
          Ok(Badge::from_qs_with(&qs, "wheel", value, color)?)
        }
        Kind::Status => {
          let color = match rs.status.as_str() {
            "planning" | "pre-alpha" | "alpha" | "inactive" => Color::Red,
            "beta" => Color::Yellow,
            "stable" | "mature" => Color::Green,
            _ => Color::Grey,
          };
          Ok(Badge::from_qs_with(&qs, "status", &rs.status, color)?)
        }
        Kind::Implementation => {
          Ok(Badge::from_qs_with(&qs, "implementation", &rs.implementation, Color::DefaultValue)?)
        }
        _ => unreachable!(),
      }
    }
    Kind::Weekly => Ok(Badge::for_dl(&qs, DlPeriod::Weekly, get_dl_granular(name).await?.0)?),
    Kind::Monthly => Ok(Badge::for_dl(&qs, DlPeriod::Monthly, get_dl_granular(name).await?.1)?),
    Kind::Total => Ok(Badge::for_dl(&qs, DlPeriod::Total, get_dl_total(name).await?)?),
  }
}
