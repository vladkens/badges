use anyhow::anyhow;
use axum::extract::{Path, Query};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use super::get_client;
use crate::badgelib::{Badge, Color};
use crate::server::BadgeRep;
use crate::server::{Dict, Res};

#[derive(Debug, Clone)]
struct Data {
  stars: u64,
  pulls: u64,
  automated: bool,
}

#[cached(time = 60, result = true)]
async fn get_data(name: String) -> Res<Data> {
  let url = format!("https://hub.docker.com/v2/repositories/{name}");
  let rep = get_client().get(&url).send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let stars = dat["star_count"].as_u64().unwrap_or(0);
  let pulls = dat["pull_count"].as_u64().unwrap_or(0);
  let automated = dat["is_automated"].as_bool().unwrap_or(false);

  Ok(Data { stars, pulls, automated })
}

#[derive(Debug, Clone)]
struct Tags {
  image_tag: String,
  image_size: u64,
}

async fn get_tags(name: String, tag: Option<String>) -> Res<Tags> {
  let url = format!("https://hub.docker.com/v2/repositories/{name}/tags");
  let req = get_client().get(&url).query(&[("ordering", "last_updated"), ("page_size", "100")]);
  let req = if let Some(ref tag) = tag { req.query(&[("name", tag)]) } else { req };

  let rep = req.send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  // [(name, [(arch, size), ..]), ..]
  let tags = dat["results"]
    .as_array()
    .ok_or(anyhow!("no data"))?
    .iter()
    .filter_map(|tag| {
      let name = tag["name"].as_str();
      let images = tag["images"].as_array().and_then(|images| {
        let images = images
          .iter()
          .filter_map(|image| {
            let arch = image["architecture"].as_str();
            let size = image["size"].as_u64();
            match (arch, size) {
              (Some(arch), Some(size)) => Some((arch, size)),
              _ => None,
            }
          })
          .collect::<Vec<_>>();

        if images.is_empty() { None } else { Some(images) }
      });

      match (name, images) {
        (Some(name), Some(images)) => Some((name, images)),
        _ => None,
      }
    })
    .collect::<Vec<_>>();

  // priority:
  // 1. if tag provided search tag with that name
  // 2. if tag provided search tag starting with that name
  // 3. if no tag provided search latest 'v' tag
  // 4. if no tag provided return first tag

  let target = if let Some(ref tag) = tag {
    tags
      .iter()
      .find(|(name, _)| *name == tag)
      .or_else(|| tags.iter().find(|(name, _)| name.starts_with(tag)))
  } else {
    None
  };

  let target = target.or_else(|| tags.iter().find(|(name, _)| name.starts_with("v")));
  let target = target.or_else(|| tags.first());

  let default_arch = "amd64";
  let image_tag = target.map(|(name, _)| name.to_string()).unwrap_or("unknown".to_string());
  let image_size = target
    .and_then(|(_, images)| {
      images
        .iter()
        .find(|(arch, _)| *arch == default_arch)
        .or_else(|| images.first())
        .map(|(_, size)| *size)
    })
    .unwrap_or(0);

  Ok(Tags { image_tag, image_size })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display, PartialEq)]
pub(crate) enum Kind {
  #[serde(rename = "v", alias = "version")]
  Version,
  #[serde(rename = "image-size")]
  Size,
  #[serde(rename = "pulls")]
  Pulls,
  #[serde(rename = "stars")]
  Stars,
  #[serde(rename = "automated")]
  Automated,
}

#[derive(Deserialize)]
pub(crate) struct Params {
  kind: Kind,
  user: String,
  repo: String,
  tag: Option<String>,
}

pub async fn handler(
  Path(Params { kind, user, repo, tag }): Path<Params>,
  Query(qs): Query<Dict>,
) -> BadgeRep {
  let name = format!("{}/{}", user, repo);

  match kind {
    Kind::Stars | Kind::Pulls | Kind::Automated => {
      let rs = get_data(name).await?;
      match kind {
        Kind::Stars => Ok(Badge::for_count(&qs, "docker stars", rs.stars)?),
        Kind::Pulls => Ok(Badge::for_count(&qs, "docker pulls", rs.pulls)?),
        Kind::Automated => {
          // todo: colors
          // https://shields.io/docker/automated/jumpserver/jms_all
          // https://img.shields.io/docker/automated/ellerbrock/bash-it
          let value = if rs.automated { "automated" } else { "manual" };
          Ok(Badge::from_qs_with(&qs, "docker build", value, Color::DefaultValue)?)
        }
        _ => unreachable!(),
      }
    }
    Kind::Version | Kind::Size => {
      let rs = get_tags(name, tag).await?;
      match kind {
        Kind::Version => Ok(Badge::for_version(&qs, "image version", &rs.image_tag)?),
        Kind::Size => Ok(Badge::for_size(&qs, "image size", rs.image_size)?),
        _ => unreachable!(),
      }
    }
  }
}
