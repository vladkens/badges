use std::collections::HashMap;

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use maud::html;

use super::_icons::ICONS;
use super::Color;
use super::utils::{cacl_width, millify, to_min_ver};

pub type Res<T = ()> = anyhow::Result<T>;
pub type Dict = HashMap<String, String>;

fn qs_first(qs: &Dict, opts: Vec<&str>) -> Option<String> {
  opts.iter().find_map(|k| qs.get(*k).map(|v| v.to_string()))
}

fn color_or(val: Option<String>, default: Color) -> Color {
  Color::from_str(&val.unwrap_or_default()).unwrap_or(default)
}

fn get_icon(name: &str, color: &str) -> Option<String> {
  let pretenders = [
    name.to_lowercase(),
    name.to_lowercase().replace('-', "").replace("!", "").replace("_", "").replace(" ", ""),
    name.to_lowercase().replace('.', "dot").replace("+", "plus"),
  ];

  let icon = pretenders.iter().find_map(|n| ICONS.get(n))?;
  let icon = format!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" role="img" viewBox="0 0 24 24" fill="{}"><path d="{}" /></svg>"#,
    color, icon
  );

  Some(format!("data:image/svg+xml;base64,{}", BASE64_STANDARD.encode(icon)))
}

pub enum DlPeriod {
  Weekly,
  Monthly,
  Yearly,
  Total,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum BadgeStyle {
  #[default]
  Flat,
  FlatSquare,
  Plastic,
  ForTheBadge,
}

impl BadgeStyle {
  fn parse(s: &str) -> Self {
    match s.to_lowercase().as_str() {
      "flat" => BadgeStyle::Flat,
      "flatsquare" | "flat-square" => BadgeStyle::FlatSquare,
      "forthebadge" | "for-the-badge" => BadgeStyle::ForTheBadge,
      "plastic" => BadgeStyle::Plastic,
      _ => BadgeStyle::Flat,
    }
  }
}

#[derive(Debug)]
pub struct Badge {
  pub label: Option<String>,
  pub label_color: Color,
  pub value: String,
  pub value_color: Color,
  pub icon: Option<String>,
  pub icon_color: Color,
  pub style: BadgeStyle,
  pub radius: u8,
}

impl Badge {
  pub fn new(label: &str, value: &str, value_color: Color) -> Badge {
    Badge {
      label: Some(label.to_string()),
      label_color: Color::DefaultLabel,
      value: value.to_string(),
      value_color,
      icon: None,
      icon_color: Color::Hex("fff".to_string()),
      style: BadgeStyle::Flat,
      radius: 3,
    }
  }

  pub fn from_qs(qs: &Dict) -> Res<Badge> {
    let label = qs.get("label").map(|v| v.to_string());
    let label_color = qs_first(qs, vec!["labelColor", "label_color"]);
    let label_color = color_or(label_color, Color::DefaultLabel);

    let value = qs.get("value").unwrap_or(&"unknown".to_string()).to_string();
    let value_color = qs_first(qs, vec!["color", "valueColor", "value_color"]);
    let value_color = color_or(value_color, Color::DefaultValue);

    let icon = qs_first(qs, vec!["icon", "logo"]);
    let icon_color = qs_first(qs, vec!["iconColor", "icon_color", "logoColor"]);
    let icon_color = color_or(icon_color, Color::Hex("fff".to_string()));

    let style = BadgeStyle::parse(qs.get("style").unwrap_or(&"flat".to_string()));
    let radius_default = if style == BadgeStyle::Flat { 3 } else { 0 };
    let radius = qs.get("radius").and_then(|v| v.parse::<u8>().ok()).unwrap_or(radius_default);
    let radius = radius.min(12);

    Ok(Badge { label, label_color, value, value_color, icon, icon_color, style, radius })
  }

  pub fn from_qs_with(qs: &Dict, label: &str, value: &str, value_color: Color) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;
    badge.label = Some(label.to_string());
    badge.value = value.to_string();
    badge.value_color = value_color;
    Ok(badge)
  }

  pub fn for_version(qs: &Dict, label: &str, value: &str) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    let value = match value.trim() {
      "unknown" | "" => "unknown",
      x if x.starts_with("v") => x,
      x => &format!("v{}", x),
    };

    badge.label = if badge.label.is_none() { Some(label.to_string()) } else { badge.label };
    badge.value = value.to_string();
    badge.value_color = match badge.value_color {
      Color::DefaultValue => Color::from_version(value),
      _ => badge.value_color,
    };

    Ok(badge)
  }

  pub fn for_min_ver(qs: &Dict, label: &str, value: &str) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    badge.label = if badge.label.is_none() { Some(label.to_string()) } else { badge.label };
    badge.value = to_min_ver(value);
    badge.value_color = match badge.value_color {
      Color::DefaultValue => Color::Blue,
      _ => badge.value_color,
    };

    Ok(badge)
  }

  pub fn for_license(qs: &Dict, license: &str) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    badge.label = if badge.label.is_none() { Some("license".to_string()) } else { badge.label };
    badge.value = license.to_string();
    badge.value_color = match badge.value_color {
      Color::DefaultValue => Color::Blue,
      _ => badge.value_color,
    };

    Ok(badge)
  }

  pub fn for_dl(qs: &Dict, period: DlPeriod, value: u64) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    let value = millify(value);
    let value = match period {
      DlPeriod::Weekly => format!("{}/week", value),
      DlPeriod::Monthly => format!("{}/month", value),
      DlPeriod::Yearly => format!("{}/year", value),
      DlPeriod::Total => value,
    };

    badge.label = if badge.label.is_none() { Some("downloads".to_string()) } else { badge.label };
    badge.value = value;
    badge.value_color = match badge.value_color {
      Color::DefaultValue => Color::Green,
      _ => badge.value_color,
    };

    Ok(badge)
  }

  pub fn for_count(qs: &Dict, label: &str, value: u64) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    badge.label = if badge.label.is_none() { Some(label.to_string()) } else { badge.label };
    badge.value = millify(value);
    badge.value_color = match badge.value_color {
      Color::DefaultValue => Color::Blue,
      _ => badge.value_color,
    };

    Ok(badge)
  }

  pub fn to_str(&self) -> String {
    let icon = self.icon.as_deref().unwrap_or_default();
    let icon = get_icon(icon, &self.icon_color.to_css());

    let ltext = self.label.clone().map(|s| s.trim().to_string()).unwrap_or_default();
    let rtext = self.value.clone().trim().to_string();
    let (has_text, has_icon) = (!ltext.is_empty(), icon.is_some());

    #[allow(clippy::nonminimal_bool)]
    let mono = (!has_text && !has_icon)
      || (has_icon && !has_text && self.label_color == Color::DefaultLabel)
      || (ltext.is_empty() && rtext.is_empty());

    let fz = 110.0;
    let ltw = cacl_width(&ltext);
    let rtw = cacl_width(&rtext);
    let pad = fz * 0.5; // left / right padding
    let gap = pad / 1.5; // gap between left and right text

    let iw = if icon.is_some() { fz * 1.2 } else { 0.0 };
    #[allow(unused_assignments)]
    let (mut lx, mut lw, mut rx, mut rw) = (0.0, 0.0, 0.0, 0.0);

    if mono {
      rx = if has_icon { pad + iw + gap } else { pad };
      rw = if rtext.is_empty() { rx - gap + pad } else { rx + rtw + gap };
    } else {
      lx = if has_icon { pad + iw + gap } else { pad };
      lw = if has_text { lx + ltw + gap } else { lx };
      rx = lw + gap;
      rw = rx + rtw + pad - lw;
    }

    let (w, h) = (lw + rw, fz * 1.75);
    let y = h * 0.56;

    let title = if has_text { format!("{ltext}: {rtext}") } else { rtext.to_string() };
    let radius = (fz / 12.0) * self.radius as f32;
    let (outx, outy) = (fz * 0.1 / 2.0, fz * 0.1);

    let svg = html!(svg xmlns="http://www.w3.org/2000/svg"
      viewBox=(format!("0 0 {} {}", w, h))
      width=(w * 20.0 / h) height="20"
      role="img" aria-label=(title)
    {
      title { (title) }

      // background gradient
      @if self.style == BadgeStyle::Flat {
        linearGradient id="s" x2="0" y2="100%" {
          stop offset="0" stop-opacity=".1" stop-color="#eee" {}
          stop offset="1" stop-opacity=".1" {}
        }
      }

      // border-radius
      mask id="r" { rect width=(w) height=(h) rx=(radius) fill="#fff" {} }

      g mask="url(#r)" {
        @if has_text || has_icon { rect x="0" y="0" width=(lw) height=(h) fill=(self.label_color.to_css()) {} }
        rect x=(w-rw) y="0" width=(rw) height=(h) fill=(self.value_color.to_css()) {}
        rect x="0" y="0" width=(w) height=(h) fill="url(#s)" {}
      }

      @if icon.is_some() {
        image x=(pad) y=((h-iw)/2.0) width=(iw) height=(iw) href=(icon.unwrap()) {}
      }

      g fill="#fff" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" font-size=(fz)
        text-anchor="start" dominant-baseline="middle" text-rendering="geometricPrecision"
      {
        @if has_text {
          text textLength=(ltw) x=(lx+outx) y=(y+outy) fill="#000" opacity="0.25" { (&ltext) }
          text textLength=(ltw) x=(lx) y=(y) { (&ltext) }
        }
        text textLength=(rtw) x=(rx+outx) y=(y+outy) fill="#000" opacity="0.25" { (&rtext) }
        text textLength=(rtw) x=(rx) y=(y) { (&rtext) }
      }
    });

    svg.into_string()
  }
}

impl IntoResponse for Badge {
  fn into_response(self) -> Response {
    let headers = [
      (header::CONTENT_TYPE, "image/svg+xml"),
      (header::CACHE_CONTROL, "public,max-age=86400,s-maxage=300,stale-while-revalidate=86400"),
    ];
    (StatusCode::OK, headers, self.to_str()).into_response()
  }
}
