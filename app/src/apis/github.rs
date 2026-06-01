use anyhow::anyhow;
use axum::extract::{Path, Query};
use badgelib::{Badge, Period};
use cached::macros::cached;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct Base {
  license: String,
  stars: u64,
  forks: u64,
  watchers: u64,
  size: u64,
  lang: String,
}

#[cached(ttl = 60, result = true)]
async fn get_data(name: String) -> Res<Base> {
  let url = format!("https://api.github.com/repos/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let license = dat["license"]["spdx_id"].as_str().unwrap_or("unknown").to_string();
  let stars = dat["stargazers_count"].as_u64().unwrap_or(0);
  let forks = dat["forks_count"].as_u64().unwrap_or(0);
  let watchers = dat["subscribers_count"].as_u64().unwrap_or(0);
  let size = dat["size"].as_u64().unwrap_or(0) * 1024; // in bytes
  let lang = dat["language"].as_str().unwrap_or("unknown").to_string().to_lowercase();

  Ok(Base { license, stars, forks, watchers, size, lang })
}

#[derive(Debug, Clone)]
struct Release {
  version: String,
  dlt: u64,
}

#[cached(ttl = 60, result = true)]
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

#[cached(ttl = 60, result = true)]
async fn last_commit(name: String) -> Res<DateTime<Utc>> {
  let url = format!("https://api.github.com/repos/{name}/commits");
  let req = get_client().get(&url).query(&[("per_page", "1")]);
  let rep = req.send().await?.error_for_status()?;
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

#[cached(ttl = 60, result = true)]
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

  let top_lang = top_lang.to_lowercase();
  Ok(LangData { top_lang, top_percent, count, total })
}

#[cached(ttl = 60, result = true)]
async fn get_contributors(name: String) -> Res<u64> {
  let url = format!("https://api.github.com/repos/{name}/contributors");
  let req = get_client().get(&url).query(&[("page", "1"), ("per_page", "1")]);
  let rep = req.send().await?.error_for_status()?;

  let link = rep.headers().get("Link").and_then(|x| x.to_str().ok()).unwrap_or("");
  let last: u64 = link
    .split(',')
    .find(|x| x.contains("rel=\"last\""))
    .and_then(|x| x.split(';').next())
    .and_then(|x| x.trim().strip_prefix('<').and_then(|x| x.strip_suffix('>')))
    .and_then(|x| x.split('?').next_back())
    .and_then(|x| x.split('&').find(|x| x.starts_with("page=")))
    .and_then(|x| x.split('=').next_back())
    .and_then(|x| x.parse::<u64>().ok())
    .unwrap_or(1);

  Ok(last)
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
  #[serde(rename = "lang")]
  Lang,
  #[serde(rename = "lang-top")]
  LangTop,
  #[serde(rename = "lang-count")]
  LangCount,
  #[serde(rename = "lang-size")]
  LangSize,
  #[serde(rename = "contributors")]
  Contributors,
}

#[derive(Deserialize)]
pub(crate) struct Params {
  kind: Kind,
  user: String,
  repo: String,
}

pub async fn handler(
  Path(Params { kind, user, repo }): Path<Params>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let name = format!("{user}/{repo}");

  match kind {
    Kind::Release => Ok(badge.for_version("release", &get_release(name).await?.version)),
    Kind::AssetsDl => Ok(badge.for_downloads(Period::Total, get_release(name).await?.dlt)),
    Kind::Contributors => Ok(badge.for_count("contributors", get_contributors(name).await?)),
    Kind::License | Kind::Stars | Kind::Forks | Kind::Watchers | Kind::RepoSize | Kind::Lang => {
      let rs = get_data(name).await?;
      match kind {
        Kind::License => Ok(badge.for_license(&rs.license)),
        Kind::Stars => Ok(badge.for_count("stars", rs.stars)),
        Kind::Forks => Ok(badge.for_count("forks", rs.forks)),
        Kind::Watchers => Ok(badge.for_count("watchers", rs.watchers)),
        Kind::RepoSize => Ok(badge.for_size("repo size", rs.size)),
        Kind::Lang => Ok(badge.label("lang").value(&rs.lang)),
        _ => unreachable!(),
      }
    }
    Kind::LangTop | Kind::LangCount | Kind::LangSize => {
      let rs = get_lang(name).await?;
      let lang_value = format!("{:.1}%", rs.top_percent * 100.0);

      match kind {
        Kind::LangTop => Ok(badge.label(&rs.top_lang).value(&lang_value)),
        Kind::LangCount => Ok(badge.for_count("lang count", rs.count)),
        Kind::LangSize => Ok(badge.for_size("code size", rs.total)),
        _ => unreachable!(),
      }
    }
    Kind::LastCommit => Ok(badge.for_duration("last commit", last_commit(name).await?)),
  }
}

pub async fn workflow_handler(
  Path((repo, user, workflow)): Path<(String, String, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let workflow = if !workflow.ends_with(".yml") { format!("{workflow}.yml") } else { workflow };

  let url = format!("https://github.com/{repo}/{user}/actions/workflows/{workflow}/badge.svg");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.text().await?;

  let status = dat.contains(">passing<");
  Ok(badge.for_ci_status("build", status))
}
