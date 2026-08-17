# CARP CLI

A terminal client for the [Copenhagen Research Platform][carp]: browse studies,
participants, deployments and exports — and author the study protocols those
studies run on.

```
carp                       # the study browser
carp protocol              # the protocol editor
```

## Installing

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

The binaries are not yet code-signed. macOS quarantines anything a browser
downloaded, and `tar` carries that attribute onto the files it extracts, so an
unsigned `carp` is killed on sight rather than warned about — no dialog, just
`Killed: 9`. Clearing the attribute is what unblocks it:

```sh
xattr -d com.apple.quarantine /usr/local/bin/carp
```

Downloading with `curl`, or building from source, avoids it entirely.

Building it yourself needs a Rust toolchain and nothing else:

```sh
cargo install --path .
```

[releases]: https://github.com/carp-dk/carp-cli/releases

## The protocol editor

A CARP study is described by a `protocol.json`: which devices take part, what
they measure, when each task runs, and what is asked of the participants.
Until now those documents were produced by
[`carp_study_app_configurations`][configs] — a Flutter project per study, whose
`main()` assembles Dart objects and whose test serialises them. It works, but
authoring a protocol means writing Dart, and reviewing one means reading JSON.

`carp protocol` opens an editor for the same document. It shows devices, tasks
and schedules rather than a tree of objects, and it writes exactly the JSON the
study app expects.

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

### Can handle:

- Devices (both CARP namespaces, including the
  CAMS 2.0 classes), background and app tasks, all nine trigger kinds, Research
  Package surveys with branching, participant roles and expected data, and the
  data endpoint.
- A protocol is joined by name — a trigger names
  its device, a task control names a task. Renaming a device moves every
  reference with it; removing one takes the triggers and controls that could
  only have referred to it, and says what it took. `z` undoes any of it.
- The Checks tab reports what the schema cannot:
  a name that does not resolve, an identifier used twice, a task nothing
  starts, a survey branch that jumps to a step that no longer exists.
- The Catalog tab lists the upstream studies;
  `Enter` forks one into a new protocol of your own.
- `u` stores the protocol as a new version under a version
  tag, choosing `Add` or `AddVersion` by what CAWS already holds.

The cli depends on the CARP study app configurations repository for its vocabs and functionality availability.
```
$ carp protocol sync
syncing from carp-dk/carp_study_app_configurations…
catalogue downloaded at 74f543e (11 studies)
  commit    74f543e65bc18300c61a967cf6c3f13e228eabf9 - Merge pull request #45…
  dated     2026-08-11T12:38:39Z
  learned   43 measure types, 33 health metrics, 16 device classes
  templates 11
```

The Catalog tab names that commit, and says when upstream has moved past it.
Syncing is something you ask for, never something that happens under you: a
value that was in a picker a moment ago should not vanish mid-edit.

> The upstream repository is private. Set `GITHUB_TOKEN` to a token with access
> to it — `export GITHUB_TOKEN=$(gh auth token)` if you use the GitHub CLI.

### Commands

| Command | What it does |
| --- | --- |
| `carp protocol` | Open the editor on a new protocol |
| `carp protocol <path>` | Open the editor on an existing one |
| `carp protocol check <path>` | Validate; exits non-zero on any error |
| `carp protocol show <path>` | Print its devices, tasks and schedules |
| `carp protocol sync` | Download the upstream studies, record the commit |
| `carp protocol catalog` | Report what the stored catalogue holds, offline |

`<path>` is a `protocol.json`, or a study directory containing
`carp/resources/protocol.json` — the layout `carp_study_app_configurations`
uses.

`check` needs no CARP session and no network, so it works as a pre-commit hook
or a CI step:

```sh
carp protocol check studies/sleep || exit 1
```

## Layout

```
carp-cli
├── src/                     the terminal application
│   ├── api/                 HTTP client, typed models, one fn per operation
│   ├── app/                 state, key handling, background tasks
│   │   └── form/            the editing surface: fields, forms, pickers
│   ├── studio/              the protocol editor's own state and actions
│   └── ui/                  rendering, one module per screen
└── packages/
    ├── carp-protocol/       the protocol document: model, serde, validation
    └── carp-catalog/        upstream sync, versioning, derived vocabulary
```

`carp-protocol` and `carp-catalog` are libraries with no dependency on the
terminal, so the protocol model can be reused by anything else that needs it.

## Testing

```sh
cargo test --workspace
```

`packages/carp-protocol/tests/corpus/` holds the `protocol.json` of every study
in `carp_study_app_configurations`, vendored at the commit named in
`SOURCE.txt`. Those files are the specification this crate is written against,
so they are what it is tested against:

- every protocol is parsed, re-serialised and compared field for field
- none may fall back to the preserve-verbatim path, which would let a modelling
  gap pass the first test unnoticed
- every protocol must validate cleanly, bar the upstream defects listed in
  `KNOWN_UPSTREAM_DEFECTS`

Refresh the corpus with `carp protocol sync` and copy the documents out of the
snapshot it writes.

## Deployments

Three CARP deployments are known by name, and a released binary talks to
production unless told otherwise:

| `--env` | Address | |
| --- | --- | --- |
| `production` | `https://carp.computerome.dk` | live data — the default |
| `test` | `https://test.carp.dk` | staging: what production becomes next |
| `dev` | `https://dev.carp.dk` | where server work lands first |

```sh
carp --env dev            # or CARP_ENV=dev
carp --env test protocol sync
carp                      # production
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
| `CARP_DATA_DIR` | Where tokens, the cache and the catalogue are stored |
| `CARP_DOWNLOAD_DIR` | Where exports, study files and protocols are written |
| `CARP_PORTAL_URL` | Base address of the CARP web portal |
| `CARP_ICONS` | `symbols` (default), `emoji` or `none` |
| `GITHUB_TOKEN` | Access to the private upstream configurations repository |

`carp --help` lists the flags. Values may also be put in a `.env` beside the
binary.

## License

Copyright (c) Copenhagen Research Platform

Licensed under the MIT license ([LICENSE](./LICENSE) or
<http://opensource.org/licenses/MIT>).

[carp]: https://carp.dk
