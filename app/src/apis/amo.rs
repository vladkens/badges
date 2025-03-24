use axum::extract::{Path, Query};
use badgelib::{Badge, Period};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
  license: String,
  dlw: u64, // weekly downloads
  users: u64,
  rating: f64,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  let url = format!("https://addons.mozilla.org/api/v4/addons/addon/{name}/");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let version = dat["current_version"]["version"].as_str().unwrap_or("unknown").to_string();
  let license = dat["current_version"]["license"]["url"].as_str().unwrap_or("unknown").to_string();
  let license = license.split('/').last().unwrap_or(&license).to_string();
  let license = license.strip_suffix(".html").unwrap_or(&license).to_string();
  let dlw = dat["weekly_downloads"].as_u64().unwrap_or(0);
  let users = dat["average_daily_users"].as_u64().unwrap_or(0);
  let rating = dat["ratings"]["average"].as_f64().unwrap_or(0.0);

  Ok(Data { version, license, dlw, users, rating })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "l", alias = "license")]
  License,
  #[serde(rename = "dw")]
  DlWeek,
  #[serde(rename = "rating")]
  Rating,
  #[serde(rename = "users")]
  Users,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data(name).await?;
  match kind {
    Kind::Version => Ok(badge.for_version("mozilla add-on", &rs.version)),
    Kind::License => Ok(badge.for_license(&rs.license)),
    Kind::DlWeek => Ok(badge.for_downloads(Period::Week, rs.dlw)),
    Kind::Rating => Ok(badge.for_rating("rating", rs.rating, 5.0)),
    Kind::Users => Ok(badge.for_count("users", rs.users)),
  }
}
