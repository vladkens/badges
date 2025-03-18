use axum::extract::{Path, Query};
use cached::proc_macro::cached;

use super::get_client;
use crate::{
  badgelib::{Badge, Color},
  server::{BadgeRep, Dict, Res},
};

#[derive(Debug, Clone)]
struct Data {
  members: u64,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  let url = format!("https://discord.com/api/v6/guilds/{name}/widget.json");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let members = dat["presence_count"].as_u64().unwrap_or(0);
  Ok(Data { members })
}

pub async fn handler(Path(name): Path<String>, Query(qs): Query<Dict>) -> BadgeRep {
  let rs = get_data(name).await?;
  let value = format!("{} online", rs.members);
  Ok(Badge::from_qs_with(&qs, "discord", &value, Color::DefaultValue)?)
}
