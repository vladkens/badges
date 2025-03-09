use serde::{Deserialize, Deserializer};

// From: https://github.com/badgen/badgen/blob/master/src/color-presets.ts
#[derive(Debug, Default, strum::EnumIter)]
pub enum Color {
  #[default]
  Default,
  Green,
  Blue,
  Red,
  Yellow,
  Orange,
  Purple,
  Pink,
  Grey,
  Cyan,
  Black,
  Hex(String),
}

impl Color {
  pub fn to_hex(&self) -> String {
    match self {
      Color::Default => "08C",
      Color::Green => "3C1",
      Color::Blue => "08C",
      Color::Red => "E43",
      Color::Yellow => "DB1",
      Color::Orange => "F73",
      Color::Purple => "94E",
      Color::Pink => "E5B",
      Color::Grey => "999",
      Color::Cyan => "1BC",
      Color::Black => "2A2A2A",
      Color::Hex(hex) => hex,
    }
    .to_string()
  }

  pub fn to_css(&self) -> String {
    format!("#{}", self.to_hex())
  }

  pub fn to_name(&self) -> Option<String> {
    match self {
      Color::Default => None,
      Color::Green => Some("green".to_string()),
      Color::Blue => Some("blue".to_string()),
      Color::Red => Some("red".to_string()),
      Color::Yellow => Some("yellow".to_string()),
      Color::Orange => Some("orange".to_string()),
      Color::Purple => Some("purple".to_string()),
      Color::Pink => Some("pink".to_string()),
      Color::Grey => Some("grey".to_string()),
      Color::Cyan => Some("cyan".to_string()),
      Color::Black => Some("black".to_string()),
      Color::Hex(_) => None,
    }
  }

  pub fn from_version(v: &str) -> Self {
    if v.contains("alpha")
      || v.contains("beta")
      || v.contains("canary")
      || v.contains("rc")
      || v.contains("dev")
    {
      return Color::Cyan;
    }

    if v.starts_with("0.") || v.starts_with("v0.") {
      return Color::Orange;
    }

    Color::Blue
  }

  // "impl FromStr for Color" are stupid becase require to import "std::str::FromStr" everywhere
  pub fn from_str(s: &str) -> Result<Self, &'static str> {
    match s.to_lowercase().as_ref() {
      "green" => Ok(Color::Green),
      "blue" => Ok(Color::Blue),
      "red" => Ok(Color::Red),
      "yellow" => Ok(Color::Yellow),
      "orange" => Ok(Color::Orange),
      "purple" => Ok(Color::Purple),
      "pink" => Ok(Color::Pink),
      "grey" => Ok(Color::Grey),
      "cyan" => Ok(Color::Cyan),
      "black" => Ok(Color::Black),
      x => {
        if (x.len() == 3 || x.len() == 6) && x.chars().all(|c| c.is_ascii_hexdigit()) {
          Ok(Color::Hex(x.to_string()))
        } else {
          Err("invalid color")
        }
      }
    }
  }
}

impl<'de> Deserialize<'de> for Color {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;

    match s.to_lowercase().as_str() {
      "default" => Ok(Color::Default),
      "green" => Ok(Color::Green),
      "blue" => Ok(Color::Blue),
      "red" => Ok(Color::Red),
      "yellow" => Ok(Color::Yellow),
      "orange" => Ok(Color::Orange),
      "purple" => Ok(Color::Purple),
      "pink" => Ok(Color::Pink),
      "grey" | "gray" => Ok(Color::Grey), // Supporting aliases
      "cyan" => Ok(Color::Cyan),
      "black" => Ok(Color::Black),
      _ if s.starts_with('#') || s.len() == 6 || s.len() == 8 => Ok(Color::Hex(s)),
      _ => Err(serde::de::Error::custom(format!("Invalid color: {}", s))),
    }
  }
}
