use axum::response::IntoResponse;
use chrono::Datelike;
use maud::{Markup, html};
use serde_variant::to_variant_name;
use strum::IntoEnumIterator;

use crate::{apis, badgelib::Color, server::AnyRep};

const DEFAULT_TITLE: &str = "badges.ws";

// MARK: Components

#[allow(unreachable_code)]
fn heading(tag: u8, name: &str) -> Markup {
  let anchor = name.to_lowercase().replace(" ", "-").replace(".", "");

  let cls = "heading";
  let dat = html! {
    (name)
    a href=(format!("#{}", anchor)) id=(anchor) tabindex="-1" class="secondary" { "#" }
  };

  html! {
    @match tag {
      1 => h1 class=(cls) { (dat) },
      2 => h2 class=(cls) { (dat) },
      3 => h3 class=(cls) { (dat) },
      4 => h4 class=(cls) { (dat) },
      5 => h5 class=(cls) { (dat) },
      6 => h6 class=(cls) { (dat) },
      _ => (unreachable!())
    }
  }
}

fn render_tbox<T: Into<String>>(name: &str, items: Vec<(T, T)>) -> maud::Markup {
  let items: Vec<(String, String)> = items.into_iter().map(|(a, b)| (a.into(), b.into())).collect();

  html!({
    section {
      (heading(4, name))

      table class="striped" {
        tbody {
          @for (path, desc) in items {
            tr {
              td class="w-1/4 py-1" { (desc) }
              td class="w-2/4 py-1" { code { (path) } }
              td class="w-1/4 py-1" { img class="h20" src=(path) alt=(desc) {} }
            }
          }
        }
      }
    }
  })
}

fn render_enum<T: IntoEnumIterator + std::fmt::Display + serde::Serialize>(
  name: &str,
  path: &str,
) -> maud::Markup {
  let items: Vec<(String, String)> = T::iter()
    .map(|x| {
      let path = path.replace("{}", to_variant_name(&x).unwrap());
      let desc = x.to_string();
      (path, desc)
    })
    .collect();

  render_tbox(name, items)
}

// MARK: Layouts

fn layout(title: Option<&str>, node: Markup) -> Markup {
  let title = match title {
    Some(title) => format!("{title} · {DEFAULT_TITLE}"),
    None => format!("{DEFAULT_TITLE} · Elegant badges for your standout projects"),
  };
  let descr = "Generate beautiful, fast and lightweight badges for your GitHub repos, documentation and projects. Supports npm, PyPI, GitHub and 15+ other platforms.";
  let build = env!("CARGO_PKG_VERSION");

  let head = html! {
    meta charset="utf-8" {}
    meta name="viewport" content="width=device-width, initial-scale=1" {}

    // Primary Meta Tags
    title { (title) }
    meta name="title" content=(title) {}
    meta name="description" content=(descr) {}

    // Open Graph / X Meta Tags
    meta property="og:type" content="website" {}
    meta property="og:url" content="https://badges.ws/" {}
    meta property="og:title" content=(title) {}
    meta property="og:description" content=(descr) {}
    meta property="og:image" content="/assets/social-preview.png" {}
    meta property="twitter:card" content="summary_large_image" {}
    meta property="twitter:url" content="https://badges.ws/" {}
    meta property="twitter:title" content=(title) {}
    meta property="twitter:description" content=(descr) {}
    meta property="twitter:image" content="/assets/social-preview.png" {}
    link rel="icon" type="image/svg+xml" href="/assets/logo.svg" {}
    // link rel="icon" type="image/png" href="/assets/favicon.png" {}
    // link rel="apple-touch-icon" href="/assets/apple-touch-icon.png" {}

    // Additional Meta Tags
    meta name="theme-color" content="#ffffff" {}
    meta name="robots" content="index, follow" {}
    meta name="google" content="notranslate" {}
    meta name="keywords" content="badges, github badges, npm badges, pypi badges, shields, markdown badges, readme badges, repository badges" {}
    meta name="author" content="badges.ws team" {}
    link rel="canonical" href="https://badges.ws/" {}

    // link rel="preconnect" href="https://unpkg.com" {}
    // link rel="stylesheet" href="https://unpkg.com/@picocss/pico@2/css/pico.min.css" {}
    link rel="preconnect" href="https://cdn.jsdelivr.net" {}
    link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css" {}
    script src="https://cdn.jsdelivr.net/npm/htmx.org@2/dist/htmx.min.js" {}
    script defer src="https://tinylytics.app/embed/5SK2Zy9T5MG5pGdyEwTY.js" {}

    link rel="stylesheet" href=(format!("/assets/main.css?{build}")) {}
  };

  let body = html! {
    nav {
      ul {
        li {
          a href="/" class="contrast no-underline" style="font-size: 2rem; font-weight: 700; line-height: 1" {
            img src="/assets/logo.svg" style="height: 50px; margin-right: 8px; margin-top: -4px" {}
            span { "badges.ws" }
          }
        }
      }

      ul {
        // li {
        //   a href="/debug" class="contrast" { "Debug" }
        // }

        li {
          a target="_blank" href="https://github.com/vladkens/badges" {
            button class="outline secondary" {
              img src="https://cdn.simpleicons.org/github/black/white"
                style="width: 24px; height: 24px; margin-top: -5px; margin-right: 8px;" {}

              "View on GitHub"
            }
          }
        }
      }
    }

    main { (node) }

    footer class="text-center" {
      hr {}

      div style="display: flex; justify-content: center; align-items: center; gap: 16px; padding-bottom: 10px" {
        a target="_blank" href="https://startupfa.me/s/badgesws?utm_source=badges.ws" {
          img src="https://startupfa.me/badges/featured-badge.webp" alt="Featured on Startup Fame" width="171" height="54" {}
        }

        a target="_blank" href="https://twelve.tools?utm_source=badges.ws" {
          img src="https://twelve.tools/badge0-light.svg" alt="Featured on Twelve Tools" width="200" height="54" {}
        }
      }

      small { "© " (chrono::Local::now().year()) " · Made by " a target="_blank" href="https://vladkens.cc" { "Badges.ws" } " team." }
    }
  };

  html! {
    (maud::DOCTYPE)
    html {
      head { (head) }
      body class="container" { (body) }
    }
  }
}

// MARK: Pages

pub async fn debug() -> AnyRep<impl IntoResponse> {
  let items = [
    "/badge/Value-red",
    "/badge/Value-Value-red",
    "/badge/Value-Value-red?labelColor=red",
    "/badge/Value-Value-07C160?logo=wechat&logoColor=white",
    "/badge/Value-07C160?logo=wechat&logoColor=white",
    "/badge/Value-07C160?logo=wechat&labelColor=red&logoColor=white",
    "/badge/-07C160?logo=wechat&logoColor=white",
    "/badge/--07C160?logo=wechat&logoColor=white",
    "/badge/-07C160?logo=wechat&logoColor=white&label=Label",
    "/badge/-07C160?logo=wechat&logoColor=red",
    "/badge/-07C160?logo=wechat&logoColor=blue&labelColor=red",
    "/badge/-red?logo=wechat&logoColor=blue&labelColor=red",
    "/badge/-red?logo=wechat&logoColor=blue",
    "/badge/Buy_me_a_coffee-ff813f?logo=buymeacoffee&logoColor=white",
    "/badge/Buy_me_a_coffee-ff813f?logo=buymeacoffee&logoColor=white&label=12&status=13",
    "/pypi/dm/twscrape",
    "/badge/%20%20%F0%9F%93%A6%F0%9F%9A%80-semantic--release-e10079",
    "/badge/Open_in_DevExpress-FF7200?style=flat-square&logo=DevExpress&logoColor=white",
    "/badge/-JavaScript-F7DF1E?style=flat&logo=javascript&logoColor=black",
    "/badge/chatGPT-74aa9c?style=for-the-badge&logo=openai&logoColor=white",
    // https://github.com/henriquesebastiao/badges
    "/badge/GitHub-181717?style=flat&logo=github&logoColor=white",
    "/badge/GitHub-100000?style=flat&logo=github&logoColor=white",
    "/badge/VSCodium-2F80ED?style=flat&logo=VSCodium&logoColor=white",
    "/badge/Markdown-000000?style=flat&logo=markdown&logoColor=white",
    "/badge/Markdown-ffffff?style=flat&logo=markdown&logoColor=black",
    "/badge/Gmail-EA4335?style=flat&logo=gmail&logoColor=white",
    "/badge/Messenger-00B2FF?style=flat&logo=messenger&logoColor=white",
    "/badge/Telegram-26A5E4?style=flat&logo=telegram&logoColor=white",
    "/badge/WeChat-07C160?style=flat&logo=wechat&logoColor=white",
    "/badge/WhatsApp-25D366?style=flat&logo=whatsapp&logoColor=white",
    "/badge/Discord-5865F2?style=flat&logo=discord&logoColor=white",
    "/badge/Slack-4A154B?style=flat&logo=slack&logoColor=white",
    "/badge/Teams-6264A7?style=flat&logo=microsoft-teams&logoColor=white",
    "/badge/Zoom-0B5CFF?style=flat&logo=zoom&logoColor=white",
    "/badge/-Behance-1769FF?style=flat&logo=behance&logoColor=white",
    "/badge/Bitbucket-0052CC?style=flat&logo=bitbucket&logoColor=white",
    "/badge/Codeforces-1F8ACB?style=flat&logo=Codeforces&logoColor=white",
    "/badge/Codepen-000000?style=flat&logo=codepen&logoColor=white",
    "/badge/Codewars-B1361E?style=flat&logo=Codewars&logoColor=white",
    "/badge/Dribbble-EA4C89?style=flat&logo=dribbble&logoColor=white",
    "/badge/Facebook-0866FF?style=flat&logo=facebook&logoColor=white",
    "/badge/GitLab-FC6D26?style=flat&logo=gitlab&logoColor=white",
    "/badge/-Hackerrank-00EA64?style=flat&logo=HackerRank&logoColor=white",
    "/badge/-LeetCode-FFA116?style=flat&logo=LeetCode&logoColor=black",
    "/badge/Instagram-E4405F?style=flat&logo=instagram&logoColor=white",
    "/badge/LinkedIn-0A66C2?style=flat&logo=linkedin&logoColor=white",
    "/badge/linktree-43E55E?style=flat&logo=linktree&logoColor=white",
    "/badge/Mastodon-6364FF?style=flat&logo=mastodon&logoColor=white",
    "/badge/Pinterest-BD081C?style=flat&logo=pinterest&logoColor=white",
    "/badge/Quora-B92B27?style=flat&logo=quora&logoColor=white",
    "/badge/Reddit-FF4500?style=flat&logo=reddit&logoColor=white",
    "/badge/Roadmap-000000?style=flat&logo=roadmap.sh&logoColor=white",
    "/badge/Signal-3A76F0?style=flat&logo=signal&logoColor=white",
    "/badge/Snapchat-FFFC00?style=flat&logo=snapchat&logoColor=black",
    "/badge/Sourcetree-0052CC?style=flat&logo=Sourcetree&logoColor=white",
    "/badge/Stack_Overflow-F58025?style=flat&logo=stack-overflow&logoColor=white",
    "/badge/TikTok-000000?style=flat&logo=tiktok&logoColor=white",
    "/badge/Tumblr-36465D?style=flat&logo=Tumblr&logoColor=white",
    "/badge/Twitter-1D9BF0?style=flat&logo=twitter&logoColor=white",
    "/badge/Twitter-000000?style=flat&logo=x&logoColor=white",
    "/badge/X-000000?style=flat&logo=x&logoColor=white",
    "/badge/xda%20developers-EA7100?style=flat&logo=xda-developers&logoColor=white",
    "/badge/App_Store-0D96F6?style=flat&logo=app-store&logoColor=white",
    "/badge/F%20Droid-1976D2?style=flat&logo=f-droid&logoColor=white",
    "/badge/Google_Play-414141?style=flat&logo=google-play&logoColor=white",
    "/badge/Flathub-000000?style=flat&logo=flathub&logoColor=white",
    "/badge/Alpine_Linux-0D597F?style=flat&logo=alpine-linux&logoColor=white",
    "/badge/Android-34A853?style=flat&logo=android&logoColor=white",
    "/badge/Arch_Linux-1793D1?style=flat&logo=arch-linux&logoColor=white",
    "/badge/Artix_Linux-10A0CC?style=flat&logo=artix-linux&logoColor=white",
    "/badge/Cent%20OS-262577?style=flat&logo=CentOS&logoColor=white",
    "/badge/Debian-A81D33?style=flat&logo=debian&logoColor=white",
    "/badge/Deepin-007CFF?style=flat&logo=deepin&logoColor=white",
    "/badge/Elementary%20OS-64BAFF?style=flat&logo=elementary&logoColor=white",
    "/badge/Fedora-51A2DA?style=flat&logo=fedora&logoColor=white",
    "/badge/FreeBSD-AB2B28?style=flat&logo=freebsd&logoColor=white",
    "/badge/Gentoo-54487A?style=flat&logo=gentoo&logoColor=white",
    "/badge/iOS-000000?style=flat&logo=ios&logoColor=white",
    "/badge/Kali_Linux-557C94?style=flat&logo=kali-linux&logoColor=white",
    "/badge/LineageOS-167C80?style=flat&logo=lineageos&logoColor=white",
    "/badge/Linux-FCC624?style=flat&logo=linux&logoColor=black",
    "/badge/Linux_Mint-87CF3E?style=flat&logo=linux-mint&logoColor=white",
    "/badge/macOS-000000?style=flat&logo=apple&logoColor=white",
    "/badge/manjaro-35BF5C?style=flat&logo=manjaro&logoColor=white",
    "/badge/NixOS-5277C3?style=flat&logo=nixos&logoColor=white",
    "/badge/OpenWrt-00B5E2?style=flat&logo=OpenWrt&logoColor=white",
    "/badge/Pop!_OS-48B9C7?style=flat&logo=Pop!_OS&logoColor=white",
    "/badge/ReactOS-0088CC?style=flat&logo=reactos&logoColor=white",
    "/badge/Red%20Hat-EE0000?style=flat&logo=redhat&logoColor=white",
    "/badge/openSUSE-73BA25?style=flat&logo=SUSE&logoColor=white",
    "/badge/Tails%20-56347C?&style=flat&logo=tails&logoColor=white",
    "/badge/Ubuntu-E95420?style=flat&logo=ubuntu&logoColor=white",
    "/badge/-Wear%20OS-4285F4?style=flat&logo=wear-os&logoColor=white",
    "/badge/Zorin%20OS-15A6F0?style=flat&logo=zorin&logoColor=white",
    "/badge/Blogger-FF5722?style=flat&logo=blogger&logoColor=white",
    "/badge/dev.to-0A0A0A?style=flat&logo=devdotto&logoColor=white",
    "/badge/GeeksforGeeks-2F8D46?style=flat&logo=geeksforgeeks&logoColor=white",
    "/badge/Medium-12100E?style=flat&logo=medium&logoColor=white",
    "/badge/RSS-FFA500?style=flat&logo=rss&logoColor=white",
    "/badge/Wordpress-21759B?style=flat&logo=wordpress&logoColor=white",
    "/badge/Amazon%20Prime-00A8E1?style=flat&logo=netflix&logoColor=white",
    "/badge/Crunchyroll-F47521?style=flat&logo=crunchyroll&logoColor=white",
    "/badge/Facebook_Gaming-005FED?style=flat&logo=facebook-gaming&logoColor=white",
    "/badge/Netflix-E50914?style=flat&logo=netflix&logoColor=whit",
    "/badge/Twitch-9146FF?style=flat&logo=twitch&logoColor=white",
    "/badge/YouTube-FF0000?style=flat&logo=youtube&logoColor=white",
    "/badge/Python-3776AB?style=flat&logo=python&logoColor=white",
    "/badge/Python-14354C?style=flat&logo=python&logoColor=white",
    "/badge/HTML-e34c26?style=flat&logo=html5&logoColor=white",
    "/badge/HTML5-E34F26?style=flat&logo=html5&logoColor=white",
    "/badge/CSS-563d7c?&style=flat&logo=css3&logoColor=white",
    "/badge/CSS3-1572B6?style=flat&logo=css3&logoColor=white",
    "/badge/.NET-512BD4?style=flat&logo=.net&logoColor=white",
    "/badge/JavaScript-F7DF1E?style=flat&logo=javascript&logoColor=black",
    "/badge/JavaScript-323330?style=flat&logo=javascript&logoColor=F7DF1E",
    "/badge/TypeScript-3178C6?style=flat&logo=typescript&logoColor=white",
    "/badge/Node.js-339933?style=flat&logo=node.js&logoColor=white",
    "/badge/Sass-CC6699?style=flat&logo=sass&logoColor=white",
    "/badge/C-A8B9CC?style=flat&logo=c&logoColor=black",
    "/badge/C%2B%2B-00599C?style=flat&logo=c%2B%2B&logoColor=white",
    "/badge/Java-ED8B00?style=flat&logo=openjdk&logoColor=white",
    "/badge/PHP-777BB4?style=flat&logo=php&logoColor=white",
    "/badge/R-276DC3?style=flat&logo=r&logoColor=white",
    "/badge/Swift-F05138?style=flat&logo=swift&logoColor=white",
    "/badge/Kotlin-7F52FF?&style=flat&logo=kotlin&logoColor=white",
    "/badge/Go-00ADD8?style=flat&logo=go&logoColor=white",
    "/badge/Ruby-CC342D?style=flat&logo=ruby&logoColor=white",
    "/badge/Scala-DC322F?style=flat&logo=scala&logoColor=white",
    "/badge/Rust-000000?style=flat&logo=rust&logoColor=white",
    "/badge/Dart-0175C2?style=flat&logo=dart&logoColor=white",
    "/badge/Lua-2C2D72?style=flat&logo=lua&logoColor=white",
    "/badge/Perl-39457E?style=flat&logo=perl&logoColor=white",
    "/badge/Elixir-4B275F?style=flat&logo=elixir&logoColor=white",
    "/badge/Shell_Script-121011?style=flat&logo=gnu-bash&logoColor=white",
    "/badge/Gatsby-663399?style=flat&logo=gatsby&logoColor=white",
    "/badge/React-61DAFB?style=flat&logo=react&logoColor=black",
    "/badge/Svelte-FF3E00?style=flat&logo=svelte&logoColor=white",
    "/badge/Vue.js-4FC08D?style=flat&logo=vue.js&logoColor=white",
    "/badge/Angular-0F0F11?style=flat&logo=angular&logoColor=white",
    "/badge/Tailwind_CSS-06B6D4?style=flat&logo=tailwind-css&logoColor=white",
    "/badge/Bootstrap-7952B3?style=flat&logo=bootstrap&logoColor=white",
    "/badge/Redux-764ABC?style=flat&logo=redux&logoColor=white",
    "/badge/React_Router-CA4245?style=flat&logo=react-router&logoColor=white",
    "/badge/jQuery-0769AD?style=flat&logo=jquery&logoColor=white",
    "/badge/Django-092E20?style=flat&logo=django&logoColor=white",
    "/badge/Ruby_on_Rails-D30001?style=flat&logo=ruby-on-rails&logoColor=white",
    "/badge/Laravel-FF2D20?style=flat&logo=laravel&logoColor=white",
    "/badge/Spring-6DB33F?style=flat&logo=spring&logoColor=white",
    "/badge/Flask-000000?style=flat&logo=flask&logoColor=white",
    "/badge/Flutter-02569B?style=flat&logo=flutter&logoColor=white",
    "/badge/Cypress-69D3A7?style=flat&logo=cypress&logoColor=white",
    "/badge/MySQL-4479A1?style=flat&logo=mysql&logoColor=white",
    "/badge/PostgreSQL-4169E1?style=flat&logo=postgresql&logoColor=white",
    "/badge/MongoDB-47A248?style=flat&logo=mongodb&logoColor=white",
    "/badge/SQLite-003B57?style=flat&logo=sqlite&logoColor=white",
    "/badge/Unity-FFFFFF?style=flat&logo=unity&logoColor=black",
    "/badge/Netlify-00C7B7?style=flat&logo=netlify&logoColor=white",
    "/badge/Heroku-430098?style=flat&logo=heroku&logoColor=white",
    "/badge/Amazon_AWS-232F3E?style=flat&logo=amazon-web-services&logoColor=white",
    "/badge/Google_Cloud-4285F4?style=flat&logo=google-cloud&logoColor=white",
    // https://github.com/Ileriayo/markdown-badges
    "/badge/amazon%20alexa-52b5f7?style=flat&logo=amazon%20alexa&logoColor=white",
    "/badge/chatGPT-74aa9c?style=flat&logo=openai&logoColor=white",
    "/badge/dependabot-025E8C?style=flat&logo=dependabot&logoColor=white",
    "/badge/github_copilot-8957E5?style=flat&logo=github-copilot&logoColor=white",
    "/badge/google%20assistant-4285F4?style=flat&logo=google%20assistant&logoColor=white",
    "/badge/google%20gemini-8E75B2?style=flat&logo=google%20gemini&logoColor=white",
    "/badge/perplexity-000000?style=flat&logo=perplexity&logoColor=088F8F",
    "/badge/bitcoin-2F3134?style=flat&logo=bitcoin&logoColor=white",
    "/badge/hyperledger-2F3134?style=flat&logo=hyperledger&logoColor=white",
    "/badge/daily.dev-CE3DF3?style=flat&logo=daily.dev&logoColor=white",
    "/badge/dev.to-0A0A0A?style=flat&logo=dev.to&logoColor=white",
    "/badge/ghost-000?style=flat&logo=ghost&logoColor=%23F7DF1E",
    "/badge/Hashnode-2962FF?style=flat&logo=hashnode&logoColor=white",
    "/badge/Micro.blog-FF8800?style=flat&logo=micro.blog&logoColor=white",
    "/badge/rss-F88900?style=flat&logo=rss&logoColor=white",
    "/badge/Substack-%23006f5c?style=flat&logo=substack&logoColor=FF6719",
    "/badge/wix-000?style=flat&logo=wix&logoColor=white",
    "/badge/Brave-FB542B?style=flat&logo=Brave&logoColor=white",
    "/badge/duckduckgo-de5833?style=flat&logo=duckduckgo&logoColor=white",
    "/badge/Edge-0078D7?style=flat&logo=Microsoft-edge&logoColor=white",
    "/badge/Firefox-FF7139?style=flat&logo=Firefox-Browser&logoColor=white",
    "/badge/Google%20Chrome-4285F4?style=flat&logo=GoogleChrome&logoColor=white",
    "/badge/gnuicecat-263A85?style=flat&logo=gnuicecat&logoColor=white",
    "/badge/Internet%20Explorer-0076D6?style=flat&logo=Internet%20Explorer&logoColor=white",
    "/badge/Opera-FF1B2D?style=flat&logo=Opera&logoColor=white",
    "/badge/Safari-000000?style=flat&logo=Safari&logoColor=white",
    "/badge/Tor-7D4698?style=flat&logo=Tor-Browser&logoColor=white",
    "/badge/Vivaldi-EF3939?style=flat&logo=Vivaldi&logoColor=white",
    "/badge/Arc-000000?style=flat&logo=arc&logoColor=white",
    "/badge/circle%20ci-%23161616?style=flat&logo=circleci&logoColor=white",
    "/badge/chipperci-1e394e?style=flat&logo=chipperci&logoColor=white",
    "/badge/CloudBees-1997B5&?logo=cloudbees&logoColor=white&style=flat",
    "/badge/fastlane-%2382bd4e?style=flat&logo=fastlane&logoColor=black",
    "/badge/gitlab%20ci-%23181717?style=flat&logo=gitlab&logoColor=white",
    "/badge/github%20actions-%232671E5?style=flat&logo=githubactions&logoColor=white",
    "/badge/teamcity-000000?style=flat&logo=teamcity&logoColor=white",
    "/badge/travis%20ci-%232B2F33?style=flat&logo=travis&logoColor=white",
    "/badge/octopus%20deploy-0D80D8?style=flat&logo=octopusdeploy&logoColor=white",
    "/badge/Amazon%20S3-FF9900?style=flat&logo=amazons3&logoColor=white",
    "/badge/Dropbox-%233B4D98?style=flat&logo=Dropbox&logoColor=white",
    "/badge/Google%20Drive-4285F4?style=flat&logo=googledrive&logoColor=white",
    "/badge/Mega-%23D90007?style=flat&logo=Mega&logoColor=white",
    "/badge/OneDrive-white?style=flat&logo=Microsoft%20OneDrive&logoColor=0078D4",
    "/badge/Next%20Cloud-0B94DE?style=flat&logo=nextcloud&logoColor=white",
    "/badge/OneDrive-0078D4?style=flat&logo=microsoftonedrive&logoColor=white",
    "/badge/Proton%20Drive-6d4aff?style=flat&logo=proton%20drive&logoColor=white",
    "/badge/Amp-005AF0?style=flat&logo=amp&logoColor=white",
    "/badge/Bitcoin-000?style=flat&logo=bitcoin&logoColor=white",
    "/badge/Bitcoin%20Cash-0AC18E?style=flat&logo=Bitcoin%20Cash&logoColor=white",
    "/badge/Bitcoin%20SV-EAB300?style=flat&logo=Bitcoin%20SV&logoColor=white",
    "/badge/Binance-FCD535?style=flat&logo=binance&logoColor=white",
    "/badge/Chainlink-375BD2?style=flat&logo=Chainlink&logoColor=white",
    "/badge/dogecoin-B59A30?style=flat&logo=dogecoin&logoColor=white",
    "/badge/dash-008DE4?style=flat&logo=dash&logoColor=white",
    "/badge/Ethereum-3C3C3D?style=flat&logo=Ethereum&logoColor=white",
    "/badge/iota-29334C?style=flat&logo=iota&logoColor=white",
    "/badge/Litecoin-A6A9AA?style=flat&logo=Litecoin&logoColor=white",
    "/badge/monero-FF6600?style=flat&logo=monero&logoColor=white",
    "/badge/polkadot-E6007A?style=flat&logo=polkadot&logoColor=white",
    "/badge/Stellar-7D00FF?style=flat&logo=Stellar&logoColor=white",
    "/badge/tether-168363?style=flat&logo=tether&logoColor=white",
    "/badge/Xrp-black?style=flat&logo=xrp&logoColor=white",
    "/badge/Zcash-F4B728?style=flat&logo=zcash&logoColor=white",
    "/badge/Amazon%20DynamoDB-4053D6?style=flat&logo=Amazon%20DynamoDB&logoColor=white",
    "/badge/Appwrite-%23FD366E?style=flat&logo=appwrite&logoColor=white",
    "/badge/ArangoDB-DDE072?style=flat&logo=arangodb&logoColor=white",
    "/badge/cassandra-%231287B1?style=flat&logo=apache-cassandra&logoColor=white",
    "/badge/ClickHouse-FFCC01?style=flat&logo=clickhouse&logoColor=white",
    "/badge/Cockroach%20Labs-6933FF?style=flat&logo=Cockroach%20Labs&logoColor=white",
    "/badge/Couchbase-EA2328?style=flat&logo=couchbase&logoColor=white",
    "/badge/CrateDB-009DC7?style=flat&logo=CrateDB&logoColor=white",
    "/badge/firebase-a08021?style=flat&logo=firebase&logoColor=ffcd34",
    "/badge/InfluxDB-22ADF6?style=flat&logo=InfluxDB&logoColor=white",
    "/badge/MariaDB-003545?style=flat&logo=mariadb&logoColor=white",
    "/badge/Musicbrainz-EB743B?style=flat&logo=musicbrainz&logoColor=BA478F",
    "/badge/Microsoft%20SQL%20Server-CC2927?style=flat&logo=microsoft%20sql%20server&logoColor=white",
    "/badge/MongoDB-%234ea94b?style=flat&logo=mongodb&logoColor=white",
    "/badge/mysql-4479A1?style=flat&logo=mysql&logoColor=white",
    "/badge/Neo4j-008CC1?style=flat&logo=neo4j&logoColor=white",
    "/badge/planetscale-%23000000?style=flat&logo=planetscale&logoColor=white",
    "/badge/pocketbase-%23b8dbe4?style=flat&logo=Pocketbase&logoColor=black",
    "/badge/postgres-%23316192?style=flat&logo=postgresql&logoColor=white",
    "/badge/Realm-39477F?style=flat&logo=realm&logoColor=white",
    "/badge/redis-%23DD0031?style=flat&logo=redis&logoColor=white",
    "/badge/Single%20Store-AA00FF?style=flat&logo=singlestore&logoColor=white",
    "/badge/sqlite-%2307405e?style=flat&logo=sqlite&logoColor=white",
    "/badge/Supabase-3ECF8E?style=flat&logo=supabase&logoColor=white",
    "/badge/SurrealDB-FF00A0?style=flat&logo=surrealdb&logoColor=white",
    "/badge/Teradata-F37440?style=flat&logo=teradata&logoColor=white",
    "/badge/adobe-%23FF0000?style=flat&logo=adobe&logoColor=white",
    "/badge/Adobe%20Acrobat%20Reader-EC1C24?style=flat&logo=Adobe%20Acrobat%20Reader&logoColor=white",
    "/badge/Adobe%20After%20Effects-9999FF?style=flat&logo=Adobe%20After%20Effects&logoColor=white",
    "/badge/Adobe%20Audition-9999FF?style=flat&logo=Adobe%20Audition&logoColor=white",
    "/badge/Adobe%20Creative%20Cloud-DA1F26?style=flat&logo=Adobe%20Creative%20Cloud&logoColor=white",
    "/badge/Adobe%20Dreamweaver-FF61F6?style=flat&logo=Adobe%20Dreamweaver&logoColor=white",
    "/badge/Adobe%20Fonts-000B1D?style=flat&logo=Adobe%20Fonts&logoColor=white",
    "/badge/adobe%20illustrator-%23FF9A00?style=flat&logo=adobe%20illustrator&logoColor=white",
    "/badge/Adobe%20InDesign-49021F?style=flat&logo=adobeindesign&logoColor=white",
    "/badge/Adobe%20Lightroom-31A8FF?style=flat&logo=Adobe%20Lightroom&logoColor=white",
    "/badge/Adobe%20Lightroom%20Classic-31A8FF?style=flat&logo=Adobe%20Lightroom%20Classic&logoColor=white",
    "/badge/adobe%20photoshop-%2331A8FF?style=flat&logo=adobe%20photoshop&logoColor=white",
    "/badge/Adobe%20Premiere%20Pro-9999FF?style=flat&logo=Adobe%20Premiere%20Pro&logoColor=white",
    "/badge/Adobe%20XD-470137?style=flat&logo=Adobe%20XD&logoColor=#FF61F6",
    "/badge/Aseprite-FFFFFF?style=flat&logo=Aseprite&logoColor=#7D929E",
    "/badge/affinity%20desginer-%231B72BE?style=flat&logo=affinity-designer&logoColor=white",
    "/badge/affinityphoto-%237E4DD2?style=flat&logo=affinity-photo&logoColor=white",
    "/badge/blender-%23F5792A?style=flat&logo=blender&logoColor=white",
    "/badge/Canva-%2300C4CC?style=flat&logo=Canva&logoColor=white",
    "/badge/ClipStudioPaint-%23CFD3D3?style=flat&logo=ClipStudioPaint&logoColor=white",
    "/badge/figma-%23F24E1E?style=flat&logo=figma&logoColor=white",
    "/badge/Framer-black?style=flat&logo=framer&logoColor=blue",
    "/badge/Gimp-657D8B?style=flat&logo=gimp&logoColor=FFFFFF",
    "/badge/Inkscape-e0e0e0?style=flat&logo=inkscape&logoColor=080A13",
    "/badge/invision-FF3366?style=flat&logo=invision&logoColor=white",
    "/badge/Krita-203759?style=flat&logo=krita&logoColor=EEF37B",
    "/badge/penpot-%23FFFFFF?style=flat&logo=penpot&logoColor=black",
    "/badge/Proto.io-161637?style=flat&logo=proto.io&logoColor=00e5ff",
    "/badge/Rhinoceros-801010?style=flat&logo=rhinoceros&logoColor=white",
    "/badge/Sketch-FFB387?style=flat&logo=sketch&logoColor=black",
    "/badge/SketchUp-005F9E?style=flat&logo=sketchup&logoColor=white",
    "/badge/-Storybook-FF4785?style=flat&logo=storybook&logoColor=white",
    "/badge/CodeChef-%23964B00?style=flat&logo=CodeChef&logoColor=white",
    "/badge/Codeforces-445f9d?style=flat&logo=Codeforces&logoColor=white",
    "/badge/HackerEarth-%232C3454?&style=flat&logo=HackerEarth&logoColor=Blue",
    "/badge/-Hackerrank-2EC866?style=flat&logo=HackerRank&logoColor=white",
    "/badge/Kaggle-035a7d?style=flat&logo=kaggle&logoColor=white",
    "/badge/LeetCode-000000?style=flat&logo=LeetCode&logoColor=#d16c06",
    "/badge/OnePlusForums-%23EB0028?style=flat&logo=OnePlus&logoColor=white",
    "/badge/Quora-%23B92B27?style=flat&logo=Quora&logoColor=white",
    "/badge/Reddit-%23FF4500?style=flat&logo=Reddit&logoColor=white",
    "/badge/ResearchGate-00CCBB?style=flat&logo=ResearchGate&logoColor=white",
    "/badge/StackExchange-%23ffffff?style=flat&logo=StackExchange",
    "/badge/-Stackoverflow-FE7A16?style=flat&logo=stack-overflow&logoColor=white",
    "/badge/XDA--Developers-%23AC6E2F?style=flat&logo=XDA-Developers&logoColor=white",
    "/badge/Bookstack-%230288D1?style=flat&logo=bookstack&logoColor=white",
    "/badge/GitBook-%23000000?style=flat&logo=gitbook&logoColor=white",
    "/badge/Readthedocs-%23000000?style=flat&logo=readthedocs&logoColor=white",
    "/badge/Wikipedia-%23000000?style=flat&logo=wikipedia&logoColor=white",
    "/badge/wiki.js-%231976D2?style=flat&logo=wikidotjs&logoColor=white",
    "/badge/-42-black?style=flat&logo=42&logoColor=white",
    "/badge/Codecademy-FFF0E5?style=flat&logo=codecademy&logoColor=1F243A",
    "/badge/coding%20ninjas-DD6620?style=flat&logo=codingninjas&logoColor=white",
    "/badge/Codewars-B1361E?style=flat&logo=codewars&logoColor=grey",
    "/badge/Coursera-%230056D2?style=flat&logo=Coursera&logoColor=white",
    "/badge/Datacamp-05192D?style=flat&logo=datacamp&logoColor=03E860",
    "/badge/Duolingo-%234DC730?style=flat&logo=Duolingo&logoColor=white",
    "/badge/edX-%2302262B?style=flat&logo=edX&logoColor=white",
    "/badge/Exercism-009CAB?style=flat&logo=exercism&logoColor=white",
    "/badge/Freecodecamp-%23123?&style=flat&logo=freecodecamp&logoColor=green",
    "/badge/future%20learn-DE00A5?style=flat&logo=futurelearn&logoColor=white",
    "/badge/GeeksforGeeks-gray?style=flat&logo=geeksforgeeks&logoColor=35914c",
    "/badge/Google%20Scholar-4285F4?style=flat&logo=google-scholar&logoColor=white",
    "/badge/KhanAcademy-%2314BF96?style=flat&logo=KhanAcademy&logoColor=white",
    "/badge/MDN_Web_Docs-black?style=flat&logo=mdnwebdocs&logoColor=white",
    "/badge/Microsoft_Learn-258ffa?style=flat&logo=microsoft&logoColor=white",
    "/badge/Pluralsight-EE3057?style=flat&logo=pluralsight&logoColor=white",
    "/badge/scrimba-2B283A?style=flat&logo=scrimba&logoColor=white",
    "/badge/Skill%20share-002333?style=flat&logo=skillshare&logoColor=00FF84",
    "/badge/Udacity-grey?style=flat&logo=udacity&logoColor=15B8E6",
    "/badge/Udemy-A435F0?style=flat&logo=Udemy&logoColor=white",
    "/badge/AmazonPay-ff9900?style=flat&logo=Amazon-Pay&logoColor=white",
    "/badge/ApplePay-000000?style=flat&logo=Apple-Pay&logoColor=white",
    "/badge/Buy%20Me%20a%20Coffee-ffdd00?style=flat&logo=buy-me-a-coffee&logoColor=black",
    "/badge/sponsor-30363D?style=flat&logo=GitHub-Sponsors&logoColor=#EA4AAA",
    "/badge/GooglePay-%233780F1?style=flat&logo=Google-Pay&logoColor=white",
    "/badge/Ko--fi-F16061?style=flat&logo=ko-fi&logoColor=white",
    "/badge/Liberapay-F6C915?style=flat&logo=liberapay&logoColor=black",
    "/badge/Patreon-F96854?style=flat&logo=patreon&logoColor=white",
    "/badge/PayPal-00457C?style=flat&logo=paypal&logoColor=white",
    "/badge/Paytm-1C2C94?style=flat&logo=paytm&logoColor=05BAF3",
    "/badge/Phonepe-54039A?style=flat&logo=phonepe&logoColor=white",
    "/badge/SamsungPay-1428A0?style=flat&logo=Samsung-Pay&logoColor=white",
    "/badge/Stripe-5469d4?style=flat&logo=stripe&logoColor=ffffff",
    "/badge/Wise-394e79?style=flat&logo=wise&logoColor=00B9FF",
    "/badge/.NET-5C2D91?style=flat&logo=.net&logoColor=white",
    "/badge/adonisjs-%23220052?style=flat&logo=adonisjs&logoColor=white",
    "/badge/aiohttp-%232C5bb4?style=flat&logo=aiohttp&logoColor=white",
    "/badge/alpinejs-white?style=flat&logo=alpinedotjs&logoColor=%238BC0D0",
    "/badge/Anaconda-%2344A833?style=flat&logo=anaconda&logoColor=white",
    "/badge/angular-%23DD0031?style=flat&logo=angular&logoColor=white",
    "/badge/angular.js-%23E23237?style=flat&logo=angularjs&logoColor=white",
    "/badge/-AntDesign-%230170FE?style=flat&logo=ant-design&logoColor=white",
    "/badge/Apache%20Spark-FDEE21?style=flat-square&logo=apachespark&logoColor=black",
    "/badge/Apache%20Kafka-000?style=flat&logo=apachekafka",
    "/badge/Apache%20Hadoop-66CCFF?style=flat&logo=apachehadoop&logoColor=black",
    "/badge/Apache%20Hive-FDEE21?style=flat&logo=apachehive&logoColor=black",
    "/badge/-ApolloGraphQL-311C87?style=flat&logo=apollo-graphql",
    "/badge/astro-%232C2052?style=flat&logo=astro&logoColor=white",
    "/badge/aurelia-%23ED2B88?style=flat&logo=aurelia&logoColor=fff",
    "/badge/blazor-%235C2D91?style=flat&logo=blazor&logoColor=white",
    "/badge/bootstrap-%238511FA?style=flat&logo=bootstrap&logoColor=white",
    "/badge/Buefy-7957D5?style=flat&logo=buefy&logoColor=48289E",
    "/badge/bulma-00D0B1?style=flat&logo=bulma&logoColor=white",
    "/badge/Bun-%23000000?style=flat&logo=bun&logoColor=white",
    "/badge/celery-%23a9cc54?style=flat&logo=celery&logoColor=ddf4a4",
    "/badge/chakra-%234ED1C5?style=flat&logo=chakraui&logoColor=white",
    "/badge/chart.js-F5788D?style=flat&logo=chart.js&logoColor=white",
    "/badge/CodeIgniter-%23EF4223?style=flat&logo=codeIgniter&logoColor=white",
    "/badge/Context--Api-000000?style=flat&logo=react",
    "/badge/cuda-000000?style=flat&logo=nVIDIA&logoColor=green",
    "/badge/daisyui-5A0EF8?style=flat&logo=daisyui&logoColor=white",
    "/badge/deno%20js-000000?style=flat&logo=deno&logoColor=white",
    "/badge/directus-%2364f?style=flat&logo=directus&logoColor=white",
    "/badge/django-%23092E20?style=flat&logo=django&logoColor=white",
    "/badge/DJANGO-REST-ff1709?style=flat&logo=django&logoColor=white&color=ff1709&labelColor=gray",
    "/badge/drupal-%230678BE?style=flat&logo=drupal&logoColor=white",
    "/badge/ejs-%23B4CA65?style=flat&logo=ejs&logoColor=black",
    "/badge/elasticsearch-%230377CC?style=flat&logo=elasticsearch&logoColor=white",
    "/badge/Electron-191970?style=flat&logo=Electron&logoColor=white",
    "/badge/ember-1C1E24?style=flat&logo=ember.js&logoColor=#D04A37",
    "/badge/esbuild-%23FFCF00?style=flat&logo=esbuild&logoColor=black",
    "/badge/expo-1C1E24?style=flat&logo=expo&logoColor=#D04A37",
    "/badge/express.js-%23404d59?style=flat&logo=express&logoColor=%2361DAFB",
    "/badge/FastAPI-005571?style=flat&logo=fastapi",
    "/badge/fastify-%23000000?style=flat&logo=fastify&logoColor=white",
    "/badge/Filament-FFAA00?style=flat&logoColor=%23000000",
    "/badge/flask-%23000?style=flat&logo=flask&logoColor=white",
    "/badge/Flutter-%2302569B?style=flat&logo=Flutter&logoColor=white",
    "/badge/framework7-%23EE350F?style=flat&logo=framework7&logoColor=white",
    "/badge/Gatsby-%23663399?style=flat&logo=gatsby&logoColor=white",
    "/badge/grav-%23FFFFFF?style=flat&logo=grav&logoColor=221E1F",
    "/badge/green%20sock-88CE02?style=flat&logo=greensock&logoColor=white",
    "/badge/GULP-%23CF4647?style=flat&logo=gulp&logoColor=white",
    "/badge/gutenberg-%23077CB2?style=flat&logo=gutenberg&logoColor=white",
    "/badge/Insomnia-black?style=flat&logo=insomnia&logoColor=5849BE",
    "/badge/Handlebars-%23000000?style=flat&logo=Handlebars.js&logoColor=white",
    "/badge/Hugo-black?style=flat&logo=Hugo",
    "/badge/Ionic-%233880FF?style=flat&logo=Ionic&logoColor=white",
    "/badge/jasmine-%238A4182?style=flat&logo=jasmine&logoColor=white",
    "/badge/javafx-%23FF0000?style=flat&logo=javafx&logoColor=white",
    "/badge/jinja-white?style=flat&logo=jinja&logoColor=black",
    "/badge/joomla-%235091CD?style=flat&logo=joomla&logoColor=white",
    "/badge/jquery-%230769AD?style=flat&logo=jquery&logoColor=white",
    "/badge/JWT-black?style=flat&logo=JSON%20web%20tokens",
    "/badge/laravel-%23FF2D20?style=flat&logo=laravel&logoColor=white",
    "/badge/livewire-%234e56a6?style=flat&logo=livewire&logoColor=white",
    "/badge/less-2B4C80?style=flat&logo=less&logoColor=white",
    "/badge/MUI-%230081CB?style=flat&logo=mui&logoColor=white",
    "/badge/meteorjs-%23d74c4c?style=flat&logo=meteor&logoColor=white",
    "/badge/Mantine-ffffff?style=flat&logo=Mantine&logoColor=339af0",
    "/badge/MaxCompute-%23FF6701?style=flat&logo=alibabacloud&logoColor=white",
    "/badge/NPM-%23CB3837?style=flat&logo=npm&logoColor=white",
    "/badge/nestjs-%23E0234E?style=flat&logo=nestjs&logoColor=white",
    "/badge/Next-black?style=flat&logo=next.js&logoColor=white",
    "/badge/node.js-6DA55F?style=flat&logo=node.js&logoColor=white",
    "/badge/NODEMON-%23323330?style=flat&logo=nodemon&logoColor=%BBDEAD",
    "/badge/Node--RED-%238F0000?style=flat&logo=node-red&logoColor=white",
    "/badge/Nuxt-002E3B?style=flat&logo=nuxtdotjs&logoColor=#00DC82",
    "/badge/nx-143055?style=flat&logo=nx&logoColor=white",
    "/badge/opencv-%23white?style=flat&logo=opencv&logoColor=white",
    "/badge/OpenGL-%23FFFFFF?style=flat&logo=opengl",
    "/badge/p5.js-ED225D?style=flat&logo=p5.js&logoColor=FFFFFF",
    "/badge/phoenixframework-%23FD4F00?style=flat&logo=phoenixframework&logoColor=black",
    "/badge/pnpm-%234a4a4a?style=flat&logo=pnpm&logoColor=f69220",
    "/badge/Poetry-%233B82F6?style=flat&logo=poetry&logoColor=0B3D8D",
    "/badge/Prefect-%23ffffff?style=flat&logo=prefect&logoColor=white",
    "/badge/Pug-FFF?style=flat&logo=pug&logoColor=A86454",
    "/badge/pytest-%23ffffff?style=flat&logo=pytest&logoColor=2f9fe3",
    "/badge/Qt-%23217346?style=flat&logo=Qt&logoColor=white",
    "/badge/quarkus-%234794EB?style=flat&logo=quarkus&logoColor=white",
    "/badge/Quasar-16B7FB?style=flat&logo=quasar&logoColor=black",
    "/badge/ros-%230A0FF9?style=flat&logo=ros&logoColor=white",
    "/badge/Rabbitmq-FF6600?style=flat&logo=rabbitmq&logoColor=white",
    "/badge/radix%20ui-161618?style=flat&logo=radix-ui&logoColor=white",
    "/badge/rails-%23CC0000?style=flat&logo=ruby-on-rails&logoColor=white",
    "/badge/RAYLIB-FFFFFF?style=flat&logo=raylib&logoColor=black",
    "/badge/react-%2320232a?style=flat&logo=react&logoColor=%2361DAFB",
    "/badge/react_native-%2320232a?style=flat&logo=react&logoColor=%2361DAFB",
    "/badge/-React%20Query-FF4154?style=flat&logo=react%20query&logoColor=white",
    "/badge/React%20Hook%20Form-%23EC5990?style=flat&logo=reacthookform&logoColor=white",
    "/badge/redux-%23593d88?style=flat&logo=redux&logoColor=white",
    "/badge/remix-%23000?style=flat&logo=remix&logoColor=white",
    "/badge/RollupJS-ef3335?style=flat&logo=rollup.js&logoColor=white",
    "/badge/rxjs-%23B7178C?style=flat&logo=reactivex&logoColor=white",
    "/badge/SASS-hotpink?style=flat&logo=SASS&logoColor=white",
    "/badge/scrapy-%2360a839?style=flat&logo=scrapy&logoColor=d1d2d3",
    "/badge/Semantic%20UI%20React-%2335BDB2?style=flat&logo=SemanticUIReact&logoColor=white",
    "/badge/snowflake-%2329B5E8?style=flat&logo=snowflake&logoColor=white",
    "/badge/Socket.io-black?style=flat&logo=socket.io&badgeColor=010101",
    "/badge/SolidJS-2c4f7c?style=flat&logo=solid&logoColor=c8c9cb",
    "/badge/spring-%236DB33F?style=flat&logo=spring&logoColor=white",
    "/badge/strapi-%232E7EEA?style=flat&logo=strapi&logoColor=white",
    "/badge/Streamlit-%23FE4B4B?style=flat&logo=streamlit&logoColor=white",
    "/badge/styled--components-DB7093?style=flat&logo=styled-components&logoColor=white",
    "/badge/stylus-%23ff6347?style=flat&logo=stylus&logoColor=white",
    "/badge/svelte-%23f1413d?style=flat&logo=svelte&logoColor=white",
    "/badge/symfony-%23000000?style=flat&logo=symfony&logoColor=white",
    "/badge/tailwindcss-%2338B2AC?style=flat&logo=tailwind-css&logoColor=white",
    "/badge/tauri-%2324C8DB?style=flat&logo=tauri&logoColor=%23FFFFFF",
    "/badge/threejs-black?style=flat&logo=three.js&logoColor=white",
    "/badge/Thymeleaf-%23005C0F?style=flat&logo=Thymeleaf&logoColor=white",
    "/badge/tRPC-%232596BE?style=flat&logo=tRPC&logoColor=white",
    "/badge/-TypeGraphQL-%23C04392?style=flat",
    "/badge/unocss-333333?style=flat&logo=unocss&logoColor=white",
    "/badge/vite-%23646CFF?style=flat&logo=vite&logoColor=white",
    "/badge/vuejs-%2335495e?style=flat&logo=vuedotjs&logoColor=%234FC08D",
    "/badge/Vuetify-1867C0?style=flat&logo=vuetify&logoColor=AEDDFF",
    "/badge/WebGL-990000?logo=webgl&logoColor=white&style=flat",
    "/badge/webpack-%238DD6F9?style=flat&logo=webpack&logoColor=black",
    "/badge/web3.js-F16822?style=flat&logo=web3.js&logoColor=white",
    "/badge/windicss-48B0F1?style=flat&logo=windi-css&logoColor=white",
    "/badge/WordPress-%23117AC9?style=flat&logo=WordPress&logoColor=white",
    "/badge/Xamarin-3199DC?style=flat&logo=xamarin&logoColor=white",
    "/badge/yarn-%232C8EBB?style=flat&logo=yarn&logoColor=white",
    "/badge/zod-%233068b7?style=flat&logo=zod&logoColor=white",
    "/badge/AMD-%23000000?style=flat&logo=amd&logoColor=white",
    "/badge/Analogue-1A1A1A?style=flat&logo=Analogue&logoColor=white",
    "/badge/battle.net-%2300AEFF?style=flat&logo=battle.net&logoColor=white",
    "/badge/bevy-%23232326?style=flat&logo=bevy&logoColor=white",
    "/badge/ea-%23000000?style=flat&logo=ea&logoColor=white",
    "/badge/epicgames-%23313131?style=flat&logo=epicgames&logoColor=white",
    "/badge/GODOT-%23FFFFFF?style=flat&logo=godot-engine",
    "/badge/HumbleBundle-%23494F5C?style=flat&logo=HumbleBundle&logoColor=white",
    "/badge/Itch-%23FF0B34?style=flat&logo=Itch.io&logoColor=white",
    "/badge/nVIDIA-%2376B900?style=flat&logo=nVIDIA&logoColor=white",
    "/badge/OpenGL-white?logo=OpenGL&style=flat",
    "/badge/PSN-%230070D1?style=flat&logo=Playstation&logoColor=white",
    "/badge/riotgames-D32936?style=flat&logo=riotgames&logoColor=white",
    "/badge/sidequest-%23101227?style=flat&logo=sidequest&logoColor=white",
    "/badge/SquareEnix-%23ED1C24?style=flat&logo=SquareEnix&logoColor=white",
    "/badge/steam-%23000000?style=flat&logo=steam&logoColor=white",
    "/badge/Ubisoft-%23F5F5F5?style=flat&logo=Ubisoft&logoColor=black",
    "/badge/unity-%23000000?style=flat&logo=unity&logoColor=white",
    "/badge/unrealengine-%23313131?style=flat&logo=unrealengine&logoColor=white",
    "/badge/xbox-%23107C10?style=flat&logo=xbox&logoColor=white",
    "/badge/3DS-D12228?style=flat&logo=nintendo-3ds&logoColor=white",
    "/badge/Gamecube-6A5FBB?style=flat&logo=nintendo-gamecube&logoColor=white",
    "/badge/Playstation-003791?style=flat&logo=playstation&logoColor=white",
    "/badge/Playstation%202-003791?style=flat&logo=playstation-2&logoColor=white",
    "/badge/Playstation%203-003791?style=flat&logo=playstation-3&logoColor=white",
    "/badge/Playstation%204-003791?style=flat&logo=playstation-4&logoColor=white",
    "/badge/Playstation%205-003791?style=flat&logo=playstation-5&logoColor=white",
    "/badge/Playstation%20Vita-003791?style=flat&logo=playstation-vita&logoColor=white",
    "/badge/Switch-E60012?style=flat&logo=nintendo-switch&logoColor=white",
    "/badge/Wii-8B8B8B?style=flat&logo=wii&logoColor=white",
    "/badge/Wii%20U-8B8B8B?style=flat&logo=wiiu&logoColor=white",
    "/badge/AlibabaCloud-%23FF6701?style=flat&logo=alibabacloud&logoColor=white",
    "/badge/AWS-%23FF9900?style=flat&logo=amazon-aws&logoColor=white",
    "/badge/azure-%230072C6?style=flat&logo=microsoftazure&logoColor=white",
    "/badge/Cloudflare-F38020?style=flat&logo=Cloudflare&logoColor=white",
    "/badge/Codeberg-2185D0?style=flat&logo=Codeberg&logoColor=white",
    "/badge/datadog-%23632CA6?style=flat&logo=datadog&logoColor=white",
    "/badge/DigitalOcean-%230167ff?style=flat&logo=digitalOcean&logoColor=white",
    "/badge/firebase-%23039BE5?style=flat&logo=firebase",
    "/badge/github%20pages-121013?style=flat&logo=github&logoColor=white",
    "/badge/glitch-%233333FF?style=flat&logo=glitch&logoColor=white",
    "/badge/GoogleCloud-%234285F4?style=flat&logo=google-cloud&logoColor=white",
    "/badge/heroku-%23430098?style=flat&logo=heroku&logoColor=white",
    "/badge/linode-00A95C?style=flat&logo=linode&logoColor=white",
    "/badge/netlify-%23000000?style=flat&logo=netlify&logoColor=#00C7B7",
    "/badge/Oracle-F80000?style=flat&logo=oracle&logoColor=white",
    "/badge/Openstack-%23f01742?style=flat&logo=openstack&logoColor=white",
    "/badge/ovh-%23123F6D?style=flat&logo=ovh&logoColor=#123F6D",
    "/badge/pythonanywhere-%232F9FD7?style=flat&logo=pythonanywhere&logoColor=151515",
    "/badge/proxmox-proxmox?style=flat&logo=proxmox&logoColor=%23E57000&labelColor=%232b2a33&color=%232b2a33",
    "/badge/Render-%46E3B7?style=flat&logo=render&logoColor=white",
    "/badge/SCALEWAY-%234f0599?style=flat&logo=scaleway&logoColor=white",
    "/badge/vercel-%23000000?style=flat&logo=vercel&logoColor=white",
    "/badge/Vultr-007BFC?style=flat&logo=vultr",
    "/badge/android%20studio-346ac1?style=flat&logo=android%20studio&logoColor=white",
    "/badge/Atom-%2366595C?style=flat&logo=atom&logoColor=white",
    "/badge/CLion-black?style=flat&logo=clion&logoColor=white",
    "/badge/CodePen-white?style=flat&logo=codepen&logoColor=black",
    "/badge/Codesandbox-040404?style=flat&logo=codesandbox&logoColor=DBDBDB",
    "/badge/doxygen-2C4AA8?style=flat&logo=doxygen&logoColor=white",
    "/badge/Eclipse-FE7A16?style=flat&logo=Eclipse&logoColor=white",
    "/badge/Emacs-%237F5AB6?&style=flat&logo=gnu-emacs&logoColor=white",
    "/badge/GoLand-0f0f0f?&style=flat&logo=goland&logoColor=white",
    "/badge/Google%20Colab-%23F9A825?style=flat&logo=googlecolab&logoColor=white",
    "/badge/Helix-%2328153e?style=flat&logo=helix&logoColor=white",
    "/badge/IntelliJIDEA-000000?style=flat&logo=intellij-idea&logoColor=white",
    "/badge/jupyter-%23FA0F00?style=flat&logo=jupyter&logoColor=white",
    "/badge/NeoVim-%2357A143?&style=flat&logo=neovim&logoColor=white",
    "/badge/NetBeansIDE-1B6AC6?style=flat&logo=apache-netbeans-ide&logoColor=white",
    "/badge/Notepad++-90E59A?style=flat&logo=notepad%2b%2b&logoColor=black",
    "/badge/Obsidian-%23483699?style=flat&logo=obsidian&logoColor=white",
    "/badge/phpstorm-143?style=flat&logo=phpstorm&logoColor=black&color=black&labelColor=darkorchid",
    "/badge/pycharm-143?style=flat&logo=pycharm&logoColor=black&color=black&labelColor=green",
    "/badge/Replit-DD1200?style=flat&logo=Replit&logoColor=white",
    "/badge/Rider-000000?style=flat&logo=Rider&logoColor=white&color=black&labelColor=crimson",
    "/badge/RStudio-4285F4?style=flat&logo=rstudio&logoColor=white",
    "/badge/Spyder-838485?style=flat&logo=spyder%20ide&logoColor=maroon",
    "/badge/Stackblitz-fff?style=flat&logo=Stackblitz&logoColor=1389FD",
    "/badge/sublime_text-%23575757?style=flat&logo=sublime-text&logoColor=important",
    "/badge/VIM-%2311AB00?style=flat&logo=vim&logoColor=white",
    "/badge/Visual%20Studio%20Code-0078d7?style=flat&logo=visual-studio-code&logoColor=white",
    "/badge/VS%20Code%20Insiders-35b393?style=flat&logo=visual-studio-code&logoColor=white",
    "/badge/Visual%20Studio-5C2D91?style=flat&logo=visual-studio&logoColor=white",
    "/badge/webstorm-143?style=flat&logo=webstorm&logoColor=white&color=black",
    "/badge/Xcode-007ACC?style=flat&logo=Xcode&logoColor=white",
    "/badge/zedindustries-084CCF?style=flat&logo=zedindustries&logoColor=white",
    "/badge/Zend-fff?style=flat&logo=zend&logoColor=0679EA",
    "/badge/Apache%20Groovy-4298B8?style=flat&logo=Apache+Groovy&logoColor=white",
    "/badge/assembly%20script-%23000000?style=flat&logo=assemblyscript&logoColor=white",
    "/badge/c-%2300599C?style=flat&logo=c&logoColor=white",
    "/badge/c%23-%23239120?style=flat&logo=csharp&logoColor=white",
    "/badge/c++-%2300599C?style=flat&logo=c%2B%2B&logoColor=white",
    "/badge/Clojure-%23Clojure?style=flat&logo=Clojure&logoColor=Clojure",
    "/badge/crystal-%23000000?style=flat&logo=crystal&logoColor=white",
    "/badge/css3-%231572B6?style=flat&logo=css3&logoColor=white",
    "/badge/dart-%230175C2?style=flat&logo=dart&logoColor=white",
    "/badge/dgraph-%23E50695?style=flat&logo=dgraph&logoColor=white",
    "/badge/elixir-%234B275F?style=flat&logo=elixir&logoColor=white",
    "/badge/Elm-60B5CC?style=flat&logo=elm&logoColor=white",
    "/badge/Erlang-white?style=flat&logo=erlang&logoColor=a90533",
    "/badge/Fortran-%23734F96?style=flat&logo=fortran&logoColor=white",
    "/badge/GDScript-%2374267B?style=flat&logo=godotengine&logoColor=white",
    "/badge/go-%2300ADD8?style=flat&logo=go&logoColor=white",
    "/badge/-GraphQL-E10098?style=flat&logo=graphql&logoColor=white",
    "/badge/Haskell-5e5086?style=flat&logo=haskell&logoColor=white",
    "/badge/html5-%23E34F26?style=flat&logo=html5&logoColor=white",
    "/badge/java-%23ED8B00?style=flat&logo=openjdk&logoColor=white",
    "/badge/javascript-%23323330?style=flat&logo=javascript&logoColor=%23F7DF1E",
    "/badge/-Julia-9558B2?style=flat&logo=julia&logoColor=white",
    "/badge/kotlin-%237F52FF?style=flat&logo=kotlin&logoColor=white",
    "/badge/latex-%23008080?style=flat&logo=latex&logoColor=white",
    "/badge/lua-%232C2D72?style=flat&logo=lua&logoColor=white",
    "/badge/markdown-%23000000?style=flat&logo=markdown&logoColor=white",
    "/badge/nim-%23FFE953?style=flat&logo=nim&logoColor=white",
    "/badge/NIX-5277C3?style=flat&logo=NixOS&logoColor=white",
    "/badge/OBJECTIVE--C-%233A95E3?style=flat&logo=apple&logoColor=white",
    "/badge/OCaml-%23E98407?style=flat&logo=ocaml&logoColor=white",
    "/badge/OCTAVE-darkblue?style=flat&logo=octave&logoColor=fcd683",
    "/badge/orgmode-%2377AA99?style=flat&logo=org&logoColor=white",
    "/badge/perl-%2339457E?style=flat&logo=perl&logoColor=white",
    "/badge/php-%23777BB4?style=flat&logo=php&logoColor=white",
    "/badge/PowerShell-%235391FE?style=flat&logo=powershell&logoColor=white",
    "/badge/python-3670A0?style=flat&logo=python&logoColor=ffdd54",
    "/badge/r-%23276DC3?style=flat&logo=r&logoColor=white",
    "/badge/rescript-%2314162c?style=flat&logo=rescript&logoColor=e34c4c",
    "/badge/ruby-%23CC342D?style=flat&logo=ruby&logoColor=white",
    "/badge/rust-%23000000?style=flat&logo=rust&logoColor=white",
    "/badge/scala-%23DC322F?style=flat&logo=scala&logoColor=white",
    "/badge/bash_script-%23121011?style=flat&logo=gnu-bash&logoColor=white",
    "/badge/Solidity-%23363636?style=flat&logo=solidity&logoColor=white",
    "/badge/swift-F54A2A?style=flat&logo=swift&logoColor=white",
    "/badge/typescript-%23007ACC?style=flat&logo=typescript&logoColor=white",
    "/badge/Windows%20Terminal-%234D4D4D?style=flat&logo=windows-terminal&logoColor=white",
    "/badge/yaml-%23ffffff?style=flat&logo=yaml&logoColor=151515",
    "/badge/Zig-%23F7A41D?style=flat&logo=zig&logoColor=white",
    "/badge/Keras-%23D00000?style=flat&logo=Keras&logoColor=white",
    "/badge/Matplotlib-%23ffffff?style=flat&logo=Matplotlib&logoColor=black",
    "/badge/mlflow-%23d9ead3?style=flat&logo=numpy&logoColor=blue",
    "/badge/numpy-%23013243?style=flat&logo=numpy&logoColor=white",
    "/badge/pandas-%23150458?style=flat&logo=pandas&logoColor=white",
    "/badge/Plotly-%233F4F75?style=flat&logo=plotly&logoColor=white",
    "/badge/PyTorch-%23EE4C2C?style=flat&logo=PyTorch&logoColor=white",
    "/badge/scikit--learn-%23F7931E?style=flat&logo=scikit-learn&logoColor=white",
    "/badge/SciPy-%230C55A5?style=flat&logo=scipy&logoColor=%white",
    "/badge/TensorFlow-%23FF6F00?style=flat&logo=TensorFlow&logoColor=white",
    "/badge/Apple_Music-9933CC?style=flat&logo=apple-music&logoColor=white",
    "/badge/Audacity-0000CC?style=flat&logo=audacity&logoColor=white",
    "/badge/Deezer-FEAA2D?style=flat&logo=deezer&logoColor=white",
    "/badge/last.fm-D51007?style=flat&logo=last.fm&logoColor=white",
    "/badge/soundcloud-FF5500?style=flat&logo=soundcloud&logoColor=white",
    "/badge/Spotify-1ED760?style=flat&logo=spotify&logoColor=white",
    "/badge/shazam-1476FE?style=flat&logo=shazam&logoColor=white",
    "/badge/tidal-00FFFF?style=flat&logo=tidal&logoColor=black",
    "/badge/YouTube_Music-FF0000?style=flat&logo=youtube-music&logoColor=white",
    "/badge/Musixmatch-%23FF5353?style=flat&logo=Musixmatch&logoColor=white",
    "/badge/LibreOffice-%2318A303?style=flat&logo=LibreOffice&logoColor=white",
    "/badge/Microsoft-0078D4?style=flat&logo=microsoft&logoColor=white",
    "/badge/Microsoft_Access-A4373A?style=flat&logo=microsoft-access&logoColor=white",
    "/badge/Microsoft_Excel-217346?style=flat&logo=microsoft-excel&logoColor=white",
    "/badge/Microsoft_Office-D83B01?style=flat&logo=microsoft-office&logoColor=white",
    "/badge/Microsoft_PowerPoint-B7472A?style=flat&logo=microsoft-powerpoint&logoColor=white",
    "/badge/Microsoft_SharePoint-0078D4?style=flat&logo=microsoft-sharepoint&logoColor=white",
    "/badge/Microsoft_Visio-3955A3?style=flat&logo=microsoft-visio&logoColor=white",
    "/badge/Microsoft_Word-2B579A?style=flat&logo=microsoft-word&logoColor=white",
    "/badge/Alpine_Linux-%230D597F?style=flat&logo=alpine-linux&logoColor=white",
    "/badge/Android-3DDC84?style=flat&logo=android&logoColor=white",
    "/badge/Arch%20Linux-1793D1?logo=arch-linux&logoColor=fff&style=flat",
    "/badge/cent%20os-002260?style=flat&logo=centos&logoColor=F0F0F0",
    "/badge/chrome%20os-3d89fc?style=flat&logo=google%20chrome&logoColor=white",
    "/badge/Debian-D70A53?style=flat&logo=debian&logoColor=white",
    "/badge/-elementary%20OS-black?style=flat&logo=elementary&logoColor=white",
    "/badge/Fedora-294172?style=flat&logo=fedora&logoColor=white",
    "/badge/-FreeBSD-%23870000?style=flat&logo=freebsd&logoColor=white",
    "/badge/Kali-268BEE?style=flat&logo=kalilinux&logoColor=white",
    "/badge/-KUbuntu-%230079C1?style=flat&logo=kubuntu&logoColor=white",
    "/badge/Linux%20Mint-87CF3E?style=flat&logo=Linux%20Mint&logoColor=white",
    "/badge/-Lubuntu-%230065C2?style=flat&logo=lubuntu&logoColor=white",
    "/badge/lineageos-167C80?style=flat&logo=lineageos&logoColor=white",
    "/badge/Manjaro-35BF5C?style=flat&logo=Manjaro&logoColor=white",
    "/badge/-MX%20Linux-%23000000?style=flat&logo=MXlinux&logoColor=white",
    "/badge/mac%20os-000000?style=flat&logo=macos&logoColor=F0F0F0",
    "/badge/NIXOS-5277C3?style=flat&logo=NixOS&logoColor=white",
    "/badge/OpenWRT-00B5E2?style=flat&logo=OpenWrt&logoColor=white",
    "/badge/-OpenBSD-%23FCC771?style=flat&logo=openbsd&logoColor=black",
    "/badge/openSUSE-%2364B345?style=flat&logo=openSUSE&logoColor=white",
    "/badge/-Rocky%20Linux-%2310B981?style=flat&logo=rockylinux&logoColor=white",
    "/badge/Solus-%23f2f2f2?style=flat&logo=solus&logoColor=5294E2",
    "/badge/SUSE-0C322C?style=flat&logo=SUSE&logoColor=white",
    "/badge/-Slackware-%231357BD?style=flat&logo=slackware&logoColor=white",
    "/badge/Ubuntu%20MATE-84A454?style=flat&logo=Ubuntu-MATE&logoColor=white",
    "/badge/unraid-%23F15A2C?style=flat&logo=unraid&logoColor=white",
    "/badge/Windows-0078D6?style=flat&logo=windows&logoColor=white",
    "/badge/Windows%2011-%230079d5?style=flat&logo=Windows%2011&logoColor=white",
    "/badge/Windows%2095-008484?style=flat&logo=windows95&logoColor=white",
    "/badge/Windows%20xp-003399?style=flat&logo=windowsxp&logoColor=white",
    "/badge/-Zorin%20OS-%2310AAEB?style=flat&logo=zorin&logoColor=white",
    "/badge/Hibernate-59666C?style=flat&logo=Hibernate&logoColor=white",
    "/badge/Prisma-3982CE?style=flat&logo=Prisma&logoColor=white",
    "/badge/Sequelize-52B0E7?style=flat&logo=Sequelize&logoColor=white",
    "/badge/TypeORM-FE0803?style=flat&logo=typeorm&logoColor=white",
    "/badge/Quill-52B0E7?style=flat&logo=apache&logoColor=white",
    "/badge/Accessibility-%230170EA?style=flat&logo=Accessibility&logoColor=white",
    "/badge/Airbnb-%23ff5a5f?style=flat&logo=Airbnb&logoColor=white",
    "/badge/alfred-%235C1F87?style=flat&logo=alfred",
    "/badge/ansible-%231A1918?style=flat&logo=ansible&logoColor=white",
    "/badge/aqua-%231904DA?style=flat&logo=aqua&logoColor=#0018A8",
    "/badge/-Arduino-00979D?style=flat&logo=Arduino&logoColor=white",
    "/badge/Babel-F9DC3e?style=flat&logo=babel&logoColor=black",
    "/badge/bitwarden-%23175DDC?style=flat&logo=bitwarden&logoColor=white",
    "/badge/cisco-%23049fd9?style=flat&logo=cisco&logoColor=black",
    "/badge/CMake-%23008FBA?style=flat&logo=cmake&logoColor=white",
    "/badge/codecov-%23ff0077?style=flat&logo=codecov&logoColor=white",
    "/badge/confluence-%23172BF4?style=flat&logo=confluence&logoColor=white",
    "/badge/Crowdin-2E3340?style=flat&logo=Crowdin&logoColor=white",
    "/badge/docker-%230db7ed?style=flat&logo=docker&logoColor=white",
    "/badge/ESLint-4B3263?style=flat&logo=eslint&logoColor=white",
    "/badge/-ElasticSearch-005571?style=flat&logo=elasticsearch",
    "/badge/espressif-E7352C?style=flat&logo=espressif&logoColor=white",
    "/badge/Google%20TalkBack-%236636B4?style=flat&logo=GoogleTalkBack&logoColor=white",
    "/badge/Gradle-02303A?style=flat&logo=Gradle&logoColor=white",
    "/badge/grafana-%23F46800?style=flat&logo=grafana&logoColor=white",
    "/badge/home%20assistant-%2341BDF5?style=flat&logo=home-assistant&logoColor=white",
    "/badge/homebridge-%23491F59?style=flat&logo=homebridge&logoColor=white",
    "/badge/JAWS-%231962AA?style=flat&logo=JAWS&logoColor=white",
    "/badge/jellyfin-%23000B25?style=flat&logo=Jellyfin&logoColor=00A4DC",
    "/badge/jira-%230A0FFF?style=flat&logo=jira&logoColor=white",
    "/badge/kubernetes-%23326ce5?style=flat&logo=kubernetes&logoColor=white",
    "/badge/Meta-%230467DF?style=flat&logo=Meta&logoColor=white",
    "/badge/mosquitto-%233C5280?style=flat&logo=eclipsemosquitto&logoColor=white",
    "/badge/Narrator-%230771D0?style=flat&logo=Narrator&logoColor=white",
    "/badge/Notion-%23000000?style=flat&logo=notion&logoColor=white",
    "/badge/NVDA-%23630093?style=flat&logo=NVDA&logoColor=white",
    "/badge/openapiinitiative-%23000000?style=flat&logo=openapiinitiative&logoColor=white",
    "/badge/OpenSea-%232081E2?style=flat&logo=opensea&logoColor=white",
    "/badge/OpenTelemetry-FFFFFF?&style=flat&logo=opentelemetry&logoColor=black",
    "/badge/packer-%23E7EEF0?style=flat&logo=packer&logoColor=%2302A8EF",
    "/badge/pihole-%2396060C?style=flat&logo=pi-hole&logoColor=white",
    "/badge/PlatformIO-%23222?style=flat&logo=platformio&logoColor=%23f5822a",
    "/badge/plex-%23E5A00D?style=flat&logo=plex&logoColor=white",
    "/badge/Portfolio-%23000000?style=flat&logo=firefox&logoColor=#FF7139",
    "/badge/Postman-FF6C37?style=flat&logo=postman&logoColor=white",
    "/badge/power_bi-F2C811?style=flat&logo=powerbi&logoColor=black",
    "/badge/prettier-%23F7B93E?style=flat&logo=prettier&logoColor=black",
    "/badge/Prezi-%23000000?style=flat&logo=Prezi&logoColor=white",
    "/badge/Prometheus-E6522C?style=flat&logo=Prometheus&logoColor=white",
    "/badge/pypi-%23ececec?style=flat&logo=pypi&logoColor=1f73b7",
    "/badge/rancher-%230075A8?style=flat&logo=rancher&logoColor=white",
    "/badge/-Raspberry_Pi-C51A4A?style=flat&logo=Raspberry-Pi",
    "/badge/SonarLint-CB2029?style=flat&logo=SONARLINT&logoColor=white",
    "/badge/SonarQube-black?style=flat&logo=sonarqube&logoColor=4E9BCD",
    "/badge/splunk-%23000000?style=flat&logo=splunk&logoColor=white",
    "/badge/-Swagger-%23Clojure?style=flat&logo=swagger&logoColor=white",
    "/badge/tampermonkey-%2300485B?style=flat&logo=tampermonkey&logoColor=white",
    "/badge/tor-%237E4798?style=flat&logo=tor-project&logoColor=white",
    "/badge/terraform-%235835CC?style=flat&logo=terraform&logoColor=white",
    "/badge/Trello-%23026AA7?style=flat&logo=Trello&logoColor=white",
    "/badge/Twilio-F22F46?style=flat&logo=Twilio logoColor=white",
    "/badge/Uber-%23000000?style=flat&logo=Uber&logoColor=white",
    "/badge/ubiquiti-%230559C9?style=flat&logo=ubiquiti&logoColor=white",
    "/badge/vagrant-%231563FF?style=flat&logo=vagrant&logoColor=white",
    "/badge/VoiceOver-%23484848?style=flat&logo=VoiceOver&logoColor=white",
    "/badge/WCAG-%23015A69?style=flat&logo=WCAG&logoColor=white",
    "/badge/wireguard-%2388171A?style=flat&logo=wireguard&logoColor=white",
    "/badge/XFCE-%232284F2?style=flat&logo=xfce&logoColor=white",
    "/badge/zigbee-%23EB0443?style=flat&logo=zigbee&logoColor=white",
    "/badge/Qiskit-%236929C4?style=flat&logo=Qiskit&logoColor=white",
    "/badge/Baidu-2932E1?style=flat&logo=Baidu&logoColor=white",
    "/badge/Microsoft%20Bing-258FFA?style=flat&logo=Microsoft%20Bing&logoColor=white",
    "/badge/DuckDuckGo-DE5833?style=flat&logo=DuckDuckGo&logoColor=white",
    "/badge/google-4285F4?style=flat&logo=google&logoColor=white",
    "/badge/Yahoo!-6001D2?style=flat&logo=Yahoo!&logoColor=white",
    "/badge/apache-%23D42029?style=flat&logo=apache&logoColor=white",
    "/badge/Apache%20Airflow-017CEE?style=flat&logo=Apache%20Airflow&logoColor=white",
    "/badge/Apache%20Ant-A81C7D?style=flat&logo=Apache%20Ant&logoColor=white",
    "/badge/Apache%20Flink-E6526F?style=flat&logo=Apache%20Flink&logoColor=white",
    "/badge/Apache%20Maven-C71A36?style=flat&logo=Apache%20Maven&logoColor=white",
    "/badge/apache%20tomcat-%23F8DC75?style=flat&logo=apache-tomcat&logoColor=black",
    "/badge/gunicorn-%298729?style=flat&logo=gunicorn&logoColor=white",
    "/badge/jenkins-%232C5263?style=flat&logo=jenkins&logoColor=white",
    "/badge/nginx-%23009639?style=flat&logo=nginx&logoColor=white",
    "/badge/Airtable-18BFFF?style=flat&logo=Airtable&logoColor=white",
    "/badge/Bluesky-0285FF?style=flat&logo=Bluesky&logoColor=white",
    "/badge/Discord-%235865F2?style=flat&logo=discord&logoColor=white",
    "/badge/Facebook-%231877F2?style=flat&logo=Facebook&logoColor=white",
    "/badge/Gmail-D14836?style=flat&logo=gmail&logoColor=white",
    "/badge/Goodreads-F3F1EA?style=flat&logo=goodreads&logoColor=372213",
    "/badge/Google%20Meet-00897B?style=flat&logo=google-meet&logoColor=white",
    "/badge/Instagram-%23E4405F?style=flat&logo=Instagram&logoColor=white",
    "/badge/kakaotalk-ffcd00?style=flat&logo=kakaotalk&logoColor=000000",
    "/badge/Line-00C300?style=flat&logo=line&logoColor=white",
    "/badge/linkedin-%230077B5?style=flat&logo=linkedin&logoColor=white",
    "/badge/linktree-1de9b6?style=flat&logo=linktree&logoColor=white",
    "/badge/Meetup-f64363?style=flat&logo=meetup&logoColor=white",
    "/badge/-MASTODON-%232B90D9?style=flat&logo=mastodon&logoColor=white",
    "/badge/Microsoft_Outlook-0078D4?style=flat&logo=microsoft-outlook&logoColor=white",
    "/badge/odysee-EF1970?style=flat&logo=Odysee&logoColor=white",
    "/badge/Pinterest-%23E60023?style=flat&logo=Pinterest&logoColor=white",
    "/badge/ProtonMail-8B89CC?style=flat&logo=protonmail&logoColor=white",
    "/badge/Polywork-543DE0?style=flat&logo=polywork&logoColor=black",
    "/badge/Session-%23000000?style=flat&logo=Session&logoColor=02f780",
    "/badge/Signal-%23039BE5?style=flat&logo=Signal&logoColor=white",
    "/badge/Skype-%2300AFF0?style=flat&logo=Skype&logoColor=white",
    "/badge/Snapchat-%23FFFC00?style=flat&logo=Snapchat&logoColor=white",
    "/badge/TeamSpeak-2580C3?style=flat&logo=teamspeak&logoColor=white",
    "/badge/Telegram-2CA5E0?style=flat&logo=telegram&logoColor=white",
    "/badge/Tencent%23QQ-%2312B7F5?style=flat&logo=tencentqq&logoColor=white",
    "/badge/TikTok-%23000000?style=flat&logo=TikTok&logoColor=white",
    "/badge/Thunderbird-0A84FF?style=flat&logo=Thunderbird&logoColor=white",
    "/badge/Tumblr-%2336465D?style=flat&logo=Tumblr&logoColor=white",
    "/badge/Tutanota-840010?style=flat&logo=Tutanota&logoColor=white",
    "/badge/Twitch-%239146FF?style=flat&logo=Twitch&logoColor=white",
    "/badge/Viber-8B66A9?style=flat&logo=viber&logoColor=white",
    "/badge/Wire-B71C1C?style=flat&logo=wire&logoColor=white",
    "/badge/X-%23000000?style=flat&logo=X&logoColor=white",
    "/badge/Xbox-%23107C10?style=flat&logo=Xbox&logoColor=white",
    "/badge/xing-%23006567?style=flat&logo=xing&logoColor=white",
    "/badge/YouTube-%23FF0000?style=flat&logo=YouTube&logoColor=white",
    "/badge/Zoom-2D8CFF?style=flat&logo=zoom&logoColor=white",
    "/badge/Threads-000000?style=flat&logo=Threads&logoColor=white",
    "/badge/Apple-%23000000?style=flat&logo=apple&logoColor=white",
    "/badge/asus-000080?style=flat&logo=asus&logoColor=white",
    "/badge/blackberry-808080?style=flat&logo=blackberry&logoColor=white",
    "/badge/Huawei-%23FF0000?style=flat&logo=huawei&logoColor=white",
    "/badge/lenovo-E2231A?style=flat&logo=lenovo&logoColor=white",
    "/badge/lg-a50034?style=flat&logo=lg&logoColor=white",
    "/badge/OnePlus-%23F5010C?style=flat&logo=oneplus&logoColor=white",
    "/badge/Motorola-%23E1140A?style=flat&logo=motorola&logoColor=white",
    "/badge/Nokia-%23124191?style=flat&logo=nokia&logoColor=white",
    "/badge/Samsung-%231428A0?style=flat&logo=samsung&logoColor=white",
    "/badge/Xiaomi-%23FF6900?style=flat&logo=xiaomi&logoColor=white",
    "/badge/Oppo-%231EA366?style=flat&logo=oppo&logoColor=white",
    "/badge/Vivo-%2300BFFF?style=flat&logo=vivo&logoColor=black",
    "/badge/F_Droid-1976D2?style=flat&logo=f-droid&logoColor=white",
    "/badge/AppGallery-C80A2D?style=flat&logo=huawei&logoColor=white",
    "/badge/Amazon%20Prime-0F79AF?style=flat&logo=amazonprime&logoColor=white",
    "/badge/Apple%20TV-000000?style=flat&logo=Apple%20TV&logoColor=white",
    "/badge/Facebook%20Gaming-015BE5?style=flat&logo=facebookgaming&logoColor=white",
    "/badge/Facebook%20Live-ED4242?style=flat&logo=Facebook%20Live&logoColor=white",
    "/badge/Fandango%20At%20Home-3478C1?style=flat&logo=fandango&logoColor=white",
    "/badge/fire%20tv-fc3b2d?style=flat&logo=amazon%20fire%20tv&logoColor=white",
    "/badge/Fubo-E64526?style=flat&logo=fubo&logoColor=white",
    "/badge/hulu-1CE783?style=flat&logo=hulu&logoColor=white",
    "/badge/kick-53FC18?style=flat&logo=kick&logoColor=white",
    "/badge/Netflix-E50914?style=flat&logo=netflix&logoColor=white",
    "/badge/roku-6f1ab1?style=flat&logo=roku&logoColor=white",
    "/badge/Tubi-6A18F5?style=flat&logo=tubi&logoColor=white",
    "/badge/Twitch-9347FF?style=flat&logo=twitch&logoColor=white",
    "/badge/Youtube%20Gaming-FF0000?style=flat&logo=Youtubegaming&logoColor=white",
    "/badge/Disney-%23006E99?style=flat&logo=disney&logoColor=white",
    "/badge/-cypress-%23E5E5E5?style=flat&logo=cypress&logoColor=058a5e",
    "/badge/-Jasmine-%238A4182?style=flat&logo=Jasmine&logoColor=white",
    "/badge/-jest-%23C21325?style=flat&logo=jest&logoColor=white",
    "/badge/-mocha-%238D6748?style=flat&logo=mocha&logoColor=white",
    "/badge/-playwright-%232EAD33?style=flat&logo=playwright&logoColor=white",
    "/badge/Puppeteer-white?style=flat&logo=Puppeteer&logoColor=black",
    "/badge/-selenium-%43B02A?style=flat&logo=selenium&logoColor=white",
    "/badge/sentry-%23362D59?style=flat&logo=sentry&logoColor=white",
    "/badge/-TestingLibrary-%23E33332?style=flat&logo=testing-library&logoColor=white",
    "/badge/-Vitest-252529?style=flat&logo=vitest&logoColor=FCC72B",
    "/badge/subversion-%23809CC9?style=flat&logo=subversion&logoColor=white",
    "/badge/bitbucket-%230047B3?style=flat&logo=bitbucket&logoColor=white",
    "/badge/forgejo-%23FB923C?style=flat&logo=forgejo&logoColor=white",
    "/badge/git-%23F05033?style=flat&logo=git&logoColor=white",
    "/badge/Gitea-34495E?style=flat&logo=gitea&logoColor=5D9425",
    "/badge/Gitee-C71D23?style=flat&logo=gitee&logoColor=white",
    "/badge/github-%23121011?style=flat&logo=github&logoColor=white",
    "/badge/gitlab-%23181717?style=flat&logo=gitlab&logoColor=white",
    "/badge/gitpod-f06611?style=flat&logo=gitpod&logoColor=white",
    "/badge/mercurial-999999?style=flat&logo=mercurial&logoColor=white",
    "/badge/-PERFORCE%20HELIX-00AEEF?style=flat&logo=Perforce&logoColor=white",
    "/badge/fitbit-00B0B9?style=flat&logo=fitbit&logoColor=white",
    "/badge/AngelList-%23D4D4D4?style=flat&logo=AngelList&logoColor=black",
    "/badge/Behance-1769ff?style=flat&logo=behance&logoColor=white",
    "/badge/Freelancer-29B2FE?style=flat&logo=Freelancer&logoColor=white",
    "/badge/Glassdoor-00A162?style=flat&logo=Glassdoor&logoColor=white",
    "/badge/HackerEarth-%232C3454?style=flat&logo=HackerEarth&logoColor=Blue",
    "/badge/indeed-003A9B?style=flat&logo=indeed&logoColor=white",
    "/badge/UpWork-6FDA44?style=flat&logo=Upwork&logoColor=white",
  ];

  let icls = "height: 62px;";
  let node = html!({
    div class="flex flex-col gap-2 items-center" {
      table {
        @for item in items {
          tr {
            td { img style=(icls) src=(format!("https://img.shields.io{item}")) {} }
            td { img style=(icls) src=(format!("{item}")) {} }
          }
        }
      }
    }
  });

  Ok(layout(None, node))
}

pub async fn index() -> AnyRep<impl IntoResponse> {
  let colors = Color::iter().filter_map(|x| x.to_name());

  #[rustfmt::skip]
  let icons = vec![
    "git", "github", "gitlab", "gitea", "bitbucket", "githubsponsors",
    "circleci", "travisci", "appveyor", "jenkins", "drone", "codecov", "coveralls",
    "bitcoin", "ethereum", "litecoin", "dogecoin", "monero", "ripple", "tether",
    "buymeacoffee", "patreon", "paypal", "liberapay", "opencollective", "kofi",
    "discord", "slack", "telegram", "whatsapp", "signal", "messenger", "line",
    "reddit", "x", "medium", "devto", "hashnode", "ghost", "rss",
    "docker", "kubernetes", "helm", "ansible", "terraform", "vagrant", "puppet",
    "rust", "python", "ruby", "php", "llvm", "javascript", "typescript", "go",
  ];

  let options = vec![
    ("label", "Text shown on the left side"),
    ("labelColor", "Color for the left side"),
    ("value", "Text shown on the right side"),
    ("valueColor", "Color for the right side"),
    ("icon", "Name from Simple Icons library"),
    ("iconColor", "Color for the icon"),
    ("style", "Badge style: flat, flat-square"),
    ("radius", "Border radius in pixels (0-12)"),
  ];

  let static_examples = vec![
    ("/badge/label-message-ff0000", "Fixed badge"),
    ("/badge/label--message-f00", "Fixed badge with dash"),
    ("/badge/label__message-red", "Fixed badge with underscore"),
  ];

  let sec_colors = html! {
    section {
      (heading(3, "Colors"))
      p {
        "Colors can be specified using predefined names or hex values via the "
        code { "?color={COLOR}" } " parameter"
    }
      div class="flex flex-row gap-2 flex-wrap" {
        @for color in colors {
          img class="h20" src=(format!("/badge/?value={color}&color={color}&label=color")) alt=(color) {}
        }
      }
    }
  };

  let sec_icons = html! {
    section {
      (heading(3, "Icons"))
      p {
        "Icons can be added to any badge using the "
        code { "icon" } " and " code { "iconColor" } " parameters. "
        "All available icons are provided by the "
        a href="https://simpleicons.org/" class="contrast" target="_blank" { "Simple Icons" }
        " project."
      }
      div class="flex flex-row gap-2 flex-wrap" {
        @for icon in icons {
          img class="h20" src=(format!("/badge/?icon={icon}&value={icon}")) alt=(icon) {}
        }
      }
    }
  };

  let sec_options = html! {
    section {
      (heading(3, "Options"))
      ul {
        @for (name, desc) in options {
          li { code { (name) } { " " (desc) } }
        }
      }
    }
  };

  let node = html! {
    (sec_colors)
    (sec_icons)
    (sec_options)

    section {
      (heading(3, "Integrations"))
      (render_tbox("Static", static_examples))
      (render_enum::<apis::npm::Kind>("NPM", "/npm/{}/apigen-ts"))
      (render_enum::<apis::pypi::Kind>("PyPI", "/pypi/{}/twscrape"))
      (render_enum::<apis::crates::Kind>("Crates.io", "/crates/{}/tokio"))
      (render_enum::<apis::dartpub::Kind>("Dart Pub", "/pub/{}/dio"))
      (render_enum::<apis::gems::Kind>("Ruby Gems", "/gem/{}/rails"))
      (render_enum::<apis::hackage::Kind>("Hackage", "/hackage/{}/servant"))
      (render_enum::<apis::hexpm::Kind>("Hex", "/hexpm/{}/jason"))
      (render_enum::<apis::nuget::Kind>("NuGet", "/nuget/{}/Newtonsoft.Json"))
      (render_enum::<apis::packagephobia::Kind>("Packagephobia", "/packagephobia/{}/apigen-ts"))
      (render_enum::<apis::packagist::Kind>("Packagist", "/packagist/{}/laravel/laravel"))
      (render_enum::<apis::clojars::Kind>("Clojars", "/clojars/{}/metosin/jsonista"))
      (render_enum::<apis::cocoapods::Kind>("CocoaPods", "/cocoapods/{}/SwiftyJSON"))
      (render_enum::<apis::puppetforge::Kind>("Puppet Forge", "/puppetforge/{}/puppetlabs/puppetdb"))
      (render_enum::<apis::cpan::Kind>("CPAN", "/cpan/{}/PerlPowerTools"))
      (render_enum::<apis::homebrew::Kind>("Homebrew", "/homebrew/{}/macmon"))
      (render_enum::<apis::homebrew::Kind>("Homebrew Cask", "/homebrew/{}/cask/firefox"))
      (render_enum::<apis::vscode::Kind>("VS Code", "/vscode/{}/esbenp.prettier-vscode"))
      (render_enum::<apis::amo::Kind>("Mozilla Add-ons", "/amo/{}/privacy-badger17"))
      (render_enum::<apis::cws::Kind>("Chrome Web Store", "/cws/{}/epcnnfbjfcgphgdmggkamkmgojdagdnn"))
      (render_enum::<apis::jetbrains::Kind>("JetBrains Plugin", "/jetbrains/{}/22282"))
      (render_enum::<apis::github::Kind>("GitHub", "/github/{}/vladkens/macmon"))
      (render_enum::<apis::docker::Kind>("Docker", "/docker/{}/grafana/grafana"))
    }
  };

  Ok(layout(None, node))
}
