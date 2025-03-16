use axum::extract::{Path, Query};
use cached::proc_macro::cached;

use super::get_client;
use crate::badgelib::{Badge, Color};
use crate::server::{BadgeRep, Dict, Res};

#[cached(time = 60, result = true)]
async fn get_docs(name: String) -> Res<bool> {
  // https://readthedocs.org/api/v3/projects/{}/builds/
  let url = format!("https://readthedocs.org/projects/{}/badge/", name);
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.text().await?;
  Ok(dat.contains("passing"))
}

pub async fn handler(Path(name): Path<String>, Query(qs): Query<Dict>) -> BadgeRep {
  let status = get_docs(name).await?;
  let value = if status { "passing" } else { "failing" };
  let color = if status { Color::Green } else { Color::Red };
  Ok(Badge::from_qs_with(&qs, "docs", value, color)?)
}
