use axum::extract::Query;
use badgelib::{Animation, Badge, Color};
use serde::Deserialize;

use crate::server::BadgeRep;

const HANDMADE_ICON: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <path d="M12 7 L13.2 10.8 L17 12 L13.2 13.2 L12 17 L10.8 13.2 L7 12 L10.8 10.8 Z" fill="#f4f4f5" />
  <circle cx="12" cy="12" r="9" fill="none" stroke="#f4f4f5" stroke-width="2" />
  <line x1="5.5" y1="18.5" x2="18.5" y2="5.5" stroke="#f4f4f5" stroke-width="2" />
</svg>"##;

// Keep each published variant immutable so README links remain stable when new
// designs are added.
const VIBE_CODED_ICON: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <path d="M8 1.5 10 9l7.5 2-7.5 2L8 20.5 6 13l-5.5-2L6 9Z" fill="#f472b6"/>
  <path d="m18 3 1.2 3.8L23 8l-3.8 1.2L18 13l-1.2-3.8L13 8l3.8-1.2Z" fill="#22d3ee"/>
  <circle cx="19" cy="18.5" r="3" fill="#facc15"/>
</svg>"##;

pub async fn handmade(Query(badge): Query<Badge>) -> BadgeRep {
  Ok(
    badge
      .value("handmade")
      .value_color(Color::Black)
      .animation(Animation::Shine)
      .icon_svg(HANDMADE_ICON),
  )
}

#[derive(Deserialize)]
pub struct VibeCodedParams {
  variant: Option<u8>,
  #[serde(flatten)]
  badge: Badge,
}

pub async fn vibecoded(Query(params): Query<VibeCodedParams>) -> BadgeRep {
  if !matches!(params.variant, None | Some(1)) {
    return Err(anyhow::anyhow!("unknown vibecoded variant").into());
  }

  Ok(
    params
      .badge
      .value("Vibe Coded")
      .value_gradient([
        Color::Hex("581c87".into()),
        Color::Hex("1e3a8a".into()),
        Color::Hex("164e63".into()),
      ])
      .animation(Animation::Aurora)
      .icon_svg(VIBE_CODED_ICON),
  )
}
