use serde::{Deserialize, Deserializer, Serialize};

pub enum Period {
  Week,
  Month,
  Year,
  Total,
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

// MARK: Color

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Color {
  Green,
  LightGreen,
  Yellow,
  YellowGreen,
  Orange,
  Red,
  #[default]
  Blue,
  Grey,
  LightGrey,
  Black,
  White,
  Cyan,
  Hex(String),
}

impl Color {
  pub fn to_hex(&self) -> String {
    match self {
      Color::Green => "00873F".into(),
      Color::LightGreen => "32CD32".into(),
      Color::Yellow => "FFD100".into(),
      Color::YellowGreen => "8BC34A".into(),
      Color::Orange => "FF7D00".into(),
      Color::Red => "D32F2F".into(),
      Color::Blue => "1976D2".into(),
      Color::Grey => "607D8B".into(),
      Color::LightGrey => "90A4AE".into(),
      Color::Black => "212121".into(),
      Color::White => "FFFFFF".into(),
      Color::Cyan => "0097A7".into(),
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
    match value.to_lowercase().trim().replace("#", "").as_ref() {
      "green" => Ok(Color::Green),
      "lightgreen" => Ok(Color::LightGreen),
      "yellow" => Ok(Color::Yellow),
      "yellowgreen" => Ok(Color::YellowGreen),
      "orange" => Ok(Color::Orange),
      "red" => Ok(Color::Red),
      "blue" => Ok(Color::Blue),
      "grey" | "gray" => Ok(Color::Grey),
      "lightgrey" => Ok(Color::LightGrey),
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
