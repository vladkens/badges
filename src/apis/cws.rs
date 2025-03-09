use axum::{
  extract::{Path, Query},
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, Color, utils::render_stars};
use crate::server::{Dict, Rep, Res};

#[derive(Debug)]
struct Data {
  version: String,
  users: String,
  score: f64,
  score_count: u64,
}

async fn get_data(name: &str) -> Res<Data> {
  // https://github.com/simov/chrome-webstore/blob/master/client.js
  let url = "https://chrome.google.com/webstore/ajax/detail";
  let req = get_client().post(url).query(&[("id", name), ("hl", "en"), ("pv", "20210820")]);
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
  #[serde(rename = "users")]
  Users,
  #[serde(rename = "score")]
  Score,
  #[serde(rename = "stars")]
  Stars,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> Rep<impl IntoResponse> {
  let rs = get_data(&name).await?;
  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "chrome web store", &rs.version)?),
    Kind::Users => Ok(Badge::new("users", &rs.users, Color::Default)),
    Kind::Score => Ok(Badge::new("score", &format!("{:.1}/5", rs.score), Color::Default)),
    Kind::Stars => Ok(Badge::new("stars", &render_stars(rs.score, 5.0), Color::Default)),
  }
}
