use anyhow::anyhow;
use axum::extract::{Path, Query};
use badgelib::{Badge, Color, Period};
use cached::macros::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::BadgeRep;

#[derive(Debug, Clone)]
struct PyPiData {
  version: String,
  license: String,
  pythons: Vec<String>,
  status: String,
  wheel: bool,
  implementation: String,
}

fn parse_version(v: &str) -> Vec<u32> {
  v.split('.').map(|part| part.parse::<u32>().unwrap_or(0)).collect()
}

fn get_python_versions(classifiers: &[&str], requires_python: Option<&str>) -> Vec<String> {
  let mut pythons = classifiers
    .iter()
    .filter(|x| x.starts_with("Programming Language :: Python :: "))
    .map(|x| x.replace("Programming Language :: Python :: ", ""))
    .filter(|x| x.contains('.') && x.split('.').all(|part| part.parse::<u32>().is_ok()))
    .collect::<Vec<_>>();

  if pythons.is_empty()
    && let Some(requires_python) = requires_python.filter(|x| !x.trim().is_empty())
  {
    pythons.push(requires_python.replace(">=", "≥").replace("<=", "≤"));
  }

  pythons.sort_by_key(|x| parse_version(x));
  pythons
}

#[cached(ttl = 60)]
async fn get_data(name: String) -> anyhow::Result<PyPiData> {
  // https://pypi.org/pypi?%3Aaction=list_classifiers
  let url = format!("https://pypi.org/pypi/{name}/json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["info"]["version"].as_str().unwrap_or("unknown").into();
  let license = dat["info"]["license"].as_str().unwrap_or("unknown").into();

  let classifiers = dat["info"]["classifiers"]
    .as_array()
    .map(|x| x.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>())
    .unwrap_or_default();

  let pythons = get_python_versions(&classifiers, dat["info"]["requires_python"].as_str());

  let status = classifiers
    .iter()
    .find(|x| x.starts_with("Development Status :: "))
    .and_then(|x| x.split(" - ").last())
    .unwrap_or("unknown")
    .to_lowercase()
    .replace("production/stable", "stable");

  let formats = dat["releases"][&version]
    .as_array()
    .map(|x| x.iter().filter_map(|x| x["packagetype"].as_str()).collect::<Vec<_>>())
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

#[cached(ttl = 3600)]
async fn get_dl_granular(name: String) -> anyhow::Result<Option<(u64, u64)>> {
  // doc: https://pypistats.org/api/
  let url = format!("https://pypistats.org/api/packages/{}/recent", name);
  let rep = get_client().get(&url).send().await?;
  if rep.status() == reqwest::StatusCode::NOT_FOUND {
    get_data(name).await?;
    return Ok(None);
  }
  let rep = rep.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dlw = dat["data"]["last_week"].as_u64().unwrap_or(0);
  let dlm = dat["data"]["last_month"].as_u64().unwrap_or(0);
  Ok(Some((dlw, dlm)))
}

#[cached(ttl = 3600)]
async fn get_dl_total(name: String) -> anyhow::Result<Option<u64>> {
  let url = format!("https://pypistats.org/api/packages/{}/overall?mirrors=true", name);
  let rep = get_client().get(&url).send().await?;
  if rep.status() == reqwest::StatusCode::NOT_FOUND {
    get_data(name).await?;
    return Ok(None);
  }
  let rep = rep.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dlt = dat["data"].as_array().ok_or(anyhow!("no data"))?;
  let dlt = dlt.iter().filter_map(|x| x["downloads"].as_u64());
  let dlt = dlt.sum::<u64>();
  Ok(Some(dlt))
}

fn download_badge(badge: Badge, period: Period, downloads: Option<u64>) -> Badge {
  match downloads {
    Some(downloads) => badge.for_downloads(period, downloads),
    None => badge.for_downloads(period, 0).value("none").value_color(Color::Gray),
  }
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
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
  #[serde(rename = "python")]
  Python,
  #[serde(rename = "wheel")]
  Wheel,
  #[serde(rename = "status")]
  Status,
  #[serde(rename = "implementation")]
  Implementation,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  match kind {
    Kind::DlTotal => {
      return Ok(download_badge(badge, Period::Total, get_dl_total(name).await?));
    }
    Kind::DlWeek => {
      let downloads = get_dl_granular(name).await?.map(|downloads| downloads.0);
      return Ok(download_badge(badge, Period::Week, downloads));
    }
    Kind::DlMonth => {
      let downloads = get_dl_granular(name).await?.map(|downloads| downloads.1);
      return Ok(download_badge(badge, Period::Month, downloads));
    }
    _ => {}
  }

  let rs = get_data(name).await?;
  match kind {
    Kind::DlTotal | Kind::DlWeek | Kind::DlMonth => unreachable!(),
    Kind::Version => Ok(badge.for_version("pypi", &rs.version)),
    Kind::License => Ok(badge.for_license(&rs.license)),
    Kind::Python => {
      let value = match rs.pythons.len() {
        0 => "unknown".to_string(),
        1 => rs.pythons[0].clone(),
        2 => format!("{} | {}", rs.pythons[0], rs.pythons[1]),
        _ => format!("{} – {}", rs.pythons.first().unwrap(), rs.pythons.last().unwrap()),
      };

      Ok(badge.label("python").value(&value))
    }
    Kind::Wheel => {
      let value = if rs.wheel { "yes" } else { "no" };
      let color = if rs.wheel { Color::Green } else { Color::Red };
      Ok(badge.label("wheel").value(value).value_color(color))
    }
    Kind::Status => {
      let color = match rs.status.as_str() {
        "planning" | "pre-alpha" | "alpha" | "inactive" => Color::Red,
        "beta" => Color::Yellow,
        "stable" | "mature" => Color::Green,
        _ => Color::Gray,
      };
      Ok(badge.label("status").value(&rs.status).value_color(color))
    }
    Kind::Implementation => Ok(badge.label("implementation").value(&rs.implementation)),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn falls_back_to_requires_python_without_version_classifiers() {
    let classifiers = [
      "Development Status :: 3 - Alpha",
      "Programming Language :: Python :: 3",
      "Programming Language :: Python :: 3 :: Only",
    ];
    assert_eq!(get_python_versions(&classifiers, Some(">=3.11")), ["≥3.11"]);
  }

  #[test]
  fn prefers_version_classifiers_over_requires_python() {
    let classifiers =
      ["Programming Language :: Python :: 3.12", "Programming Language :: Python :: 3.11"];
    assert_eq!(get_python_versions(&classifiers, Some(">=3.11")), ["3.11", "3.12"]);
  }
}
