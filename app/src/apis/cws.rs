use std::time::Duration;

use axum::extract::{Path, Query};
use badgelib::Badge;
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct Data {
  version: String,
  users: String,
  score: f64,
  score_count: u64,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  // https://github.com/simov/chrome-webstore/blob/master/client.js
  let url = "https://chrome.google.com/webstore/ajax/detail";
  let opt = &[("id", name.as_str()), ("hl", "en"), ("pv", "20210820")];
  let req = get_client().post(url).query(opt);
  let req = req.body("").header("accept", "*/*").header("content-length", "0").build()?;
  let rep = get_client().execute(req).await?.error_for_status()?;

  let dat = &rep.text().await?[6..];
  let dat: serde_json::Value = serde_json::from_str(dat)?;
  let dat = &dat[1][1]; // detail

  // https://github.com/simov/chrome-webstore/blob/master/map/detail.js
  let version = dat[6].as_str().unwrap_or("unknown").to_string();
  let users = dat[0][23].as_str().unwrap_or("0").to_string();
  let score = dat[0][12].as_f64().unwrap_or(0.0);
  let score_count = dat[0][22].as_u64().unwrap_or(0);

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
    Kind::Users => Ok(badge.label("users").value(&rs.users)),
    Kind::Stars => Ok(badge.for_stars("stars", rs.score, 5.0)),
  }
}
