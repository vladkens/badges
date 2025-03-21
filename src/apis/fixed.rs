use axum::extract::{Path, Query};

use crate::badgelib::{Badge, Color};
use crate::server::{BadgeRep, Dict};

pub async fn handler1(Query(qs): Query<Dict>) -> BadgeRep {
  Ok(Badge::from_qs(&qs)?)
}

pub async fn handler2(Path(config): Path<String>, Query(qs): Query<Dict>) -> BadgeRep {
  // Label, message and color separated by a dash -. For example: `label-message-color`
  // Message and color only, separated by a dash -. For example: `just%20the%20message-8A2BE2`
  // Rules:
  // - Underscore _ or %20 is converted to space
  // - Double underscore __ -> _
  // - Double dash -- -> -

  const TOKEN_UNDER: &str = "<UNDER>";
  const TOKEN_DASH: &str = "<DASH>";

  let config = config.replace("__", TOKEN_UNDER).replace("--", TOKEN_DASH);
  let config = config.replace("_", " ").replace("%20", " ");
  let parts = config.split('-').collect::<Vec<&str>>();

  let (label, value, color) = match parts.len() {
    1 => ("", parts[0], Color::DefaultValue),
    2 => ("", parts[0], Color::from_str(parts[1]).unwrap_or_default()),
    3 => (parts[0], parts[1], Color::from_str(parts[2]).unwrap_or_default()),
    _ => return Err(anyhow::anyhow!("Invalid config: {}", config).into()),
  };

  let label = label.replace(TOKEN_UNDER, "_").replace(TOKEN_DASH, "-");
  let value = value.replace(TOKEN_UNDER, "_").replace(TOKEN_DASH, "-");
  // println!(">> {:?} {:?} {:?}", label, value, color);

  let mut badge = Badge::from_qs(&qs)?;
  badge.llabel = if label.is_empty() { badge.llabel } else { Some(label.to_string()) };
  badge.rlabel = value.to_string();
  badge.rcolor = match badge.rcolor {
    Color::DefaultValue => color,
    _ => badge.rcolor,
  };

  Ok(badge)
}

pub async fn handler3(
  Path((label, value, color)): Path<(String, String, Color)>,
  Query(qs): Query<Dict>,
) -> BadgeRep {
  let label = qs.get("label").unwrap_or(&label);
  let value = qs.get("value").unwrap_or(&value);
  let color = qs.get("color").map_or(color, |x| Color::from_str(x).unwrap_or_default());

  let mut badge = Badge::from_qs(&qs)?;
  badge.llabel = Some(label.to_string());
  badge.rlabel = value.to_string();
  badge.rcolor = match badge.rcolor {
    Color::DefaultValue => color,
    _ => badge.rcolor,
  };

  Ok(badge)
}
