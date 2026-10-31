# Copyright (c) vladkens | MIT License | https://github.com/vladkens/badges
# /// script
# requires-python = ">=3.11"
# ///
"""Checks that badges backed by upstream APIs still show real values.

Each case fetches a badge as JSON and fails on an error badge, a non-JSON response, an empty
value, "unknown" or a value that doesn't match its regex. A failing badge is still HTTP 200 with
an SVG, so a case is retried on a failed check, not on the status.

uv run scripts/smoke.py   (BASE_URL=http://localhost:8080 to check a local server)
"""

import json
import os
import re
import sys
import time
import urllib.request
from http.client import HTTPException

BASE_URL = os.environ.get("BASE_URL", "https://badges.ws")
ATTEMPTS = 3
TIMEOUT = 20  # seconds per request
PAUSE = 61  # seconds between attempts: providers cache Ok values for 60s, a sooner retry sees the same

# <path> <regex the badge value must match>
CASES = r"""
/npm/v/react                                          ^v[0-9]+\.[0-9]+\.[0-9]+
/npm/l/react                                          ^MIT$
/npm/dw/react                                         ^[1-9][0-9.]*[kMBT]?/week$
/npm/dm/react                                         ^[1-9][0-9.]*[kMBT]?/month$
/npm/dt/react                                         ^[1-9][0-9.]*[kMBT]?$

/pypi/v/requests                                      ^v[0-9]+\.[0-9]+\.[0-9]+
/pypi/l/requests                                      ^Apache-2\.0$
/pypi/dw/requests                                     ^[1-9][0-9.]*[kMBT]?/week$
/pypi/dm/requests                                     ^[1-9][0-9.]*[kMBT]?/month$
/pypi/dt/requests                                     ^[1-9][0-9.]*[kMBT]?$
/pypi/python/requests                                 ^3\.[0-9]+
/pypi/wheel/requests                                  ^yes$
/pypi/status/requests                                 ^stable$
/pypi/implementation/requests                         pypy

/packagephobia/publish/react                          ^[1-9][0-9.]* [kMG]?B$
/packagephobia/install/react                          ^[1-9][0-9.]* [kMG]?B$

/crates/v/tokio                                       ^v[0-9]+\.[0-9]+\.[0-9]+
/crates/l/tokio                                       ^MIT$
/crates/dw/tokio                                      ^[1-9][0-9.]*[kMBT]?/week$
/crates/dm/tokio                                      ^[1-9][0-9.]*[kMBT]?/month$
/crates/dt/tokio                                      ^[1-9][0-9.]*[kMBT]?$
/crates/msrv/tokio                                    ^v1\.[0-9]+
/crates/docs/tokio                                    ^passing$
/crates/size/tokio                                    ^[1-9][0-9.]* [KMG]iB$
/crates/deps/tokio                                    ^(none|up to date|maybe insecure|[0-9]+ of [0-9]+ outdated)$

/cocoapods/v/Alamofire                                ^v[0-9]+\.[0-9]+\.[0-9]+
/cocoapods/l/Alamofire                                ^MIT$
/cocoapods/p/Alamofire                                ios

/clojars/v/ring/ring-core                             ^\[ring/ring-core "[0-9]+\.[0-9]+
/clojars/l/ring/ring-core                             MIT
/clojars/dt/ring/ring-core                            ^[1-9][0-9.]*[kMBT]?$

/packagist/v/laravel/framework                        ^v[0-9]+\.[0-9]+\.[0-9]+
/packagist/l/laravel/framework                        ^MIT$
/packagist/dw/laravel/framework                       ^[1-9][0-9.]*[kMBT]?/week$
/packagist/dm/laravel/framework                       ^[1-9][0-9.]*[kMBT]?/month$
/packagist/dt/laravel/framework                       ^[1-9][0-9.]*[kMBT]?$
/packagist/php/laravel/framework                      [0-9]+\.[0-9]+
/packagist/stars/laravel/framework                    ^[1-9][0-9.]*[kMBT]?$
/packagist/dependents/laravel/framework               ^[1-9][0-9.]*[kMBT]?$

/gem/v/rails                                          ^v[0-9]+\.[0-9]+\.[0-9]+
/gem/l/rails                                          ^MIT$
/gem/dt/rails                                         ^[1-9][0-9.]*[kMBT]?$
/gem/ruby/rails                                       [0-9]+\.[0-9]+

/pub/v/http                                           ^v[0-9]+\.[0-9]+\.[0-9]+
/pub/l/http                                           ^bsd-3-clause$
/pub/dw/http                                          ^[1-9][0-9.]*[kMBT]?/week$
/pub/dm/http                                          ^[1-9][0-9.]*[kMBT]?/month$

/hackage/v/aeson                                      ^v[0-9]+\.[0-9]+\.[0-9]+
/hackage/l/aeson                                      ^BSD-3-Clause$

/hexpm/v/jason                                        ^v[0-9]+\.[0-9]+\.[0-9]+
/hexpm/l/jason                                        ^Apache-2\.0$
/hexpm/dw/jason                                       ^[1-9][0-9.]*[kMBT]?/week$
/hexpm/dm/jason                                       ^[1-9][0-9.]*[kMBT]?/month$
/hexpm/dt/jason                                       ^[1-9][0-9.]*[kMBT]?$

/nuget/v/Newtonsoft.Json                              ^v[0-9]+\.[0-9]+\.[0-9]+
/nuget/dt/Newtonsoft.Json                             ^[1-9][0-9.]*[kMBT]?$

/puppetforge/version/puppetlabs/stdlib                ^v[0-9]+\.[0-9]+\.[0-9]+
/puppetforge/dt/puppetlabs/stdlib                     ^[1-9][0-9.]*[kMBT]?$

/cpan/v/Moose                                         ^v[0-9]+\.[0-9]+
/cpan/l/Moose                                         ^perl_5$

/homebrew/v/wget                                      ^v[0-9]+\.[0-9]+\.[0-9]+
/homebrew/dm/wget                                     ^[1-9][0-9.]*[kMBT]?/month$
/homebrew/dy/wget                                     ^[1-9][0-9.]*[kMBT]?/year$
/homebrew/v/cask/firefox                              ^v[0-9]+\.[0-9]+
/homebrew/dm/cask/firefox                             ^[1-9][0-9.]*[kMBT]?/month$
/homebrew/dy/cask/firefox                             ^[1-9][0-9.]*[kMBT]?/year$

/vscode/v/ms-python.python                            ^v[0-9]+\.[0-9]+\.[0-9]+
/vscode/i/ms-python.python                            ^[1-9][0-9.]*[kMBT]?$
/vscode/d/ms-python.python                            ^[1-9][0-9.]*[kMBT]?$

/amo/v/ublock-origin                                  ^v[0-9]+\.[0-9]+\.[0-9]+
/amo/l/ublock-origin                                  ^GPL-3\.0
/amo/dw/ublock-origin                                 ^[1-9][0-9.]*[kMBT]?/week$
/amo/rating/ublock-origin                             ^[1-5]\.[0-9]/5$
/amo/users/ublock-origin                              ^[1-9][0-9.]*[kMBT]?$

/cws/v/fmkadmapgofadopljbjfkapdkoienihi               ^v[0-9]+\.[0-9]+\.[0-9]+
/cws/rating/fmkadmapgofadopljbjfkapdkoienihi          ^[1-5]\.[0-9]/5$
/cws/rating-count/fmkadmapgofadopljbjfkapdkoienihi    ^[1-9][0-9.]*[kMBT]?$
/cws/stars/fmkadmapgofadopljbjfkapdkoienihi           ^★
/cws/users/fmkadmapgofadopljbjfkapdkoienihi           ^[1-9][0-9.]*[kMBT]?$

/jetbrains/v/164                                      ^v[0-9]+\.[0-9]+\.[0-9]+
/jetbrains/dt/164                                     ^[1-9][0-9.]*[kMBT]?$
/jetbrains/score/164                                  ^[1-5]\.[0-9]/5$
/jetbrains/stars/164                                  ^★

/github/json/badges/shields/master/package.json?query=name  ^shields\.io$
/github/release/facebook/react                        ^v[0-9]+\.[0-9]+\.[0-9]+
/github/assets-dl/BurntSushi/ripgrep                  ^[1-9][0-9.]*[kMBT]?$
/github/l/facebook/react                              ^MIT$
/github/stars/facebook/react                          ^[1-9][0-9.]*[kMBT]?$
/github/forks/facebook/react                          ^[1-9][0-9.]*[kMBT]?$
/github/watchers/facebook/react                       ^[1-9][0-9.]*[kMBT]?$
/github/last-commit/facebook/react                    ^(today|yesterday|[0-9]+ days ago)$
/github/repo-size/facebook/react                      ^[1-9][0-9.]* [KMG]iB$
/github/lang/facebook/react                           ^[a-z]
/github/lang-top/facebook/react                       ^[1-9][0-9.]*%$
/github/lang-count/facebook/react                     ^[1-9][0-9]*$
/github/lang-size/facebook/react                      ^[1-9][0-9.]* [KMG]iB$
/github/contributors/facebook/react                   ^[1-9][0-9.]+[kMBT]?$
/github/workflow/actions/checkout/test                ^passing$

/codecov/c/github/codecov/example-python              ^[1-9][0-9]*%$
/codecov/c/github/codecov/example-python/main         ^[1-9][0-9]*%$

/docker/v/library/nginx                               ^v.
/docker/image-size/library/nginx                      ^[1-9][0-9.]* [KMG]iB$
/docker/image-size/library/nginx/latest               ^[1-9][0-9.]* [KMG]iB$
/docker/pulls/library/nginx                           ^[1-9][0-9.]*[kMBT]?$
/docker/stars/library/nginx                           ^[1-9][0-9.]*[kMBT]?$

/readthedocs/status/pip                               ^passing$
/discord/online/267624335836053506                    ^[1-9][0-9]* online$

/youtube/channel/views/UC8ENHE5xdFSwx71u3fDH5Xw       ^[1-9][0-9.]*[kMBT]?$
/youtube/channel/videos/UC8ENHE5xdFSwx71u3fDH5Xw      ^[1-9][0-9.]*[kMBT]?$
/youtube/channel/subscribers/UC8ENHE5xdFSwx71u3fDH5Xw ^[1-9][0-9.]*[kMBT]?$
/youtube/views/dQw4w9WgXcQ                            ^[1-9][0-9.]*[kMBT]?$
/youtube/likes/dQw4w9WgXcQ                            ^[1-9][0-9.]*[kMBT]?$
"""


def url(path: str) -> str:
  # `.json` selects the format, except where the path has a query (as `/github/json/` always does)
  if "?" in path:
    return f"{BASE_URL}{path}&format=json"

  return f"{BASE_URL}{path}.json"


def check(path: str, regex: str) -> str | None:
  """Why the badge is wrong, or None when it is fine."""
  try:
    with urllib.request.urlopen(url(path), timeout=TIMEOUT) as rep:
      body = rep.read().decode(errors="replace")
  except (OSError, HTTPException) as e:
    return f"request failed: {e}"

  if not body:
    return "no response"

  try:
    x = json.loads(body)
  except ValueError:
    x = None

  if not isinstance(x, dict):
    m = re.search(r'aria-label="[^"]*"', body)
    return f"not json: {m[0] if m else body[:100]}"

  label, value = x.get("label"), x.get("value")
  if label == "error" or not value or "unknown" in str(value) or not re.search(regex, str(value)):
    return f"label={label} value={value}"

  return None


def main():
  cases = [line.split(maxsplit=1) for line in CASES.splitlines() if line.strip()]
  failed = 0

  for path, regex in cases:
    for attempt in range(ATTEMPTS):
      if attempt:
        time.sleep(PAUSE)

      error = check(path, regex)
      if error is None:
        break

    if error is not None:
      failed += 1
      print(f"FAIL {path}: {error}", flush=True)

  print(f"{len(cases) - failed}/{len(cases)} passed against {BASE_URL}")
  if failed:
    sys.exit(1)


if __name__ == "__main__":
  main()
