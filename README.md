# CARP CLI

A client for the [Copenhagen Research Platform][carp]. Read your studies, 
participants, deployments, uploaded measurements, exports and files from a
shell, a script, a CI job or a Python notebook.

```sh
carp studies list
carp participants list $STUDY --format csv > participants.csv
carp data query $DEPLOYMENT --device "Primary Phone" \
     --type dk.cachet.carp.heartrate --from 7d --format ndjson
```

```python
import carp

client = carp.Client(env="production")
rows = client.data_stream(deployment=DEPLOYMENT, device="Primary Phone",
                          data_type="dk.cachet.carp.heartrate", start="7d")
frame = carp.to_pandas(rows)
```

## 🚀 Installing

Every version tagged on `main` publishes a binary per platform under
[Releases][releases]. Take the archive for your machine, unpack it, and put
`carp` on your `PATH`:

| Platform | Archive |
| --- | --- |
| Linux, Intel/AMD | `carp-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux, ARM | `carp-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| macOS, Apple silicon | `carp-<version>-aarch64-apple-darwin.tar.gz` |
| macOS, Intel | `carp-<version>-x86_64-apple-darwin.tar.gz` |
| Windows | `carp-<version>-x86_64-pc-windows-msvc.zip` |

```sh
tar -xzf carp-0.2.0-aarch64-apple-darwin.tar.gz
install carp-0.2.0-aarch64-apple-darwin/carp /usr/local/bin/
carp --version
```

`SHA256SUMS` ships beside the archives, so a download can be checked against
what the workflow built:

```sh
sha256sum --check --ignore-missing SHA256SUMS
```

The binaries are not yet code-signed thus macOS quarantines anything a browser
downloaded, and `tar` carries that attribute onto the files it extracts, so an
unsigned `carp` is killed on sight rather than warned about. 
Clearing the attribute is what unblocks it or allow it from Settings -> Security & Privacy:

```sh
xattr -d com.apple.quarantine /usr/local/bin/carp
```

Downloading with `curl`, or building from source, avoids it entirely.

Or from crates.io, which needs a Rust toolchain and nothing else:

```sh
cargo install carp-dk
cargo install carp-dk --no-default-features   # without the browser
cargo install --path .                        # from a checkout
```

The Python module is on PyPI, a wheel per platform plus a source distribution:

```sh
pip install carp-cli
pip install 'carp-cli[pandas]'    # adds to_pandas()
```

The libraries underneath are published too, for anything that wants the client
or the protocol model without the command:
[`carp-client`](https://crates.io/crates/carp-client),
[`carp-protocol`](https://crates.io/crates/carp-protocol),
[`carp-catalog`](https://crates.io/crates/carp-catalog).

[releases]: https://github.com/carp-dk/carp-cli/releases

## 🔐 Signing in

```sh
carp auth login              # opens a browser, once
carp auth status
```

The session is stored per deployment and refreshed as needed.

`carp auth token` prints the bearer token, for a request made by hand. It is a
credential.

## 🤖 Commands

| | |
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

### Getting measurements out

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

### Output

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

### Exit codes

| | |
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
Failures raise `CarpAuthError`, `CarpNotFoundError`, `CarpForbiddenError` or `CarpError`.

See [`packages/carp-python/README.md`](packages/carp-python/README.md).

## The protocol editor

A CARP study is described by a `protocol.json`: which devices take part, what
they measure, when each task runs, and what is asked of the participants.
You can use `carp protocol edit` to open an editor for the same document.
It shows devices, tasks and schedules rather than a tree of objects, and it writes exactly the
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

To update the protocol vocabulary, run `carp protocol sync`. with the 
`GITHUB_TOKEN` environment variable set to a token with access to the upstream repository.
You need to have access to the `carp_study_app_configurations` repository.
> The upstream repository is private. Set `GITHUB_TOKEN` to a token with access
> to it — `export GITHUB_TOKEN=$(gh auth token)` if you use the GitHub CLI.

`check` needs no CARP session and no network, so it works as a pre-commit hook
or a CI step:

```sh
carp protocol check studies/sleep || exit 1
```

`<path>` is a `protocol.json`, or a study directory containing
`carp/resources/protocol.json` — the layout `carp_study_app_configurations`
uses.

## Layout

```
carp-cli
├── src/                     the `carp` command
│   ├── cli.rs               the argument surface
│   ├── commands/            what each one does, one module per noun
│   ├── output/              table, JSON, NDJSON and CSV
│   ├── app/  ui/  studio/   the interactive browser (feature = "tui")
│   └── db/                  its local cache
└── packages/
    ├── carp-client/         session, HTTP client, typed models, transfers
    ├── carp-protocol/       the protocol document: model, serde, validation
    ├── carp-catalog/        upstream sync, versioning, derived vocabulary
    └── carp-python/         the Python extension module
```

Everything that talks to CARP is in `carp-client`, with no dependency on a
terminal. That is what lets the command line and the Python module be two front
ends onto one client rather than two implementations of one.

## Testing

```sh
cargo test --workspace
cargo test --no-default-features   # the command line without the browser
```

The Python bindings have their own suite, against a built wheel:

```sh
cd packages/carp-python
maturin develop
pytest tests
```

## Releasing

Bump `[workspace.package] version` in `Cargo.toml` and merge to `main`. 
The release workflow sees a version with no tag, builds the binaries 
and the wheels, publishes the four crates to crates.io and `carp-cli` 
to PyPI, and then creates the GitHub release and its tag. A push that 
does not change the version does nothing.

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

Flags outrank the environment, an address outranks a name, and the last resort
is production. `carp --help` lists the flags. Values may also be put in a `.env`
beside the binary.

## License

Copyright (c) Copenhagen Research Platform

Licensed under the MIT license ([LICENSE](./LICENSE) or
<http://opensource.org/licenses/MIT>).

[carp]: https://carp.dk
[configs]: https://github.com/carp-dk/carp_study_app_configurations
