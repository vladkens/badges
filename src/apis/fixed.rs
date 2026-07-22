use std::collections::HashMap;

use axum::extract::{Path, Query, Request};
use badgelib::{Badge, Color};

use crate::server::BadgeRep;

pub async fn handler1(Query(badge): Query<Badge>) -> BadgeRep {
  Ok(badge)
}

pub async fn handler2(Path(config): Path<String>, Query(badge): Query<Badge>) -> BadgeRep {
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
    1 => ("", parts[0], Color::Blue),
    2 => ("", parts[0], Color::try_from(parts[1]).unwrap_or_default()),
    3 => (parts[0], parts[1], Color::try_from(parts[2]).unwrap_or_default()),
    _ => return Err(anyhow::anyhow!("Invalid config: {}", config).into()),
  };

  let label = label.replace(TOKEN_UNDER, "_").replace(TOKEN_DASH, "-");
  let value = value.replace(TOKEN_UNDER, "_").replace(TOKEN_DASH, "-");
  // println!(">> {:?} {:?} {:?}", label, value, color);

  Ok(badge.label(&label).value(&value).value_color(color))
}

pub async fn handler3(
  Path((label, value, color)): Path<(String, String, Color)>,
  Query(badge): Query<Badge>,
  req: Request,
) -> BadgeRep {
  // query args have higher priority in this handler
  let qs: Query<HashMap<String, String>> = Query::try_from_uri(req.uri())?;
  let label = qs.get("label").unwrap_or(&label);
  let value = qs.get("value").unwrap_or(&value);
  let color = qs.get("color").and_then(|c| Color::try_from(c.as_str()).ok()).unwrap_or(color);
  Ok(badge.label(label).value(value).value_color(color))
}
