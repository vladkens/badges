# badgelib

A Rust library for generating customizable SVG badges, similar to shields.io.

## Features

- Generate SVG badges with custom text, colors, and icons
- Built-in width calculation for proper badge sizing
- Support for Simple Icons
- Optional Axum web framework integration
- Flexible badge styling options
- Built-in formatters for common badge types:
  - Version badges
  - Download count badges  
  - Rating badges
  - Star count badges
  - Duration/age badges
  - License badges

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
badgelib = "0.1.0"
```

To enable Axum integration, add the `axum` feature:

```toml 
badgelib = { version = "0.1.0", features = ["axum"] }
```

## Usage

```rust
use badgelib::Badge;

// Create a simple version badge
let badge = Badge::default().for_version("version", "1.0.0");

// Generate SVG
let svg = badge.to_svg();

// Or generate JSON
let json = badge.to_json();
```

### Customization

Badges can be customized with:

- Custom label and value text
- Label and value colors
- Icons (using Simple Icons)
- Icon colors  
- Border radius
- Different styles (flat, flat-square, etc)

```rust
use badgelib::{Badge, Color};

let badge = Badge::default()
  .label("downloads")
  .value("1.2k")
  .label_color(Color::Blue)
  .value_color(Color::Green)
  .logo("rust")
  .logo_color(Color::Red)
  .with_radius(8)
  .to_svg();
```

### Using with Axum

When enabled with the `axum` feature, badges can be used as responses in Axum handlers:

```rust
async fn badge_handler() -> impl IntoResponse {
  Badge::default().for_version("version", "1.0.0")
}
```

### Built-in Formatters

The library includes several built-in formatters for common badge types:

```rust
use badgelib::{Badge, Period};
use chrono::{DateTime, Utc};

// Version badge (automatically adds v prefix and colors)
let version = Badge::default().for_version("version", "1.0.0");  // Shows as "v1.0.0"

// License badge
let license = Badge::default().for_license("MIT");

// Download count badge with period and automatic formatting
let downloads = Badge::default().for_downloads(Period::Month, 1234567);  // Shows as "1.2M/month"

// CI status badge (green/red)
let ci = Badge::default().for_ci_status("build", true);  // Shows as "build passing"

// Generic count badge with automatic formatting
let count = Badge::default().for_count("stars", 1234);  // Shows as "1.2k"

// Size badge with IEC formatting
let size = Badge::default().for_size("size", 1234567);  // Shows as "1.2 MiB"

// Rating badge (0-5 scale)
let rating = Badge::default().for_rating("rating", 4.5, 5.0);  // Shows as "4.5/5"

// Star rating badge with visual stars
let stars = Badge::default().for_stars("rating", 4.5, 5.0);  // Shows as "★★★★½"

// Duration/age badge with automatic formatting and colors
let date = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap();
let age = Badge::default().for_duration("updated", date.into());  // Shows as "1 year ago"
```

Each formatter provides:
- Automatic value formatting (numbers, dates, etc)
- Contextual colors based on values
- Sensible defaults for labels
- Built-in data visualization where appropriate

## License

[MIT License](../LICENSE)
