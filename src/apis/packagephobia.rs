use axum::{
  extract::{Path, Query},
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, Color};
use crate::server::{Dict, Rep, Res};

#[derive(Debug)]
struct Data {
  publish_pretty: String,
  publish_color: Color,
  install_pretty: String,
  install_color: Color,
}

async fn get_data(name: &str) -> Res<Data> {
  let url = format!("https://packagephobia.com/v2/api.json?p={name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let publish_pretty = dat["publish"]["pretty"].as_str().unwrap_or("unknown").to_string();
  let publish_color = dat["publish"]["color"]
    .as_str()
    .and_then(|x| x.strip_prefix("#"))
    .and_then(|x| Color::from_str(x).ok())
    .unwrap_or(Color::Default);

  let install_pretty = dat["install"]["pretty"].as_str().unwrap_or("unknown").to_string();
  let install_color = dat["install"]["color"]
    .as_str()
    .and_then(|x| x.strip_prefix("#"))
    .and_then(|x| Color::from_str(x).ok())
    .unwrap_or(Color::Default);

  Ok(Data { publish_pretty, publish_color, install_pretty, install_color })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "publish")]
  Publish,
  #[serde(rename = "install")]
  Install,
}

pub async fn handler(
  Path((kind, name)): Path<(Kind, String)>,
  Query(qs): Query<Dict>,
) -> Rep<impl IntoResponse> {
  // Add '@' if it's a scoped package
  let name = if name.contains("/") && !name.starts_with('@') { format!("@{}", name) } else { name };
  let rs = get_data(&name).await?;

  match kind {
    Kind::Publish => {
      Ok(Badge::from_qs_with(&qs, "publish size", &rs.publish_pretty, rs.publish_color)?)
    }
    Kind::Install => {
      Ok(Badge::from_qs_with(&qs, "install size", &rs.install_pretty, rs.install_color)?)
    }
  }
}
