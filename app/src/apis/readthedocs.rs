use axum::extract::{Path, Query};
use badgelib::Badge;
use cached::proc_macro::cached;

use super::get_client;
use crate::server::{BadgeRep, Res};

#[cached(time = 60, result = true)]
async fn get_docs(name: String) -> Res<bool> {
  // https://readthedocs.org/api/v3/projects/{}/builds/
  let url = format!("https://readthedocs.org/projects/{}/badge/", name);
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.text().await?;
  Ok(dat.contains("passing"))
}

pub async fn handler(Path(name): Path<String>, Query(badge): Query<Badge>) -> BadgeRep {
  let status = get_docs(name).await?;
  Ok(badge.for_ci_status("docs", status))
}
