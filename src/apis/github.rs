use std::collections::HashSet;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail};
use axum::extract::{Path, Query};
use badgelib::{Badge, Color, Period};
use cached::macros::cached;
use chrono::{DateTime, Utc};
use reqwest::header::HeaderMap;
use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::BadgeRep;

struct GitHubToken {
  value: String,
  blocked_until: AtomicU64,
}

struct GitHubClient {
  client: reqwest::Client,
  tokens: Vec<GitHubToken>,
}

impl GitHubClient {
  fn from_env() -> Self {
    let value = std::env::var("GH_TOKENS").unwrap_or_default();
    Self::new(parse_tokens(&value))
  }

  fn new(tokens: Vec<String>) -> Self {
    let tokens = tokens
      .into_iter()
      .map(|value| GitHubToken { value, blocked_until: AtomicU64::new(0) })
      .collect();

    Self { client: get_client(), tokens }
  }

  fn available_tokens(&self, now: u64) -> Vec<usize> {
    let mut tokens = self
      .tokens
      .iter()
      .enumerate()
      .filter_map(|(index, token)| {
        (token.blocked_until.load(Ordering::Relaxed) <= now).then_some(index)
      })
      .collect::<Vec<_>>();
    fastrand::shuffle(&mut tokens);
    tokens
  }

  async fn get(&self, url: &str, query: &[(&str, &str)]) -> anyhow::Result<Response> {
    if self.tokens.is_empty() {
      return Ok(self.client.get(url).query(query).send().await?.error_for_status()?);
    }

    let now = unix_time();
    let tokens = self.available_tokens(now);
    if tokens.is_empty() {
      let reset = self
        .tokens
        .iter()
        .map(|token| token.blocked_until.load(Ordering::Relaxed))
        .min()
        .unwrap_or(now);
      if reset == u64::MAX {
        bail!("all github tokens were rejected");
      }
      bail!("all github tokens are rate limited until {reset}");
    }

    let mut last_rate_limit = None;
    for index in tokens {
      let token = &self.tokens[index];
      let rep = self.client.get(url).query(query).bearer_auth(&token.value).send().await?;

      if rep.status() == StatusCode::UNAUTHORIZED {
        token.blocked_until.store(u64::MAX, Ordering::Relaxed);
        tracing::warn!(token_index = index, "github token rejected");
        continue;
      }

      if let Some(reset) = rate_limit_reset(rep.status(), rep.headers(), now) {
        token.blocked_until.store(reset, Ordering::Relaxed);
        tracing::warn!(token_index = index, reset_at = reset, "github token rate limited");
        last_rate_limit = Some(rep);
        continue;
      }

      return Ok(rep.error_for_status()?);
    }

    if let Some(rep) = last_rate_limit {
      return Ok(rep.error_for_status()?);
    }

    bail!("all github tokens were rejected")
  }
}

fn parse_tokens(value: &str) -> Vec<String> {
  let mut seen = HashSet::new();
  value
    .split(|c: char| c == ',' || c.is_whitespace())
    .filter(|token| !token.is_empty())
    .filter(|token| seen.insert(*token))
    .map(str::to_string)
    .collect()
}

fn unix_time() -> u64 {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn rate_limit_reset(status: StatusCode, headers: &HeaderMap, now: u64) -> Option<u64> {
  if status != StatusCode::FORBIDDEN && status != StatusCode::TOO_MANY_REQUESTS {
    return None;
  }

  let remaining = headers.get("x-ratelimit-remaining")?.to_str().ok()?;
  if remaining != "0" {
    return None;
  }

  Some(
    headers
      .get("x-ratelimit-reset")
      .and_then(|value| value.to_str().ok())
      .and_then(|value| value.parse().ok())
      .unwrap_or(now + 60),
  )
}

async fn github_get(url: &str) -> anyhow::Result<Response> {
  github_get_with_query(url, &[]).await
}

async fn github_get_with_query(url: &str, query: &[(&str, &str)]) -> anyhow::Result<Response> {
  static CLIENT: OnceLock<GitHubClient> = OnceLock::new();
  CLIENT.get_or_init(GitHubClient::from_env).get(url, query).await
}

#[derive(Debug, Clone)]
struct Base {
  license: String,
  stars: u64,
  forks: u64,
  watchers: u64,
  size: u64,
  lang: String,
}

#[cached(ttl = 60)]
async fn get_data(name: String) -> anyhow::Result<Base> {
  let url = format!("https://api.github.com/repos/{name}");
  let rep = github_get(&url).await?;
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

#[derive(Debug, Clone)]
enum ReleaseState {
  Published(Release),
  Missing,
  RepositoryMissing,
}

#[cached(ttl = 60)]
async fn get_release(name: String) -> anyhow::Result<ReleaseState> {
  let url = format!("https://api.github.com/repos/{name}/releases/latest");
  let rep = match github_get(&url).await {
    Ok(rep) => rep,
    Err(err) if error_status(&err) == Some(StatusCode::NOT_FOUND) => {
      let repo_url = format!("https://api.github.com/repos/{name}");
      return match github_get(&repo_url).await {
        Ok(_) => Ok(ReleaseState::Missing),
        Err(err) if error_status(&err) == Some(StatusCode::NOT_FOUND) => {
          Ok(ReleaseState::RepositoryMissing)
        }
        Err(err) => Err(err),
      };
    }
    Err(err) => return Err(err),
  };
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["tag_name"].as_str().unwrap_or("unknown").to_string();
  let dlt = dat["assets"]
    .as_array()
    .map(|p| p.iter().filter_map(|x| x["download_count"].as_u64()).sum::<u64>())
    .unwrap_or(0);

  Ok(ReleaseState::Published(Release { version, dlt }))
}

fn error_status(err: &anyhow::Error) -> Option<StatusCode> {
  err.downcast_ref::<reqwest::Error>().and_then(reqwest::Error::status)
}

#[cached(ttl = 60)]
async fn last_commit(name: String) -> anyhow::Result<DateTime<Utc>> {
  let url = format!("https://api.github.com/repos/{name}/commits");
  let rep = github_get_with_query(&url, &[("per_page", "1")]).await?;
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

#[cached(ttl = 60)]
async fn get_lang(name: String) -> anyhow::Result<LangData> {
  let url = format!("https://api.github.com/repos/{name}/languages");
  let rep = github_get(&url).await?;
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

#[cached(ttl = 60)]
async fn get_contributors(name: String) -> anyhow::Result<u64> {
  let url = format!("https://api.github.com/repos/{name}/contributors");
  let rep = github_get_with_query(&url, &[("page", "1"), ("per_page", "1")]).await?;

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
    kind @ (Kind::Release | Kind::AssetsDl) => {
      let release = get_release(name).await?;
      let badge = match (kind, release) {
        (_, ReleaseState::RepositoryMissing) => {
          badge.label("repo").value("not found").value_color(Color::Red)
        }
        (Kind::Release, ReleaseState::Published(release)) => {
          badge.for_version("release", &release.version)
        }
        (Kind::Release, ReleaseState::Missing) => {
          badge.for_version("release", "none").value_color(Color::Gray)
        }
        (Kind::AssetsDl, ReleaseState::Published(release)) => {
          badge.for_downloads(Period::Total, release.dlt)
        }
        (Kind::AssetsDl, ReleaseState::Missing) => {
          badge.for_downloads(Period::Total, 0).value("none").value_color(Color::Gray)
        }
        _ => unreachable!(),
      };
      Ok(badge)
    }
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_tokens_from_spaces_commas_and_lines() {
    let tokens = parse_tokens("one two,three\nfour\t five,one");
    assert_eq!(tokens, ["one", "two", "three", "four", "five"]);
  }

  #[test]
  fn recognizes_primary_rate_limit() {
    let mut headers = HeaderMap::new();
    headers.insert("x-ratelimit-remaining", "0".parse().unwrap());
    headers.insert("x-ratelimit-reset", "200".parse().unwrap());

    assert_eq!(rate_limit_reset(StatusCode::FORBIDDEN, &headers, 100), Some(200));
    assert_eq!(rate_limit_reset(StatusCode::INTERNAL_SERVER_ERROR, &headers, 100), None);
  }

  #[test]
  fn ignores_non_rate_limit_forbidden_response() {
    let mut headers = HeaderMap::new();
    headers.insert("x-ratelimit-remaining", "42".parse().unwrap());

    assert_eq!(rate_limit_reset(StatusCode::FORBIDDEN, &headers, 100), None);
  }

  #[test]
  fn excludes_rate_limited_tokens() {
    let client = GitHubClient::new(vec!["one".into(), "two".into(), "three".into()]);
    client.tokens[1].blocked_until.store(200, Ordering::Relaxed);

    let tokens = client.available_tokens(100);
    assert_eq!(tokens.len(), 2);
    assert!(tokens.contains(&0));
    assert!(tokens.contains(&2));
  }

  #[test]
  fn restores_tokens_after_reset() {
    let client = GitHubClient::new(vec!["one".into(), "two".into()]);
    client.tokens[0].blocked_until.store(100, Ordering::Relaxed);

    let tokens = client.available_tokens(100);
    assert_eq!(tokens.len(), 2);
  }
}
