use anyhow::anyhow;
use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::Color;
use crate::badgelib::utils::for_date;
use crate::server::{Dict, Res};
use crate::{
  badgelib::{Badge, DlPeriod},
  server::BadgeRep,
};

#[derive(Debug, Clone)]
struct Base {
  license: String,
  stars: u64,
  forks: u64,
  watchers: u64,
  size: u64,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Base> {
  let url = format!("https://api.github.com/repos/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let license = dat["license"]["spdx_id"].as_str().unwrap_or("unknown").to_string();
  let stars = dat["stargazers_count"].as_u64().unwrap_or(0);
  let forks = dat["forks_count"].as_u64().unwrap_or(0);
  let watchers = dat["watchers_count"].as_u64().unwrap_or(0);
  let size = dat["size"].as_u64().unwrap_or(0) * 1024; // in bytes

  Ok(Base { license, stars, forks, watchers, size })
}

#[derive(Debug, Clone)]
struct Release {
  version: String,
  dlt: u64,
}

#[cached(time = 60, result = true)]
async fn get_release(name: String) -> Res<Release> {
  let url = format!("https://api.github.com/repos/{name}/releases/latest");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["tag_name"].as_str().unwrap_or("unknown").to_string();
  let dlt = dat["assets"]
    .as_array()
    .map(|p| p.iter().filter_map(|x| x["download_count"].as_u64()).sum::<u64>())
    .unwrap_or(0);

  Ok(Release { version, dlt })
}

#[cached(time = 60, result = true)]
async fn last_commit(name: String) -> Res<DateTime<Utc>> {
  let url = format!("https://api.github.com/repos/{name}/commits");
  let rep = get_client().get(&url).query(&[("per_page", "1")]);
  let rep = rep.send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  dat[0]["commit"]["author"]["date"]
    .as_str()
    .and_then(|x| x.parse::<DateTime<Utc>>().ok())
    .ok_or_else(|| anyhow!("no date"))
}

#[derive(Debug, Clone)]
struct LangData {
  top_lang: String,
  top_percent: f32,
  count: u64,
  total: u64,
}

#[cached(time = 60, result = true)]
async fn get_lang(name: String) -> Res<LangData> {
  let url = format!("https://api.github.com/repos/{name}/languages");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let mut langs: Vec<(String, u64)> = dat
    .as_object()
    .map(|p| p.iter().map(|(k, v)| (k.to_string(), v.as_u64().unwrap_or(0))).collect())
    .unwrap_or_default();

  langs.sort_by_key(|(_, v)| *v);
  langs.reverse();

  let total = langs.iter().map(|(_, v)| v).sum::<u64>();
  let top_lang = langs.first().map(|(k, _)| k.clone()).unwrap_or_else(|| "unknown".to_string());
  let top_percent = langs.first().map(|(_, v)| *v as f32 / total as f32).unwrap_or(0.0);
  let count = langs.len() as u64;

  Ok(LangData { top_lang, top_percent, count, total })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "release")]
  Release,
  #[serde(rename = "assets-dl")]
  AssetsDl,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "stars")]
  Stars,
  #[serde(rename = "forks")]
  Forks,
  #[serde(rename = "watchers")]
  Watchers,
  #[serde(rename = "last-commit")]
  LastCommit,
  #[serde(rename = "repo-size")]
  RepoSize,
  #[serde(rename = "lang-top")]
  LangTop,
  #[serde(rename = "lang-count")]
  LangCount,
  #[serde(rename = "lang-size")]
  LangSize,
}

pub async fn handler(Path((kind, name)): Path<(Kind, String)>, Query(qs): Query<Dict>) -> BadgeRep {
  match kind {
    Kind::Release => Ok(Badge::for_version(&qs, "release", &get_release(name).await?.version)?),
    Kind::AssetsDl => Ok(Badge::for_dl(&qs, DlPeriod::Total, get_release(name).await?.dlt)?),
    Kind::License | Kind::Stars | Kind::Forks | Kind::Watchers | Kind::RepoSize => {
      let rs = get_data(name).await?;
      match kind {
        Kind::License => Ok(Badge::for_license(&qs, &rs.license)?),
        Kind::Stars => Ok(Badge::for_count(&qs, "stars", rs.stars)?),
        Kind::Forks => Ok(Badge::for_count(&qs, "forks", rs.forks)?),
        Kind::Watchers => Ok(Badge::for_count(&qs, "watchers", rs.watchers)?),
        Kind::RepoSize => Ok(Badge::for_size(&qs, "repo size", rs.size)?),
        _ => unreachable!(),
      }
    }
    Kind::LangTop | Kind::LangCount | Kind::LangSize => {
      let rs = get_lang(name).await?;
      let lang_value = format!("{:.1}%", rs.top_percent * 100.0);

      match kind {
        Kind::LangTop => {
          Ok(Badge::from_qs_with(&qs, &rs.top_lang, &lang_value, Color::DefaultValue)?)
        }
        Kind::LangCount => Ok(Badge::for_count(&qs, "lang count", rs.count)?),
        Kind::LangSize => Ok(Badge::for_size(&qs, "code size", rs.total)?),
        _ => unreachable!(),
      }
    }
    Kind::LastCommit => {
      let (value, color) = for_date(last_commit(name).await?);
      Ok(Badge::from_qs_with(&qs, "last commit", &value, color)?)
    }
  }
}

pub async fn workflow_handler(
  Path((repo, user, workflow)): Path<(String, String, String)>,
  Query(qs): Query<Dict>,
) -> BadgeRep {
  let workflow = if !workflow.ends_with(".yml") { format!("{workflow}.yml") } else { workflow };

  let url = format!("https://github.com/{repo}/{user}/actions/workflows/{workflow}/badge.svg");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.text().await?;

  let status = dat.contains(">passing<");
  let value = if status { "passing" } else { "failing" };
  let color = if status { Color::Green } else { Color::Red };
  Ok(Badge::from_qs_with(&qs, "build", value, color)?)
}
