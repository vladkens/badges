use super::_width::WIDTHS;

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

pub fn to_ver_label(verions: Vec<String>) -> String {
  if verions.len() == 1 {
    return verions[0].clone();
  }

  if verions.len() == 2 {
    return format!("{} | {}", verions[0], verions[1]);
  }

  format!("{} – {}", verions.first().unwrap(), verions.last().unwrap())
}

pub fn to_min_ver(version: &str) -> String {
  version.replace(">=", "≥").replace("<=", "≤")
}

pub fn render_stars(score: f64, max_score: f64) -> String {
  let scale = max_score / 5.0;
  let score = score / scale;

  // unfortunately, not supported yet https://symbl.cc/en/2BE8/
  let full_part = "★".repeat(score as usize);
  let half_part = if score.fract() >= 0.5 { "½" } else { "" };
  let mut line = format!("{}{}", full_part, half_part);

  let size = line.chars().count();
  if size < 5 {
    line.push_str(&"☆".repeat(5 - size));
  }

  line
}

pub fn millify(n: u64) -> String {
  let mut n = n as f64;
  let mut i = 0;
  let units = ["", "K", "M", "B", "T"];
  while n >= 1_000.0 {
    n /= 1_000.0;
    i += 1;
  }

  let label = format!("{n:.1}");
  let label = label.strip_suffix(".0").unwrap_or(&label);
  let label = format!("{label}{}", units[i]);
  label
}
