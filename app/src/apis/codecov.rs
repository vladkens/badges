use axum::extract::{Path, Query};
use badgelib::{Badge, Color};
use cached::macros::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::server::{BadgeRep, Res};

#[derive(Debug, Deserialize, Serialize, strum::Display, Hash, Clone, PartialEq, Eq)]
pub(crate) enum Service {
  #[serde(rename = "github", alias = "gh")]
  GitHub,
  #[serde(rename = "gitlab", alias = "gl")]
  GitLab,
  #[serde(rename = "bitbucket", alias = "bb")]
  Bitbucket,
}

#[cached(ttl = 60, result = true)]
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
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let name = match branch {
    Some(branch) => format!("{user}/{repo}/branch/{branch}"),
    None => format!("{user}/{repo}"),
  };

  match get_coverage(service, name).await {
    Ok(cov) => {
      let color = match cov {
        0..=50 => Color::Red,
        51..=69 => Color::Orange,
        70..=84 => Color::Yellow,
        85..=99 => Color::Green,
        _ => Color::Green,
      };
      Ok(badge.label("coverage").value(&format!("{}%", cov)).value_color(color))
    }
    Err(_) => Ok(badge.label("coverage").value("unknown").value_color(Color::Gray)),
  }
}
