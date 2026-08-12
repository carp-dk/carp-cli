# carp-cli

A terminal client for the [Copenhagen Research Platform][carp]: browse studies,
participants, deployments and exports — and author the study protocols those
studies run on.

```
carp                       # the study browser
carp protocol              # the protocol editor
```

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

### What it does

- **Every part of a protocol.** Devices (both CARP namespaces, including the
  CAMS 2.0 classes), background and app tasks, all nine trigger kinds, Research
  Package surveys with branching, participant roles and expected data, and the
  data endpoint.
- **Keeps references intact.** A protocol is joined by name — a trigger names
  its device, a task control names a task. Renaming a device moves every
  reference with it; removing one takes the triggers and controls that could
  only have referred to it, and says what it took. `z` undoes any of it.
- **Checks before you deploy.** The Checks tab reports what the schema cannot:
  a name that does not resolve, an identifier used twice, a task nothing
  starts, a survey branch that jumps to a step that no longer exists.
- **Starts from a real study.** The Catalog tab lists the upstream studies;
  `Enter` forks one into a new protocol of your own.
- **Uploads to CARP.** `u` stores the protocol as a new version under a version
  tag, choosing `Add` or `AddVersion` by what CAWS already holds.

### Where its vocabulary comes from

Which measure types exist, which health metrics can be read, which question
types a survey supports — none of that is fixed in this tool. Every sampling
package a study app links in contributes its own, and the set changes release
to release.

So it is not hard-coded: it is **derived** from the protocols in
`carp_study_app_configurations`, and pinned to the commit it was derived from.

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
carp-cli-rust
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

## Configuration

| Variable | Meaning |
| --- | --- |
| `CARP_SERVER` | Base URL of the CARP web service |
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

Copyright (c) Alireza Hajebrahimi <6937697+iarata@users.noreply.github.com>

Licensed under the MIT license ([LICENSE](./LICENSE) or
<http://opensource.org/licenses/MIT>).

[carp]: https://carp.cachet.dk
[configs]: https://github.com/carp-dk/carp_study_app_configurations
