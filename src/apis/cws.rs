use std::sync::LazyLock;

use axum::extract::{Path, Query};
use badgelib::Badge;
use cached::macros::cached;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::BadgeRep;

// The old `chrome.google.com/webstore/ajax/detail` endpoint was removed by Google.
// Like shields.io (via the `webextension-store-meta` package) we now scrape the public
// store page at `chromewebstore.google.com/detail/<id>`. Values shown there are rounded
// (e.g. "200,000" users, "1.8K ratings").
static RE_VERSION: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r#"class="[^"]*\bnBZElf\b[^"]*">\s*([0-9][0-9.]*)"#).unwrap());
static RE_RATING: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r#"class="[^"]*\bVq0ZA\b[^"]*">\s*([0-9][0-9.]*)"#).unwrap());
static RE_RATING_COUNT: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r#"class="[^"]*\bxJEoWe\b[^"]*">\s*([0-9][0-9.,]*[KMB]?)\s*ratings"#).unwrap()
});
static RE_USERS: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r#"class="[^"]*\bF9iKBc\b[^"]*">(?:[^<]|<[^>]+>)*?([0-9][0-9.,]*[KMB]?)\+?\s*users"#)
    .unwrap()
});

#[derive(Debug, Clone)]
struct Data {
  version: String,
  users: u64,
  score: f64,
  score_count: u64,
}

// Parse a store-formatted number like "200,000", "1.8K" or "2.3M" into a plain integer.
fn parse_metric(raw: &str) -> u64 {
  let s = raw.trim().replace(',', "");
  let (num, mult) = match s.chars().last() {
    Some('K') => (&s[..s.len() - 1], 1_000f64),
    Some('M') => (&s[..s.len() - 1], 1_000_000f64),
    Some('B') => (&s[..s.len() - 1], 1_000_000_000f64),
    _ => (s.as_str(), 1f64),
  };
  (num.parse::<f64>().unwrap_or(0.0) * mult) as u64
}

#[cached(ttl_secs = 60)]
async fn get_data(name: String) -> anyhow::Result<Data> {
  // https://github.com/awesome-webextension/webextension-store-meta
  let url = format!("https://chromewebstore.google.com/detail/{name}?hl=en");
  let req = get_client().get(&url);
  // Bypass the EU consent interstitial that redirects requests from GDPR regions.
  let req = req.header("cookie", "SOCS=CAESEwgDEgk0ODE3Nzk3MjQaAmVuIAEaBgiA_LyaBg");
  let rep = req.send().await?.error_for_status()?;

  // A missing extension redirects to the store homepage, so the final url no longer
  // points at a detail page. Bail here instead of returning empty values.
  if !rep.url().path().contains("/detail/") {
    anyhow::bail!("extension not found");
  }

  let html = rep.text().await?;

  let cap = |re: &Regex| re.captures(&html).map(|c| c[1].to_string());

  let version = cap(&RE_VERSION).unwrap_or_else(|| "unknown".to_string());
  let users = cap(&RE_USERS).map(|s| parse_metric(&s)).unwrap_or(0);
  let score = cap(&RE_RATING).and_then(|s| s.parse().ok()).unwrap_or(0.0);
  let score_count = cap(&RE_RATING_COUNT).map(|s| parse_metric(&s)).unwrap_or(0);

  Ok(Data { version, users, score, score_count })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "rating")]
  Rating,
  #[serde(rename = "rating-count")]
  RatingCount,
  #[serde(rename = "stars")]
  Stars,
  #[serde(rename = "users")]
  Users,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data(name).await?;
  match kind {
    Kind::Version => Ok(badge.for_version("chrome web store", &rs.version)),
    Kind::Rating => Ok(badge.for_rating("rating", rs.score, 5.0)),
    Kind::RatingCount => Ok(badge.for_count("ratings", rs.score_count)),
    Kind::Users => Ok(badge.for_count("users", rs.users)),
    Kind::Stars => Ok(badge.for_stars("stars", rs.score, 5.0)),
  }
}
