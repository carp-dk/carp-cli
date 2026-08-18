# CARP CLI

[![crates.io](https://img.shields.io/crates/v/carp-dk?label=crates.io&logo=rust)](https://crates.io/crates/carp-dk)
[![PyPI](https://img.shields.io/pypi/v/carp-cli?label=PyPI&logo=python&logoColor=white)](https://pypi.org/project/carp-cli/)
[![CI](https://github.com/carp-dk/carp-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/carp-dk/carp-cli/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)

A client for the [Copenhagen Research Platform][carp]. Read your studies,
participants, deployments, uploaded measurements, exports and files from a
shell, a script, a CI job or a Python notebook.

```sh
carp studies list
carp participants list $STUDY --format csv > participants.csv
carp data query $DEPLOYMENT --device "Primary Phone" \
     --type dk.cachet.carp.heartrate --from 7d --format ndjson
```

<details>
<summary><strong>The same thing from Python</strong></summary>

```python
import carp

client = carp.Client(env="production")
rows = client.data_stream(deployment=DEPLOYMENT, device="Primary Phone",
                          data_type="dk.cachet.carp.heartrate", start="7d")
frame = carp.to_pandas(rows)
```

The module ships the same client the command uses, and shares its session.
More in [From Python](#from-python).

</details>

**Contents** — [Install](#install) · [Signing in](#signing-in) ·
[Commands](#commands) · [From Python](#from-python) ·
[The protocol editor](#the-protocol-editor) · [Deployments](#deployments) ·
[Configuration](#configuration) · [Libraries](#libraries) ·
[Contributing](#contributing) · [License](#license)

## Install

The command is `carp` everywhere. The crate is
[`carp-dk`](https://crates.io/crates/carp-dk) and the Python distribution is
[`carp-cli`](https://pypi.org/project/carp-cli/)

<details open>
<summary><strong>Homebrew</strong> — macOS and Linux</summary>

```sh
brew install carp-dk/tap/carp
brew upgrade carp                # later
```

</details>

<details>
<summary><strong>A release archive</strong></summary>

Every build is attached to a [GitHub release][releases]:

| Platform | Archive |
| --- | --- |
| Linux, Intel/AMD | `carp-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux, ARM | `carp-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| macOS, Apple silicon | `carp-<version>-aarch64-apple-darwin.tar.gz` |
| macOS, Intel | `carp-<version>-x86_64-apple-darwin.tar.gz` |
| Windows | `carp-<version>-x86_64-pc-windows-msvc.zip` |

Unpack it and put `carp` on your `PATH`:

```sh
tar -xzf carp-<version>-aarch64-apple-darwin.tar.gz
install carp-<version>-aarch64-apple-darwin/carp /usr/local/bin/
carp --version
```

macOS might refuse to run a `carp` binary that is downloaded from the
release archive, to unblock it, run or allow from Settings → Privacy &
Security:

```sh
xattr -d com.apple.quarantine /usr/local/bin/carp
```

</details>

<details>
<summary><strong>cargo</strong></summary>

```sh
cargo install carp-dk
cargo install carp-dk --no-default-features   # without the browser
cargo install --path .                        # from a checkout
```

</details>

<details>
<summary><strong>pip</strong></summary>

On PyPI as [`carp-cli`](https://pypi.org/project/carp-cli/), a wheel per
platform plus a source distribution:

```sh
pip install carp-cli
pip install 'carp-cli[pandas]'    # adds to_pandas()
```

Installed as `carp-cli`, imported as `carp`. See
[From Python](#from-python).

</details>

## Signing in

```sh
carp auth login              # opens a browser, once
carp auth status
```

The session is stored per deployment and refreshed as needed.

`carp auth token` prints the bearer token, for a request made by hand. It is a
credential.

## Commands

| Command | What it does |
| --- | --- |
| `carp studies list` | Studies you can see |
| `carp studies show <study>` | One, with its staff and participant groups |
| `carp participants list <study>` | Who is enrolled. `--all` walks every page |
| `carp deployments list <study>` | Deployments and how far each has got |
| `carp deployments show <study> <id>` | Every device and participant on one |
| `carp data summary <study>` | How much was collected, by task and day |
| `carp data query <deployment>` | The measurements themselves |
| `carp data statistics <deployment>…` | Upload counts |
| `carp export list\|create\|download\|delete` | Study data exports |
| `carp files list\|download <study>` | Uploaded study files |
| `carp protocol check\|show\|sync\|catalog\|edit` | Study protocols |
| `carp tui` | The interactive browser |
| `carp completions <shell>` | A completion script |

`carp <command> --help` has the flags.

<details>
<summary><strong>Getting measurements out</strong> — data streams, windows, exports</summary>

A *data stream* is one kind of measurement from one device in one deployment,
and that is the level at which you ask for it:

```sh
carp data query $DEPLOYMENT \
    --device "Primary Phone" \
    --type dk.cachet.carp.heartrate \
    --from 2026-08-01 --to 2026-08-08
```

`--from` and `--to` take a date, a full timestamp, or an age — `7d`, `36h`,
`90m`. `--to` defaults to now.

You can also use `--raw` to print the server's response directly.

For the bulk of a study, ask for an export instead. The server packages one in
the background:

```sh
carp export create $STUDY --wait
carp export download $STUDY $EXPORT_ID
```

</details>

<details>
<summary><strong>Output</strong> — table, <code>json</code>, <code>ndjson</code>, <code>csv</code></summary>

Results print as a table when you are looking at them and as JSON when
something else is:

```sh
carp studies list                 # a table, at a terminal
carp studies list | jq '.[].name' # JSON, into a pipe
```

`--format table|json|ndjson|csv` overrides the guess, and `--json` is shorthand
for the second. The table shows selected columns and shortens long values;
`json` has every field the server sent. `ndjson` gives one record per line,
which is what to reach for when a result is large enough that you would rather
not hold it whole.

Anything that is not the result — progress, confirmations, warnings — goes to
stderr, so a pipe carries only the record.

</details>

<details>
<summary><strong>Exit codes</strong></summary>

| Code | Meaning |
| --- | --- |
| `0` | success |
| `1` | failed |
| `2` | the arguments did not parse |
| `3` | not signed in |
| `4` | no such study, deployment, export or file |
| `5` | signed in, but not allowed |

So a script can tell the cases apart without reading the message:

```sh
carp auth status >/dev/null 2>&1 || carp auth login
```

Under `--format json` a failure prints `{"error": {...}}` on stderr, naming the
same cases as `kind`.

</details>

## From Python

`pip install carp-cli` gives the same client as a module. It shares the CLI's
session — `carp auth login` in a terminal signs in the notebook beside it, and
`Client.login()` does the reverse:

```python
import carp

client = carp.Client(env="test")
client.login()                                    # only if not already signed in

for study in client.studies():
    print(study["studyId"], study["name"])

rows = client.data_stream(
    deployment=DEPLOYMENT,
    device="Primary Phone",
    data_type="dk.cachet.carp.heartrate",
    start="7d",
)
frame = carp.to_pandas(rows)
```

Calls block, and return plain lists and dictionaries exactly as CARP sent them.

<details>
<summary><strong>Failures</strong></summary>

| Exception | Raised when |
| --- | --- |
| `carp.CarpAuthError` | no session, or the server rejected it — call `login()` |
| `carp.CarpNotFoundError` | no such study, deployment, export or file |
| `carp.CarpForbiddenError` | signed in, but not allowed to see it |
| `carp.CarpError` | anything else; the base of the three above |

</details>

Full module documentation:
[`packages/carp-python/README.md`](packages/carp-python/README.md).

## The protocol editor

A CARP study is described by a `protocol.json`: which devices take part, what
they measure, when each task runs, and what is asked of the participants.
`carp protocol edit` opens an editor for that same document. It shows devices,
tasks and schedules rather than a tree of objects, and it writes exactly the
JSON the study app expects.

```
 Overview  Devices  Tasks  Triggers  Survey  Participants  Catalog  Checks
╭─ tasks 2/3 ──────────────────────────╮╭─ task ───────────────────────────╮
│▌ Sleep Diary   RPAppTask   2  1 trig ││ name        Sleep Diary          │
│  Step Count    Background  1  1 trig ││ type        RPAppTask            │
│  Monitoring    Monitoring  3  1 trig ││                                  │
│                                      ││ shown to the participant         │
│                                      ││ card type   survey               │
╰──────────────────────────────────────╯│ title       How did you sleep?   │
                                        │                                  │
                                        │ started by                       │
                                        │   • daily at 20:00, on Primary…  │
                                        ╰──────────────────────────────────╯
 a add · e edit · x remove · m measures · Enter survey · s save · z undo
```

`<path>` is a `protocol.json`, or a study directory containing
`carp/resources/protocol.json` — the layout that
[`carp_study_app_configurations`][configs] uses.

<details>
<summary><strong>Checking a protocol in CI</strong> — no session, no network</summary>

`carp protocol check` needs no CARP session and no network, so it works as a
pre-commit hook or a CI step:

```sh
carp protocol check studies/sleep || exit 1
```

</details>

<details>
<summary><strong>Updating the vocabulary</strong> — <code>carp protocol sync</code></summary>

`carp protocol sync` updates the protocol vocabulary from the upstream
configurations. That repository is private, so it needs `GITHUB_TOKEN` set to a
token with access to it:

```sh
export GITHUB_TOKEN=$(gh auth token)   # if you use the GitHub CLI
carp protocol sync
```

What is recorded and where it is kept:
[`packages/carp-catalog/README.md`](packages/carp-catalog/README.md).

</details>

## Deployments

| `--env` | Address |
| --- | --- |
| `production` | `https://carp.computerome.dk`  |
| `test` | `https://test.carp.dk` |
| `dev` | `https://dev.carp.dk` |

```sh
carp --env dev studies list       # or CARP_ENV=dev
carp --env test protocol sync
carp studies list                 # production
```

Each deployment keeps its own session and its own cache, keyed by host, so
moving between them neither signs you out of the other nor mixes their studies
together. Anywhere else is reachable by address with `--server`, which
overrides `--env`.

## Configuration

Flags outrank the environment, an address outranks a name, and the last resort
is production. `carp --help` lists the flags. Values may also be put in a `.env`
beside the binary.

<details>
<summary><strong>Environment variables</strong></summary>

| Variable | Meaning |
| --- | --- |
| `CARP_ENV` | `production` (default), `test` or `dev` |
| `CARP_SERVER` | Base URL of the CARP web service; overrides `CARP_ENV` |
| `CARP_REALM` | Keycloak realm (default `Carp`) |
| `CARP_CLIENT_ID` | Public OAuth2 client id (default `carp-cli`) |
| `CARP_DATA_DIR` | Where the session, the cache and the catalogue are stored |
| `CARP_DOWNLOAD_DIR` | Where exports, study files and protocols are written |
| `CARP_PORTAL_URL` | Base address of the CARP web portal |
| `CARP_ICONS` | `symbols` (default), `emoji` or `none` |
| `GITHUB_TOKEN` | Access to the private upstream configurations repository |

</details>

## Libraries

| Package | What it is | Documentation |
| --- | --- | --- |
| [`carp-client`](https://crates.io/crates/carp-client) | The web service client: deployments, sessions, one function per API operation | [README](packages/carp-client/README.md) |
| [`carp-protocol`](https://crates.io/crates/carp-protocol) | The study protocol as a Rust domain model | [README](packages/carp-protocol/README.md) |
| [`carp-catalog`](https://crates.io/crates/carp-catalog) | The vocabulary a protocol can be written in | [README](packages/carp-catalog/README.md) |
| [`carp-cli`](https://pypi.org/project/carp-cli/) | The Python module, built on `carp-client` | [README](packages/carp-python/README.md) |

## Contributing

| | |
| --- | --- |
| [Getting set up](.github/CONTRIBUTING.md#getting-set-up) | What to install, and how to build and run it |
| [The layout](.github/CONTRIBUTING.md#the-layout) | Which crate holds what |
| [The checks](.github/CONTRIBUTING.md#the-checks) | What CI runs before a change lands |
| [Sending a change](.github/CONTRIBUTING.md#sending-a-change) | Branches, commits, pull requests |
| [PUBLISHING.md](.github/PUBLISHING.md) | Releasing one version to crates.io, PyPI, GitHub Releases and Homebrew |
| [MACOS_SIGNING.md](.github/MACOS_SIGNING.md) | Signing and notarising the macOS binaries |

## License

Copyright (c) Copenhagen Research Platform

Licensed under the MIT license ([LICENSE](./LICENSE) or
<http://opensource.org/licenses/MIT>).

[carp]: https://carp.dk
[releases]: https://github.com/carp-dk/carp-cli/releases
[configs]: https://github.com/carp-dk/carp_study_app_configurations
