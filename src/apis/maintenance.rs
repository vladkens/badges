use axum::extract::{Path, Query};
use badgelib::{Badge, Color};
use chrono::Datelike;
use serde::{Deserialize, Serialize};

use crate::server::BadgeRep;

#[derive(Debug, Deserialize, Serialize, strum::EnumIter, strum::Display, PartialEq)]
pub(crate) enum Kind {
  #[serde(rename = "yes")]
  Yes,
  #[serde(rename = "no")]
  No,
}

pub async fn handler(
  Path((kind, year)): Path<(Kind, u16)>,
  Query(badge): Query<Badge>,
) -> BadgeRep {
  let current_year = chrono::Utc::now().year() as u16;

  let kind = match kind {
    Kind::No => Kind::No,
    Kind::Yes => {
      if year >= current_year {
        Kind::Yes
      } else {
        Kind::No
      }
    }
  };

  let value = match kind {
    Kind::Yes => "yes".into(),
    Kind::No => format!("no (as of {year})"),
  };

  let color = if kind == Kind::Yes { Color::Green } else { Color::Red };
  Ok(badge.label("maintained").value(&value).value_color(color))
}
