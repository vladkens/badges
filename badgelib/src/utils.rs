use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::Deserialize;
use serde::de::IntoDeserializer;

use super::_width::WIDTHS;
use super::Color;
use crate::_icons::ICONS;

// https://github.com/serde-rs/serde/issues/1425#issuecomment-462282398
// note: "default" should be used with Option<T> to work, example:
// #[serde(deserialize_with = "empty_string_as_none", default)]
pub(crate) fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
  D: serde::Deserializer<'de>,
  T: serde::Deserialize<'de>,
{
  let opt: Option<String> = Option::deserialize(de)?;
  match opt.as_deref() {
    None | Some("") => Ok(None),
    Some(s) => T::deserialize(s.into_deserializer()).map(Some),
  }
}

pub fn cacl_width(text: &str) -> f32 {
  let fallback_width = WIDTHS[64]; // Width as "@" for overflows
  let mut total_width = 0.0;
  for ch in text.chars() {
    let index = ch as usize;
    let width = WIDTHS.get(index).copied().unwrap_or(fallback_width);
    total_width += width;
  }

  total_width
}

// pub fn to_min_ver(version: &str) -> String {
//   version.replace(">=", "≥").replace("<=", "≤")
// }

pub fn millify(n: u64) -> String {
  let mut n = n as f64;
  let mut i = 0;
  let units = ["", "k", "M", "B", "T"];
  while n >= 1_000.0 {
    n /= 1_000.0;
    i += 1;
  }

  let label = format!("{n:.1}");
  let label = label.strip_suffix(".0").unwrap_or(&label);
  let label = format!("{label}{}", units[i]);
  label
}

// https://www.npmjs.com/package/byte-size
pub fn millify_iec(n: u64) -> String {
  let mut n = n as f64;
  let mut i = 0;
  let units = ["", "KiB", "MiB", "GiB", "TiB"];
  while n >= 1_024.0 {
    n /= 1_024.0;
    i += 1;
  }

  let label = format!("{n:.1}");
  let label = label.strip_suffix(".0").unwrap_or(&label);
  let label = format!("{label} {}", units[i]);
  label
}

pub(crate) fn get_icon(name: &str, color: &Option<Color>) -> Option<String> {
  let pretenders = [
    name.to_lowercase(),
    name.to_lowercase().replace('-', "").replace("!", "").replace("_", "").replace(" ", ""),
    name.to_lowercase().replace('.', "dot").replace("+", "plus"),
  ];

  let icon = format!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" role="img" viewBox="0 0 24 24" fill="{}"><path d="{}" /></svg>"#,
    color.clone().map(|x| x.to_css()).unwrap_or("#fff".into()),
    pretenders.iter().find_map(|n| ICONS.get(n))?
  );

  Some(format!("data:image/svg+xml;base64,{}", BASE64_STANDARD.encode(icon)))
}

pub(crate) fn text_color(bg_color: &Color) -> Color {
  let hex = bg_color.to_hex();
  let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
  let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
  let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;

  // Using relative luminance formula
  let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  // println!("Luminance: {}", luminance);
  if luminance > 0.85 { Color::Black } else { Color::White }
}

pub(crate) fn rating_color<T: Into<f64>>(value: T, max_value: T) -> Color {
  let (value, max_value) = (value.into(), max_value.into());

  match value {
    x if x >= max_value * 0.80 => Color::Green,
    x if x >= max_value * 0.60 => Color::YellowGreen,
    x if x >= max_value * 0.40 => Color::Yellow,
    x if x >= max_value * 0.20 => Color::Orange,
    _ => Color::Red,
  }
}
