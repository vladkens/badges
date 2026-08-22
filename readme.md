# badges.ws · Elegant badges for your standout projects

<picture><img src="assets/logo.svg" align="right" width="128" alt="badges.ws logo" /></picture>

[<img src="https://badges.ws/maintenance/yes/2026" alt="Maintained in 2026" />](https://github.com/vladkens/badges)
[<img src="https://badges.ws/badge/PRs-welcome-brightgreen?logo=github" alt="Pull requests welcome" />](https://github.com/vladkens/badges/pulls)
[<img src="https://badges.ws/github/license/vladkens/badges" alt="MIT license" />](https://github.com/vladkens/badges/blob/main/LICENSE)
[<img src="https://badges.ws/badge/Built_with_Rust-000000?logo=rust" alt="Built with Rust" />](https://www.rust-lang.org)
[<img src="https://badges.ws/badge/Hosted_on_Fly.io-24175B?logo=flydotio" alt="Hosted on Fly.io" />](https://fly.io)
[<img src="https://badges.ws/badge/Buy_Me_a_Coffee-ff813f?logo=buymeacoffee" alt="Buy Me a Coffee" />](https://buymeacoffee.com/vladkens)

Badges.ws is the perfect way to add a touch of elegance to your GitHub repositories, documentation, or any project that needs a badge with personality. Designed to be **fast**, **lightweight**, and **effortlessly stylish**, Badges.ws lets you generate custom SVG badges in seconds.

## Why Badges.ws?

- ✅ **Instant Integration** – Generate badges for **npm, PyPI, GitHub**, and 20+ other platforms in seconds.
- 🎨 **Pixel-Perfect & Customizable** – Choose **colors, icons, styles, and animations** to match your brand.
- 🔄 **Always Fresh** – Live integrations ensure your badges **never show stale data**.
- ⚡ **Blazing-Fast** – Built with **Rust** for **lightweight performance** and **minimal server footprint**.
- 🔧 **Self-Hosting Freedom** – Deploy with a single Docker command for **full control** over your infrastructure.

## Showcase

### Package Information

<picture><img src="https://badges.ws/npm/v/react?color=cb3837&logo=npm" alt="React npm version" /></picture>
<picture><img src="https://badges.ws/pypi/v/requests?color=3775a9&logo=pypi" alt="Requests PyPI version" /></picture>
<picture><img src="https://badges.ws/crates/v/tokio?color=f74d02&logo=rust" alt="Tokio crates.io version" /></picture>
<picture><img src="https://badges.ws/gem/v/rails?color=cc342d&logo=rubygems" alt="Rails RubyGems version" /></picture>
<picture><img src="https://badges.ws/packagist/v/laravel/laravel?color=f28d1a&logo=packagist" alt="Laravel Packagist version" /></picture>

### GitHub Insights

<picture><img src="https://badges.ws/github/stars/facebook/react?logo=github" alt="React GitHub stars" /></picture>
<picture><img src="https://badges.ws/github/release/facebook/react" alt="React GitHub release" /></picture>
<picture><img src="https://badges.ws/github/license/facebook/react" alt="React license" /></picture>

### Marketplaces

<picture><img src="https://badges.ws/homebrew/v/node?color=orange&logo=homebrew" alt="Node.js Homebrew version" /></picture>
<picture><img src="https://badges.ws/vscode/v/ms-python.python?color=blue&logo=python" alt="Python VS Code extension version" /></picture>
<picture><img src="https://badges.ws/cws/v/ckkdlimhmcjmikdlpkmbgfkaikojcbjk?logo=chromewebstore" alt="Chrome Web Store extension version" /></picture>

### Social

<picture><img src="https://badges.ws/badge/Gmail-EA4335?logo=gmail" alt="Gmail badge" /></picture>
<picture><img src="https://badges.ws/badge/Telegram-26A5E4?logo=telegram" alt="Telegram badge" /></picture>
<picture><img src="https://badges.ws/badge/X-000000?logo=x" alt="X badge" /></picture>
<picture><img src="https://badges.ws/badge/Discord-5865F2?logo=discord" alt="Discord badge" /></picture>
<picture><img src="https://badges.ws/badge/Reddit-FF4500?logo=reddit" alt="Reddit badge" /></picture>
<picture><img src="https://badges.ws/badge/YouTube-FF0000?logo=youtube" alt="YouTube badge" /></picture>
<picture><img src="https://badges.ws/badge/Twitch-9146FF?logo=twitch" alt="Twitch badge" /></picture>

### Styles & Effects

Choose `flat`, `flat-square`, or `for-the-badge`, then tune the radius or add an animation:

<picture><img src="https://badges.ws/badge/style-flat-3b82f6?style=flat" alt="Flat badge style" /></picture>
<picture><img src="https://badges.ws/badge/style-flat--square-3b82f6?style=flat-square" alt="Flat-square badge style" /></picture>
<picture><img src="https://badges.ws/badge/style-for--the--badge-3b82f6?style=for-the-badge" alt="For-the-badge style" /></picture>
<picture><img src="https://badges.ws/badge/release-ready-16a34a?animation=shine" alt="Badge with shine animation" /></picture>
<picture><img src="https://badges.ws/badge/northern_lights-3b82f6?animation=aurora" alt="Badge with aurora animation" /></picture>

`shine` sweeps a soft highlight across the badge, while `aurora` adds a slowly drifting field of color. The rendering library also provides `flow` for animated gradients, with all animations respecting reduced-motion preferences.

### Presets

<picture><img src="https://badges.ws/handmade" alt="Handmade badge preset" /></picture>
<picture><img src="https://badges.ws/vibecoded" alt="Vibe Coded badge preset" /></picture>

Presets combine custom artwork and effects into stable URLs. `/handmade` uses a custom SVG icon with Shine; `/vibecoded` combines a multicolor icon, gradient, and Aurora animation.

### Icons

Use `logo` and `logoColor` to add and customize a name from [Simple Icons](https://simpleicons.org). Lookups are case-insensitive, common separators may be omitted, and removed or renamed slugs from supported historical releases remain available. Names such as `GitHub Actions`, `C++`, and `.NET` work directly when URL-encoded:

<picture><img src="https://badges.ws/badge/GitHub_Actions-181717?logo=GitHub%20Actions" alt="GitHub Actions icon" /></picture>
<picture><img src="https://badges.ws/badge/C%2B%2B-00599C?logo=C%2B%2B" alt="C++ icon" /></picture>
<picture><img src="https://badges.ws/badge/.NET-512BD4?logo=.NET" alt=".NET icon" /></picture>

Logos follow the text color by default; set `logoColor` only when you need an explicit override. The legacy `icon` and `iconColor` names remain accepted as aliases.

### More Options

Visit [badges.ws](https://badges.ws) to explore!

## Get Started in 10 Seconds

Creating your custom badge is as easy as plugging values into a URL:

```text
https://badges.ws/badge/{label}-{message}-{color}
```

Replace `{label}`, `{message}`, and `{color}` with your desired text and color code, and your badge is ready to shine! Embed it in your `readme.md`:

```markdown
[<img src="https://badges.ws/badge/Version-1.0.0-red" />](https://your-project.link)
```

### Options

| Parameter | Description |
| --- | --- |
| `label` | Text shown on the left side |
| `labelColor` | Background color for the left side |
| `value` | Text shown on the right side |
| `color` | Background color for the right side |
| `logo` | Simple Icons name or slug |
| `logoColor` | Explicit logo color; defaults to the text color |
| `style` | `flat`, `flat-square`, or `for-the-badge` |
| `radius` | Border radius from `0` to `12` pixels |
| `animation` | `flow`, `shine`, or `aurora` |
| `format` | `svg` or `json` |
| `cache` | Cache lifetime in seconds |

**Or self-host your badge service:**

```sh
docker run -d -p 8080:8080 ghcr.io/vladkens/badges:main
```

### GitHub API Authentication

GitHub badges work without authentication, but a token increases the API rate limit. Create a [fine-grained personal access token](https://github.com/settings/personal-access-tokens/new) and leave additional permissions disabled—the service only reads public data. Pass one or more comma-separated tokens through `GH_TOKENS`:

```dotenv
GH_TOKENS=github_pat_TOKEN_1,github_pat_TOKEN_2
```

For Docker:

```sh
docker run -d -p 8080:8080 \
  -e GH_TOKENS='github_pat_TOKEN_1,github_pat_TOKEN_2' \
  ghcr.io/vladkens/badges:main
```

## Live Integrations

- **Languages & Packages**: `JavaScript / TypeScript (npm)`, `Python (PyPI)`, `Rust (Crates.io)`, `Ruby (RubyGems)`, `PHP (Packagist)`, `Dart (Pub)`, `Haskell (Hackage)`, `C# / F# (NuGet)`, `Swift / Objective-C (CocoaPods)`, `Clojure (Clojars)`, `Elixir (Hex)`, `Puppet Forge`, `Perl (CPAN)`, `Package Phobia`
- **Marketplaces**: `Homebrew and Casks`, `VS Code Marketplace`, `Chrome Web Store`, `Firefox Add-ons`, `JetBrains Plugins`
- **Services & CI/CD**: `GitHub and GitHub Actions`, `Docker Hub`, `Codecov`, `Read the Docs`
- **Community**: `Discord`, `YouTube`

## Contribute

**Missing an integration?** Request or contribute — let’s build the ultimate badge toolkit together!

## Credits & Inspiration

- Badge rendering is powered by [badgelib](https://github.com/vladkens/badgelib).
- This project was inspired by [Shields.io](https://github.com/badges/shields) and [Badgen.net](https://github.com/badgen/badgen.net).
- Icons are provided by the amazing [Simple Icons](https://simpleicons.org) project.
- The font used for badge rendering is [DejaVu Sans](https://dejavu-fonts.github.io), an open-source font family designed for high-quality text rendering.

## License

Distributed under the [MIT License](/LICENSE).
