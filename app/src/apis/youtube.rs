use std::time::Duration;

use axum::extract::{Path, Query};
use badgelib::Badge;
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use crate::apis::{get_client, get_env};
use crate::server::{BadgeRep, Res};

#[derive(Debug, Clone)]
struct VideoData {
  views: u64,
  likes: u64,
}

#[cached(time = 60, result = true)]
async fn get_video_data(vid: String) -> Res<VideoData> {
  let key = get_env("YT_TOKEN")?;
  let url = "https://www.googleapis.com/youtube/v3/videos?part=statistics";
  let req = get_client().get(url).query(&[("key", &key), ("id", &vid)]);
  let rep = req.send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dat = &dat["items"][0]["statistics"];
  let views: u64 = dat["viewCount"].as_str().unwrap_or("0").parse()?;
  let likes: u64 = dat["likeCount"].as_str().unwrap_or("0").parse()?;
  // let comments: u64 = dat["commentCount"].as_str().unwrap_or("0").parse()?;

  Ok(VideoData { views, likes })
}

#[derive(Debug, Clone)]
struct ChannelData {
  videos: u64,
  views: u64,
  subs: u64,
}

#[cached(time = 60, result = true)]
async fn get_channel_data(cid: String) -> Res<ChannelData> {
  let var = if cid.starts_with("@") { "forUsername" } else { "id" };
  let cid = cid.trim_start_matches('@');

  let key = get_env("YT_TOKEN")?;
  let url = "https://www.googleapis.com/youtube/v3/channels?part=statistics";
  let req = get_client().get(url).query(&[("key", &key), (var, &cid.into())]);
  let rep = req.send().await?.error_for_status()?;
  let dat = rep.json::<serde_json::Value>().await?;

  let dat = &dat["items"][0]["statistics"];
  let videos = dat["videoCount"].as_str().unwrap_or("0").parse()?;
  let views: u64 = dat["viewCount"].as_str().unwrap_or("0").parse()?;
  let subs: u64 = dat["subscriberCount"].as_str().unwrap_or("0").parse()?;

  Ok(ChannelData { videos, views, subs })
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display, PartialEq)]
pub(crate) enum KindVideo {
  #[serde(rename = "views")]
  Views,
  #[serde(rename = "likes")]
  Likes,
}

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display, PartialEq)]
pub(crate) enum KindChannel {
  #[serde(rename = "views")]
  Views,
  #[serde(rename = "videos")]
  Videos,
  #[serde(rename = "subscribers")]
  Subs,
}

pub async fn video_handler(
  Path((kind, id)): Path<(KindVideo, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_video_data(id).await?;
  match kind {
    KindVideo::Views => Ok(badge.for_count("views", rs.views)),
    KindVideo::Likes => Ok(badge.for_count("likes", rs.likes)),
  }
}

pub async fn channel_handler(
  Path((kind, id)): Path<(KindChannel, String)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let rs = get_channel_data(id).await?;
  match kind {
    KindChannel::Views => Ok(badge.for_count("views", rs.views)),
    KindChannel::Videos => Ok(badge.for_count("videos", rs.videos)),
    KindChannel::Subs => Ok(badge.for_count("subscribers", rs.subs)),
  }
}
