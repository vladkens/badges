use serde::{Deserialize, Deserializer, Serialize};

pub enum Period {
  Week,
  Month,
  Year,
  Total,
}

// MARK: Color

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Color {
  #[default]
  Blue,
  Green,
  YellowGreen,
  Yellow,
  Orange,
  Red,
  Gray,
  Black,
  White,
  Cyan,
  Hex(String),
}

impl Color {
  pub fn to_hex(&self) -> String {
    match self {
      Color::Blue => "1e88e5".into(),
      Color::Green => "2ecc71".into(),
      Color::YellowGreen => "a3e048".into(),
      Color::Yellow => "f9a825".into(),
      Color::Orange => "ff7043".into(),
      Color::Red => "e53935".into(),
      Color::Gray => "607d8b".into(),
      Color::Black => "1c1c1e".into(),
      Color::White => "f5f5f5".into(),
      Color::Cyan => "00acc1".into(),
      Color::Hex(hex) => {
        if hex.len() == 3 {
          hex.chars().map(|c| c.to_string().repeat(2)).collect()
        } else {
          hex.clone()
        }
      }
    }
  }

  pub fn to_css(&self) -> String {
    format!("#{}", self.to_hex())
  }
}

impl<'a> TryFrom<&'a str> for Color {
  type Error = String;

  fn try_from(value: &'a str) -> Result<Self, Self::Error> {
    // https://github.com/badges/shields/blob/master/badge-maker/lib/color.js
    match value.to_lowercase().trim().replace("#", "").as_ref() {
      "blue" => Ok(Color::Blue),
      "green" => Ok(Color::Green),
      "yellowgreen" => Ok(Color::YellowGreen),
      "yellow" => Ok(Color::Yellow),
      "orange" => Ok(Color::Orange),
      "red" => Ok(Color::Red),
      "grey" => Ok(Color::Gray),
      "black" => Ok(Color::Black),
      "white" => Ok(Color::White),
      "cyan" => Ok(Color::Cyan),
      x => {
        if (x.len() == 3 || x.len() == 6) && x.chars().all(|c| c.is_ascii_hexdigit()) {
          Ok(Color::Hex(x.to_string()))
        } else {
          Err(format!("invalid color '{}'", x))
        }
      }
    }
  }
}

impl<'de> Deserialize<'de> for Color {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let s = String::deserialize(deserializer)?;
    Self::try_from(s.as_ref()).map_err(serde::de::Error::custom)
  }
}

impl Serialize for Color {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_hex())
  }
}

// MARK: Style

#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Style {
  #[default]
  Flat,
  FlatSquare,
}

impl<'a> TryFrom<&'a str> for Style {
  type Error = String;

  fn try_from(value: &'a str) -> Result<Self, Self::Error> {
    match value.to_lowercase().replace("-", "").trim() {
      "flat" => Ok(Style::Flat),
      "flatsquare" => Ok(Style::FlatSquare),
      x => Err(format!("unknown badge style '{}'", x)),
    }
  }
}

impl<'de> Deserialize<'de> for Style {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let s = String::deserialize(deserializer)?;
    Self::try_from(s.as_ref()).map_err(serde::de::Error::custom)
  }
}

// MARK: Format

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Format {
  #[default]
  Svg,
  Json,
}

impl<'a> TryFrom<&'a str> for Format {
  type Error = String;

  fn try_from(value: &'a str) -> Result<Self, Self::Error> {
    match value.to_lowercase().trim() {
      "svg" => Ok(Format::Svg),
      "json" => Ok(Format::Json),
      x => Err(format!("unknown badge format '{}'", x)),
    }
  }
}

impl<'de> Deserialize<'de> for Format {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let s = String::deserialize(deserializer)?;
    Self::try_from(s.as_ref()).map_err(serde::de::Error::custom)
  }
}
