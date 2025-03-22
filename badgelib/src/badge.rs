use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Color;
#[cfg(feature = "axum")]
use crate::param::Format;
use crate::param::{Period, Style};
use crate::utils::{cacl_width, empty_string_as_none, get_icon, millify, millify_iec, text_color};

#[cfg(feature = "axum")]
fn default_cache() -> u32 {
  86400 // 24 hours
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Badge {
  #[serde(rename = "label")]
  label: Option<String>,

  #[serde(rename = "labelColor", deserialize_with = "empty_string_as_none", default)]
  label_color: Option<Color>,

  #[serde(rename = "value")]
  value: Option<String>,

  #[serde(rename = "color", deserialize_with = "empty_string_as_none", default)]
  value_color: Option<Color>,

  #[serde(rename = "logo", alias = "icon")]
  logo: Option<String>,

  #[serde(rename = "logoColor", alias = "iconColor")]
  #[serde(deserialize_with = "empty_string_as_none", default)]
  logo_color: Option<Color>,

  #[serde(rename = "radius")]
  radius: Option<u8>,

  #[serde(rename = "style", default = "Default::default")]
  style: Style,

  #[cfg(feature = "axum")]
  #[serde(rename = "format", default = "Default::default")]
  format: Format,

  #[cfg(feature = "axum")]
  #[serde(rename = "cache", default = "default_cache")]
  cache: u32,
}

impl Badge {
  pub fn new() -> Self {
    Self::default()
  }

  // MARK: Setters

  pub fn label(mut self, label: &str) -> Self {
    self.label = Some(label.into());
    self
  }

  pub fn label_color(mut self, color: Color) -> Self {
    self.label_color = Some(color);
    self
  }

  pub fn value(mut self, value: &str) -> Self {
    self.value = Some(value.into());
    self
  }

  pub fn value_color(mut self, color: Color) -> Self {
    self.value_color = Some(color);
    self
  }

  pub fn logo(mut self, logo: &str) -> Self {
    self.logo = Some(logo.into());
    self
  }

  pub fn logo_color(mut self, color: Color) -> Self {
    self.logo_color = Some(color);
    self
  }

  pub fn radius(mut self, radius: u8) -> Self {
    self.radius = Some(radius);
    self
  }

  // MARK: Predefined

  pub fn for_version(mut self, label: &str, value: &str) -> Self {
    let value = match value.to_lowercase().trim() {
      "" | "unknown" | "none" => "unknown".into(),
      x if x.starts_with('v') => x.into(),
      x => format!("v{x}"),
    };

    let color = match &value {
      x if x.contains("alpha")
        || x.contains("beta")
        || x.contains("canary")
        || x.contains("rc")
        || x.contains("dev") =>
      {
        Color::Cyan
      }
      x if x.starts_with("v0.") => Color::Orange,
      _ => Color::Blue,
    };

    self.label = self.label.or(Some(label.into()));
    self.value = Some(value);
    self.value_color = self.value_color.or(Some(color));
    self
  }

  pub fn for_license(mut self, license: &str) -> Self {
    self.label = self.label.or(Some("license".into()));
    self.value = Some(license.into());
    self.value_color = self.value_color.or(Some(Color::Blue));
    self
  }

  pub fn for_downloads(mut self, period: Period, value: u64) -> Self {
    let value = match period {
      Period::Week => format!("{}/week", millify(value)),
      Period::Month => format!("{}/month", millify(value)),
      Period::Year => format!("{}/year", millify(value)),
      Period::Total => millify(value),
    };

    self.label = self.label.or(Some("downloads".into()));
    self.value = Some(value);
    self.value_color = self.value_color.or(Some(Color::Green));
    self
  }

  pub fn for_ci_status(mut self, label: &str, status: bool) -> Self {
    let value = if status { "passing" } else { "failing" };
    let color = if status { Color::Green } else { Color::Red };

    self.label = self.label.or(Some(label.into()));
    self.value = Some(value.into());
    self.value_color = Some(color);
    self
  }

  pub fn for_count(mut self, label: &str, value: u64) -> Self {
    self.label = self.label.or(Some(label.into()));
    self.value = Some(millify(value));
    self.value_color = Some(Color::Blue);
    self
  }

  pub fn for_size(mut self, label: &str, value: u64) -> Self {
    self.label = self.label.or(Some(label.into()));
    self.value = Some(millify_iec(value));
    self.value_color = Some(Color::Blue);
    self
  }

  pub fn for_rating(mut self, label: &str, value: f64, max_value: f64) -> Self {
    self.label = self.label.or(Some(label.into()));
    self.value = Some(format!("{:.1}/{}", value, max_value));
    self.value_color = Some(Color::Blue);
    self
  }

  pub fn for_stars(mut self, label: &str, value: f64, max_value: f64) -> Self {
    let value = {
      let scale = max_value / 5.0;
      let score = value / scale;

      // unfortunately not supported yet https://symbl.cc/en/2BE8/
      let full_part = "★".repeat(score as usize);
      let half_part = if score.fract() >= 0.5 { "½" } else { "" };
      let mut line = format!("{}{}", full_part, half_part);

      let size = line.chars().count();
      if size < 5 {
        line.push_str(&"☆".repeat(5 - size));
      }

      line
    };

    self.label = self.label.or(Some(label.into()));
    self.value = Some(value);
    self.value_color = Some(Color::Blue);
    self
  }

  pub fn for_duration(mut self, label: &str, value: DateTime<Utc>) -> Self {
    let days = Utc::now().signed_duration_since(value).num_days();
    let (value, color) = match days {
      0 => ("today".into(), Color::Green),
      1 => ("yesterday".into(), Color::Green),
      2..=7 => (format!("{} days ago", days), Color::Green),
      8..=30 => (format!("{} days ago", days), Color::LightGreen),
      31..=180 => (format!("{} months ago", days / 30), Color::Yellow),
      181..=365 => (format!("{} months ago", days / 30), Color::YellowGreen),
      366..=730 => (format!("{} year ago", days / 365), Color::Orange),
      _ => (format!("{} years ago", days / 365), Color::Red),
    };

    self.label = self.label.or(Some(label.into()));
    self.value = Some(value);
    self.value_color = Some(color); // Changed from Color::Blue to use the calculated color
    self
  }

  // MARK: Render

  pub fn to_json(&self) -> String {
    serde_json::to_string(self).unwrap()
  }

  pub fn to_svg(&self) -> String {
    let icon = get_icon(self.logo.as_deref().unwrap_or_default(), &self.logo_color);

    let ltext = self.label.clone().map(|x| x.trim().to_string()).unwrap_or_default();
    let rtext = self.value.clone().map(|x| x.trim().to_string()).unwrap_or_default();
    let (has_text, has_icon) = (!ltext.is_empty(), icon.is_some());

    #[allow(clippy::nonminimal_bool)]
    let mono = (!has_text && !has_icon)
      || (has_icon && !has_text && self.label_color.is_none())
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
    let (outx, outy) = (fz * 0.1 / 2.0, fz * 0.1);

    let hh = 20.0 * 1.0;
    let ww = w * hh / h;

    let lb_color = self.label_color.clone().unwrap_or(Color::Black);
    let rb_color = self.value_color.clone().unwrap_or(Color::Blue);
    let lt_color = text_color(&lb_color).to_css();
    let rt_color = text_color(&rb_color).to_css();
    let lb_color = lb_color.to_css();
    let rb_color = rb_color.to_css();

    let radius = self.radius.unwrap_or(if self.style == Style::Flat { 3 } else { 0 }).min(12);
    let radius = (fz / 12.0) * radius as f32;

    let svg = maud::html!(svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox=(format!("0 0 {} {}", w, h))
      width=(ww) height=(hh)
      role="img" aria-label=(title)
    {
      title { (title) }

      // background gradient
      @if self.style == Style::Flat {
        linearGradient id="s" x2="0" y2="100%" {
          stop offset="0" stop-opacity=".1" stop-color="#eee" {}
          stop offset="1" stop-opacity=".1" {}
        }
      }

      // border-radius
      mask id="r" { rect width=(w) height=(h) rx=(radius) fill="#fff" {} }

      g mask="url(#r)" {
        @if has_text || has_icon { rect x="0" y="0" width=(w) height=(h) fill=(lb_color) {} }
        rect x=(w-rw) y="0" width=(rw) height=(h) fill=(rb_color) rx=(0) {}
        rect x="0" y="0" width=(w) height=(h) fill="url(#s)" {}
      }

      @if icon.is_some() {
        image x=(pad) y=((h-iw)/2.0) width=(iw) height=(iw) href=(icon.unwrap()) {}
      }

      g font-family="Verdana,Geneva,DejaVu Sans,sans-serif" font-size=(fz)
        text-anchor="start" dominant-baseline="middle" text-rendering="geometricPrecision"
      {
        @if has_text {
          text textLength=(ltw) x=(lx+outx) y=(y+outy) fill="#000" opacity="0.25" { (&ltext) }
          text textLength=(ltw) x=(lx) y=(y) fill=(lt_color) { (&ltext) }
        }
        text textLength=(rtw) x=(rx+outx) y=(y+outy) fill="#000" opacity="0.25" { (&rtext) }
        text textLength=(rtw) x=(rx) y=(y) fill=(rt_color) { (&rtext) }
      }
    });

    svg.into_string()
  }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Badge {
  fn into_response(self) -> axum::response::Response {
    let cc = format!("public,max-age={0},s-maxage=300,stale-while-revalidate={0}", self.cache);
    let (ct, content) = match self.format {
      Format::Svg => ("image/svg+xml", self.to_svg()),
      Format::Json => ("application/json", self.to_json()),
    };

    let rep = (
      axum::http::StatusCode::OK,
      [(axum::http::header::CACHE_CONTROL, cc), (axum::http::header::CONTENT_TYPE, ct.into())],
      content,
    );

    rep.into_response()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_for_version() {
    // Test empty/unknown values
    let rs = Badge::new().for_version("pkg", "");
    assert_eq!(rs.value, Some("unknown".to_string()));
    assert_eq!(rs.label, Some("pkg".to_string()));

    // Test version with v prefix
    let rs = Badge::new().for_version("pkg", "v1.0.0");
    assert_eq!(rs.value, Some("v1.0.0".to_string()));
    assert_eq!(rs.value_color, Some(Color::Blue));

    // Test version without v prefix
    let rs = Badge::new().for_version("pkg", "1.0.0");
    assert_eq!(rs.value, Some("v1.0.0".to_string()));
    assert_eq!(rs.value_color, Some(Color::Blue));

    // Test version without v prefix
    let rs = Badge::new().for_version("pkg", "1.0.0");
    assert_eq!(rs.value, Some("v1.0.0".to_string()));
    assert_eq!(rs.value_color, Some(Color::Blue));

    // Test beta version
    let rs = Badge::new().for_version("pkg", "v1.0.0-beta");
    assert_eq!(rs.value_color, Some(Color::Cyan));

    // Test v0 version
    let rs = Badge::new().for_version("pkg", "v0.1.0");
    assert_eq!(rs.value_color, Some(Color::Orange));
  }
}
