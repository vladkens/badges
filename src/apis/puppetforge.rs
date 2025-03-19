use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, Color, DlPeriod};
use crate::server::{BadgeRep, Dict, Res};

#[derive(Debug, Clone)]
struct Data {
  ver: String,
  dlt: u64,
  score: u64,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  let url = format!("https://forgeapi.puppetlabs.com/v3/modules/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let ver = dat["current_release"]["version"].as_str().unwrap_or("unknown").into();
  let dlt = dat["downloads"].as_u64().unwrap_or(0);
  let score = dat["current_release"]["validation_score"].as_u64().unwrap_or_default();

  Ok(Data { ver, dlt, score })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display)]
pub(crate) enum Kind {
  #[serde(rename = "version", alias = "v")]
  Version,
  #[serde(rename = "dt")]
  Total,
  #[serde(rename = "score")]
  Score,
}

pub async fn handler(Path((kind, name)): Path<(Kind, String)>, Query(qs): Query<Dict>) -> BadgeRep {
  let name = name.replace("/", "-");
  let rs = get_data(name).await?;

  match kind {
    Kind::Version => Ok(Badge::for_version(&qs, "puppetforge", &rs.ver)?),
    Kind::Total => Ok(Badge::for_dl(&qs, DlPeriod::Total, rs.dlt)?),
    Kind::Score => {
      let value = format!("{}%", rs.score);
      Ok(Badge::from_qs_with(&qs, "quality score", &value, Color::DefaultValue)?)
    }
  }
}
