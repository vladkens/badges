use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, Color};
use crate::server::BadgeRep;
use crate::server::{Dict, Res};

#[derive(Debug, Deserialize, Serialize, strum::Display, Hash, Clone, PartialEq, Eq)]
pub(crate) enum Service {
  #[serde(rename = "github", alias = "gh")]
  GitHub,
  #[serde(rename = "gitlab", alias = "gl")]
  GitLab,
  #[serde(rename = "bitbucket", alias = "bb")]
  Bitbucket,
}

#[cached(time = 60, result = true)]
async fn get_coverage(service: Service, name: String) -> Res<u64> {
  let url = format!("https://codecov.io/{service}/{name}/graph/badge.txt");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.text().await?;
  let cov = dat.trim().parse::<u64>()?;
  Ok(cov)
}

#[derive(Deserialize)]
pub(crate) struct Params {
  service: Service,
  user: String,
  repo: String,
  branch: Option<String>,
}

pub async fn handler(
  Path(Params { service, user, repo, branch }): Path<Params>,
  Query(qs): Query<Dict>,
) -> BadgeRep {
  println!(">> service: {:?}, user: {}, repo: {}, branch: {:?}", service, user, repo, branch);

  let name = match branch {
    Some(branch) => format!("{user}/{repo}/branch/{branch}"),
    None => format!("{user}/{repo}"),
  };

  // todo: coverage color
  match get_coverage(service, name).await {
    Ok(cov) => Ok(Badge::from_qs_with(&qs, "coverage", &format!("{cov}%"), Color::DefaultValue)?),
    Err(_) => Ok(Badge::from_qs_with(&qs, "coverage", "unknown", Color::Grey)?),
  }
}
