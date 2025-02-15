use std::str::FromStr;

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use maud::html;

use crate::colors::Color;
use crate::icons::ICONS;
use crate::server::{Dict, Res};
use crate::utils::{cacl_width, millify};

fn qs_first(qs: &Dict, opts: Vec<&str>) -> Option<String> {
  opts.iter().find_map(|k| qs.get(*k).map(|v| v.to_string()))
}

fn color_or(val: Option<String>, default: Color) -> Color {
  Color::from_str(&val.unwrap_or_default()).unwrap_or(default)
}

fn get_icon(name: &str, color: &str) -> Option<String> {
  let icon = ICONS.get(name)?;
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

pub struct Badge {
  pub label: Option<String>,
  pub label_color: Color,
  pub value: String,
  pub value_color: Color,
  pub icon: Option<String>,
  pub icon_color: Color,
}

impl Badge {
  pub fn new(label: &str, value: &str, value_color: Color) -> Badge {
    Badge {
      label: Some(label.to_string()),
      label_color: Color::Hex("555".to_string()),
      value: value.to_string(),
      value_color,
      icon: None,
      icon_color: Color::Hex("fff".to_string()),
    }
  }

  pub fn from_qs(qs: &Dict) -> Res<Badge> {
    let label = qs.get("label").map(|v| v.to_string());
    let label_color = qs_first(&qs, vec!["labelColor", "label_color"]);
    let label_color = color_or(label_color, Color::Hex("555".to_string()));

    let value = qs.get("value").unwrap_or(&"unknown".to_string()).to_string();
    let value_color = qs_first(&qs, vec!["color", "valueColor", "value_color"]);
    let value_color = color_or(value_color, Color::Default);

    let icon = qs_first(&qs, vec!["icon", "logo"]);
    let icon_color = qs_first(&qs, vec!["iconColor", "icon_color", "logoColor"]);
    let icon_color = color_or(icon_color, Color::Hex("fff".to_string()));

    Ok(Badge { label, label_color, value, value_color, icon, icon_color })
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
      "unknown" => "unknown",
      x if x.is_empty() => "unknown",
      x if x.starts_with("v") => x,
      x => &format!("v{}", x),
    };

    badge.label = if badge.label.is_none() { Some(label.to_string()) } else { badge.label };
    badge.value = value.to_string();
    badge.value_color = match badge.value_color {
      Color::Default => Color::from_version(value),
      _ => badge.value_color,
    };

    Ok(badge)
  }

  pub fn for_license(qs: &Dict, license: &str) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    badge.label = if badge.label.is_none() { Some("license".to_string()) } else { badge.label };
    badge.value = license.to_string();
    badge.value_color = match badge.value_color {
      Color::Default => Color::Blue,
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
      Color::Default => Color::Green,
      _ => badge.value_color,
    };

    Ok(badge)
  }

  pub fn for_count(qs: &Dict, label: &str, value: u64) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    badge.label = if badge.label.is_none() { Some(label.to_string()) } else { badge.label };
    badge.value = millify(value);
    badge.value_color = match badge.value_color {
      Color::Default => Color::Blue,
      _ => badge.value_color,
    };

    Ok(badge)
  }
}

impl IntoResponse for Badge {
  fn into_response(self) -> Response {
    let icon = get_icon(&self.icon.unwrap_or_default(), &self.icon_color.to_css());

    let ltext = self.label.unwrap_or("".to_string());
    let rtext = self.value.trim().to_string();
    let rtext = if rtext.is_empty() { "unknown".to_string() } else { rtext };
    let (has_text, has_icon) = (!ltext.is_empty(), icon.is_some());

    let fz = 110.0;
    let ltw = cacl_width(&ltext);
    let rtw = cacl_width(&rtext);
    let pad = fz / 2.5;
    let gap = if has_text || has_icon { pad } else { pad };

    let iw = if icon.is_some() { fz * 0.8 } else { 0.0 };
    let lw = match (has_icon, has_text) {
      (true, true) => pad + iw + gap * 1.25 + ltw + gap,
      (true, false) => pad + iw + gap * 1.75,
      (false, true) => pad + ltw + gap,
      (false, false) => 0.0,
    };
    let rw = gap + rtw + pad;

    let (w, h) = (lw + rw, fz * 1.75);
    let y = h * 0.56;

    let off = fz / 12.0;
    let ltx = w - rw - ltw - gap;
    let rtx = w - rw + gap;
    let isz = fz * 1.1;

    let title = if has_text { &format!("{ltext}: {rtext}") } else { &rtext };
    let radius = fz / 4.0;

    let svg = html!(svg xmlns="http://www.w3.org/2000/svg"
      viewBox=(format!("0 0 {} {}", w, h))
      width=(w * 20.0 / h) height="20"
      role="img" aria-label=(title)
    {
      title { (title) }

      // background gradient
      linearGradient id="s" x2="0" y2="100%" {
        stop offset="0" stop-opacity=".1" stop-color="#eee" {}
        stop offset="1" stop-opacity=".1" {}
      }

      // border-radius
      mask id="r" { rect width=(w) height=(h) rx=(radius) fill="#fff" {} }

      g mask="url(#r)" {
        @if has_text || has_icon { rect x="0" y="0" width=(lw) height=(h) fill=(self.label_color.to_css()) {} }
        rect x=(w-rw) y="0" width=(rw) height=(h) fill=(self.value_color.to_css()) {}
        rect x="0" y="0" width=(w) height=(h) fill="url(#s)" {}
      }

      @if icon.is_some() { image x=(gap) y=((h-isz)/2.0) width=(isz) height=(isz) href=(icon.unwrap()) {} }

      g font-size=(fz) fill="#fff" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-anchor="start" dominant-baseline="middle" {
        @if has_text {
          text textLength=(ltw) x=(ltx+off) y=(y+off) fill="#000" opacity="0.1" { (&ltext) }
          text textLength=(ltw) x=(ltx) y=(y) { (&ltext) }
        }
        text textLength=(rtw) x=(rtx+off) y=(y+off) fill="#000" opacity="0.1" { (&rtext) }
        text textLength=(rtw) x=(rtx) y=(y) { (&rtext) }
      }
    });

    let headers = [
      (header::CONTENT_TYPE, "image/svg+xml"),
      (header::CACHE_CONTROL, "public,max-age=86400,s-maxage=300,stale-while-revalidate=86400"),
    ];
    (StatusCode::OK, headers, svg.into_string()).into_response()
  }
}
