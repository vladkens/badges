# badges.ws

<img src="assets/favicon.svg" align="right" width="128" />

[<img src="https://badges.ws/github/license/vladkens/badges" />](https://github.com/vladkens/badges/blob/main/LICENSE)
[<img src="https://badges.ws/badge/Powered_by_Fly.io-24175B?logo=flydotio&logoColor=fff" />](https://fly.io)
[<img src="https://badges.ws/badge/-/buy%20me%20a%20coffee/ff813f?icon=buymeacoffee&label" alt="donate" />](https://buymeacoffee.com/vladkens)

Badges.ws is a modern, Rust-based badge generation service inspired by [Shields.io](https://github.com/badges/shields) and [Badgen.net](https://github.com/badgen/badgen.net). It offers a simpler, more efficient codebase with up-to-date integrations, self-hosting capabilities, and low memory consumption. Additionally, it incorporates API-level data caching on top of control-cache headers.

## Why Badges.ws?

- 🚀 **Rust-based**: Modern, efficient, and safe codebase.
- 🆕 **Newer Codebase**: Simplified and more maintainable.
- 🔄 **Up-to-date Integrations**: Supports the latest platforms and services.
- 🏠 **Self-hosted**: Easily deployable on your own infrastructure.
- 💾 **Low Memory Usage**: Optimized for minimal resource consumption.
- 📦 **API Caching**: Reduces load and improves performance.

## Integrations

### Languages & Package Managers

- [x] JavaScript / TypeScript (npm) + Packagephobia
- [x] Python (PyPI)
- [x] Rust (Cargo)
- [x] Ruby (RubyGems)
- [x] PHP (Packagist)
- [x] Dart (Pub)
- [x] Haskell (Cabal)
- [x] C# / F# (NuGet)
- [ ] Clojure (Clojars)
- [ ] Elixir (Hex)

### Platforms & Ecosystems

- [x] Homebrew (macOS/Linux package manager)
- [x] VSCode Marketplace (VS Code extensions)
- [x] Chrome Web Store (Chrome extensions)
- [x] Firefox Add-ons (Firefox extensions)
- [x] JetBrains Plugins

### Services

- [x] GitHub
- [ ] GitLab
- [ ] Docker Hub

## Usage

To generate a badge, use the following URL format:

```sh
https://badges.ws/badge/{label}-{message}-{color}
```

For more examples and options, visit [badges.ws](https://badges.ws).

## Self-hosted Solution

Badges.ws can be easily self-hosted using Docker. Run the following command:

```sh
docker run -d -p 8080:80 ghcr.io/vladkens/badges:latest
```

## Contributing

PRs and issues are welcome! Feel free to request new endpoints, services, or features.

## License

This project is licensed under the MIT License. See the [LICENSE](/LICENSE) file for details.

---

Made with 🍆 by the Badges.ws team.
