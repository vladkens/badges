#![forbid(absolute_paths_not_starting_with_crate)]
pub(crate) mod amo;
pub(crate) mod clojars;
pub(crate) mod cocoapods;
pub(crate) mod crates;
pub(crate) mod cws;
pub(crate) mod dartpub;
pub(crate) mod discord;
pub(crate) mod fixed;
pub(crate) mod gems;
pub(crate) mod github;
pub(crate) mod hackage;
pub(crate) mod hexpm;
pub(crate) mod homebrew;
pub(crate) mod jetbrains;
pub(crate) mod npm;
pub(crate) mod nuget;
pub(crate) mod packagephobia;
pub(crate) mod packagist;
pub(crate) mod pypi;
pub(crate) mod readthedocs;
pub(crate) mod vscode;

pub(crate) fn get_client() -> reqwest::Client {
  let ua = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
  reqwest::Client::builder().user_agent(ua).build().unwrap()
}
