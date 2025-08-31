use std::time::Duration;

use axum::extract::{Path, Query};
use badgelib::{Badge, Color};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
pub enum Data {
  Members(u64),
  Message(String),
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  let url = format!("https://discord.com/api/v6/guilds/{name}/widget.json");
  let rep = get_client().get(&url).send().await?;

  let status = rep.status();
  let dat = rep.json::<serde_json::Value>().await?;

  match status.as_u16() {
    200 => {
      let members = dat["presence_count"].as_u64().unwrap_or(0);
      Ok(Data::Members(members))
    }
    403 | 404 => {
      let msg = dat["message"].as_str().unwrap_or("unknown error").to_lowercase();
      Ok(Data::Message(msg))
    }
    _ => Err(anyhow::anyhow!("Unexpected response code: {}", status)),
  }
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "online")]
  Online,
}

pub async fn handler(
  Path((_kind, name)): Path<(Kind, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_data(name).await?;
  Ok(match rs {
    Data::Members(m) => badge.label("discord").value(&format!("{} online", m)),
    Data::Message(m) => badge.label("discord").value_color(Color::Red).value(&m),
  })
}
