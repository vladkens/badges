use std::collections::HashMap;

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use maud::html;

use super::_icons::ICONS;
use super::Color;
use super::utils::{cacl_width, millify, millify_iec, to_min_ver};

pub type Res<T = ()> = anyhow::Result<T>;
pub type Dict = HashMap<String, String>;

const DEFAULT_CACHE: u32 = 86400; // 24 hours

fn qs_first(qs: &Dict, opts: &[&str]) -> Option<String> {
  opts.iter().find_map(|k| qs.get(*k).map(|v| v.to_string()))
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

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize)]
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

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum BadgeFormat {
  Svg,
  Json,
}

impl BadgeFormat {
  pub fn parse(s: &str) -> Self {
    match s.to_lowercase().as_str() {
      "json" => BadgeFormat::Json,
      _ => BadgeFormat::Svg,
    }
  }
}

#[derive(Debug, serde::Serialize)]
pub struct Badge {
  pub llabel: Option<String>,
  pub lcolor: Color,
  pub rlabel: String,
  pub rcolor: Color,
  pub icon: Option<String>,
  pub icon_color: Color,
  pub style: BadgeStyle,
  pub radius: u8,
  pub scale: f32,
  pub cache: u32,
  pub format: BadgeFormat,
}

impl Badge {
  pub fn new(label: &str, value: &str, value_color: Color) -> Badge {
    Badge {
      llabel: Some(label.to_string()),
      lcolor: Color::DefaultLabel,
      rlabel: value.to_string(),
      rcolor: value_color,
      icon: None,
      icon_color: Color::Hex("fff".to_string()),
      style: BadgeStyle::Flat,
      radius: 3,
      scale: 1.0,
      cache: DEFAULT_CACHE,
      format: BadgeFormat::Svg,
    }
  }

  pub fn from_qs(qs: &Dict) -> Res<Badge> {
    let llabel = qs.get("label").map(|v| v.to_string());
    let lcolor = qs_first(qs, &["lcolor", "labelColor"])
      .and_then(|x| Color::from_str(&x).ok())
      .unwrap_or(Color::DefaultLabel);

    let rlabel = qs.get("value").unwrap_or(&"unknown".to_string()).to_string();
    let rcolor = qs_first(qs, &["rcolor", "color"])
      .and_then(|x| Color::from_str(&x).ok())
      .unwrap_or(Color::DefaultValue);

    let icon = qs_first(qs, &["icon", "logo"]);
    let icon_color = qs_first(qs, &["iconColor", "logoColor"])
      .and_then(|x| Color::from_str(&x).ok())
      .unwrap_or(Color::Hex("fff".to_string()));

    let style = qs.get("style").map(|x| BadgeStyle::parse(x)).unwrap_or(BadgeStyle::Flat);
    let radius = qs
      .get("radius")
      .and_then(|v| v.parse::<u8>().ok())
      .unwrap_or(if style == BadgeStyle::Flat { 3 } else { 0 })
      .min(12);

    let scale = qs.get("scale").and_then(|x| x.parse::<f32>().ok()).unwrap_or(1.0).clamp(0.1, 8.0);
    let cache = qs_first(qs, &["cache", "cacheSeconds"])
      .and_then(|x| x.parse::<u32>().ok())
      .unwrap_or(DEFAULT_CACHE)
      .clamp(300, DEFAULT_CACHE * 7);

    let format = qs.get("format").map(|x| BadgeFormat::parse(x)).unwrap_or(BadgeFormat::Svg);

    Ok(Badge {
      llabel,
      lcolor,
      rlabel,
      rcolor,
      icon,
      icon_color,
      style,
      radius,
      scale,
      cache,
      format,
    })
  }

  pub fn from_qs_with(qs: &Dict, label: &str, value: &str, value_color: Color) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;
    badge.llabel = if badge.llabel.is_none() { Some(label.to_string()) } else { badge.llabel };
    badge.rlabel = value.to_string();
    badge.rcolor = match badge.rcolor {
      Color::DefaultValue => value_color,
      _ => badge.rcolor,
    };
    Ok(badge)
  }

  pub fn for_version(qs: &Dict, label: &str, value: &str) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    let value = match value.trim() {
      "unknown" | "" => "unknown",
      x if x.starts_with("v") => x,
      x => &format!("v{}", x),
    };

    badge.llabel = if badge.llabel.is_none() { Some(label.to_string()) } else { badge.llabel };
    badge.rlabel = value.to_string();
    badge.rcolor = match badge.rcolor {
      Color::DefaultValue => Color::from_version(value),
      _ => badge.rcolor,
    };

    Ok(badge)
  }

  pub fn for_min_ver(qs: &Dict, label: &str, value: &str) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    badge.llabel = if badge.llabel.is_none() { Some(label.to_string()) } else { badge.llabel };
    badge.rlabel = to_min_ver(value);
    badge.rcolor = match badge.rcolor {
      Color::DefaultValue => Color::Blue,
      _ => badge.rcolor,
    };

    Ok(badge)
  }

  pub fn for_license(qs: &Dict, license: &str) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    badge.llabel = if badge.llabel.is_none() { Some("license".to_string()) } else { badge.llabel };
    badge.rlabel = license.to_string();
    badge.rcolor = match badge.rcolor {
      Color::DefaultValue => Color::Blue,
      _ => badge.rcolor,
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

    badge.llabel =
      if badge.llabel.is_none() { Some("downloads".to_string()) } else { badge.llabel };
    badge.rlabel = value;
    badge.rcolor = match badge.rcolor {
      Color::DefaultValue => Color::Green,
      _ => badge.rcolor,
    };

    Ok(badge)
  }

  pub fn for_count(qs: &Dict, label: &str, value: u64) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    badge.llabel = if badge.llabel.is_none() { Some(label.to_string()) } else { badge.llabel };
    badge.rlabel = millify(value);
    badge.rcolor = match badge.rcolor {
      Color::DefaultValue => Color::Blue,
      _ => badge.rcolor,
    };

    Ok(badge)
  }

  pub fn for_size(qs: &Dict, label: &str, bytes: u64) -> Res<Badge> {
    let mut badge = Badge::from_qs(qs)?;

    badge.llabel = if badge.llabel.is_none() { Some(label.to_string()) } else { badge.llabel };
    badge.rlabel = millify_iec(bytes);
    badge.rcolor = match badge.rcolor {
      Color::DefaultValue => Color::Blue,
      _ => badge.rcolor,
    };

    Ok(badge)
  }

  pub fn to_str(&self) -> String {
    let icon = self.icon.as_deref().unwrap_or_default();
    let icon = get_icon(icon, &self.icon_color.to_css());

    let ltext = self.llabel.clone().map(|s| s.trim().to_string()).unwrap_or_default();
    let rtext = self.rlabel.clone().trim().to_string();
    let (has_text, has_icon) = (!ltext.is_empty(), icon.is_some());

    #[allow(clippy::nonminimal_bool)]
    let mono = (!has_text && !has_icon)
      || (has_icon && !has_text && self.lcolor == Color::DefaultLabel)
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

    let hh = 20.0 * self.scale;
    let ww = w * hh / h;

    let svg = html!(svg xmlns="http://www.w3.org/2000/svg"
      viewBox=(format!("0 0 {} {}", w, h))
      width=(ww) height=(hh)
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
        @if has_text || has_icon { rect x="0" y="0" width=(w) height=(h) fill=(self.lcolor.to_css()) {} }
        rect x=(w-rw) y="0" width=(rw) height=(h) fill=(self.rcolor.to_css()) rx=(0) {}
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
    let cc = format!("public,max-age={0},s-maxage=300,stale-while-revalidate={0}", self.cache);
    match self.format {
      BadgeFormat::Json => {
        let headers = [(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, &cc)];
        let content = serde_json::to_string(&self).unwrap();
        (StatusCode::OK, headers, content).into_response()
      }
      _ => {
        let headers = [(header::CONTENT_TYPE, "image/svg+xml"), (header::CACHE_CONTROL, &cc)];
        (StatusCode::OK, headers, self.to_str()).into_response()
      }
    }
  }
}
