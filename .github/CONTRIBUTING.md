# Contributing

- [Getting set up](#getting-set-up)
- [The layout](#the-layout)
- [The checks](#the-checks)
- [The Python bindings](#the-python-bindings)
- [The protocol catalogue](#the-protocol-catalogue)
- [Sending a change](#sending-a-change)
- [House style](#house-style)
- [Releasing and distribution](#releasing-and-distribution)
- [Licence](#licence)

## Getting set up

Rust 1.90 or newer, stable, on the 2024 edition. Nothing else is required —
the interactive browser, the HTTP client and the local cache are all crates.

```sh
git clone https://github.com/carp-dk/carp-cli
cd carp-cli
cargo build
cargo run -- studies list
```

`cargo run -- <args>` is the command you will type most; everything after `--`
reaches `carp` unchanged. It talks to production unless told otherwise, and a
deployment is picked per invocation:

```sh
cargo run -- --env dev auth login
cargo run -- --env dev studies list
```

`dev` and `test` are separate deployments with separate sessions and separate
caches, so working against one leaves the others alone. Anything you would put
on the command line can also go in a `.env` beside the binary, which is
gitignored — see [Configuration](../README.md#configuration) for the variables.

There is one default that is not a preference: with no flags and no
environment, `carp` talks to `https://carp.computerome.dk`. A test asserts it
(`the_default_is_production`, in `packages/carp-client/src/config.rs`) because
a default switched to `dev` for an afternoon's convenience is exactly the kind
of thing that reaches someone's machine.

## The layout

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
ends onto one client rather than two implementations of one, and it is the rule
to keep: a new endpoint belongs in `carp-client`, and `src/commands/` should
only be arranging what it returns.

The interactive browser sits behind the `tui` feature, on by default. Off, the
build drops `ratatui`, `crossterm` and the local cache, and `carp` is a plain
command line tool — which is what the Python wheel links against, so the
feature has to stay genuinely optional rather than optional in name.

## The checks

CI runs these on every pull request — the suite on Linux, macOS and Windows,
the rest on Linux. Running them locally first is faster than finding out from a
red tick:

| Command | What it is for |
| --- | --- |
| `cargo fmt --all --check` | formatting, not negotiable |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | a warning locally is a failure here |
| `cargo test --locked --workspace --all-targets` | the suite |
| `cargo test --locked --workspace --doc` | the doctests, which `--all-targets` skips |
| `cargo test --locked --no-default-features --all-targets` | the command line with no browser |

Two of those deserve their reasons written down.

**Not `--all-features`.** `extension-module` on `carp-python` tells pyo3 not to
link libpython, which is right for a wheel and wrong for a test binary. The
feature exists for maturin, not for you.

**`--doc` separately.** `--all-targets` silently excludes doctests, and
`carp-protocol`'s are the documentation people read first, so they are run on
their own.

The documentation build needs nightly, for `doc_cfg` — without it the
`tui`-gated modules are simply missing from the docs rather than marked as
conditional:

```sh
RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo +nightly doc --workspace --no-deps
```

The workflows are linted too, since a mistake in `release.yml` is otherwise
found by a release failing:

```sh
actionlint                      # the workflows, shellcheck included
shellcheck .github/homebrew/*.sh   # the scripts they call
```

`Cargo.lock` is committed and every command above passes `--locked`. A change
that moves a dependency moves the lock file with it, in the same commit.

## The Python bindings

`carp-python` is a pyo3 extension module built with maturin, published to PyPI
as `carp-cli` and imported as `carp`. It has its own suite, run against a built
wheel rather than the source:

```sh
cd packages/carp-python
python -m venv .venv && source .venv/bin/activate
pip install maturin pytest pandas
maturin develop
pytest tests
```

`maturin develop` compiles the extension into the active environment. An
extension module can compile and still fail to load, so the test that matters
most is the first `import carp`.

The wheels are `abi3-py39`: one wheel per platform, loadable by every CPython
from 3.9 up. If you touch the pyo3 surface, keep it inside abi3 — CI builds on
3.12 and then imports the result on 3.9 to make sure the promise holds.

More about the module itself is in
[`packages/carp-python/README.md`](../packages/carp-python/README.md).

## The protocol catalogue

`carp protocol sync` derives the protocol vocabulary from
`carp_study_app_configurations`, which is private. To run it you need
`GITHUB_TOKEN` set to a token with access to that repository:

```sh
export GITHUB_TOKEN=$(gh auth token)
cargo run -- protocol sync
```

Everything else about protocols works offline. `carp protocol check` in
particular needs no CARP session and no network at all, which is what makes it
usable as a pre-commit hook or a CI step.

## Sending a change

Branch off `main`, open a pull request against it, and let CI finish. There is
no template to fill in and no minimum size — a corrected sentence in a comment
is a welcome change.

Two things make a change easy to take:

- **Say why in the commit, not just what.** Subjects are written in the
  imperative and describe the effect: *Stop the install example naming a
  version that has no archives*. Some of the older history carries
  `feat:`/`fix:` prefixes; they are not required.
- **Bring the test with it.** A bug fix that cannot fail on the old code is
  hard to keep fixed.

Do not bump `[workspace.package] version` in a normal pull request. That
version is the release trigger — merging a bump to `main` publishes to
crates.io and PyPI, and neither can be taken back. See
[Releasing and distribution](#releasing-and-distribution).

## House style

The code is commented more heavily than most, and deliberately so: the comments
explain *why* a thing is the way it is, on the assumption that what it does is
already visible in the code. `Cross.toml` is the extreme example — twenty lines
about a compiler from 2016 — and it exists so that nobody has to rediscover the
same link error.

That is the standard to write to. In practice:

- `//!` at the top of a module says what it is for and what decision shaped it.
- A comment earns its place by saying something the code cannot: a constraint,
  a rejected alternative, a reason an obvious simplification does not work.
- Documentation is prose. Full sentences, and no abbreviations that a reader
  arriving at this file for the first time would have to look up.
- User-facing output is prose too. Results go to stdout, everything else to
  stderr, so that a pipe carries only the record.

## Releasing and distribution

Only maintainers can run these, but the documents are here for anyone who wants
to know how a version reaches a machine.

| Document | What it covers |
| --- | --- |
| [PUBLISHING.md](PUBLISHING.md) | The four channels — crates.io, PyPI, GitHub Releases, Homebrew — how one version reaches all of them, the order they publish in and why, and the one-time setup each needs |
| [MACOS_SIGNING.md](MACOS_SIGNING.md) | The Developer ID certificate and App Store Connect key that let the release sign and notarize the macOS binaries, and the six secrets they become |

The short version: bump `[workspace.package] version` in `Cargo.toml` and merge
to `main`. The release workflow sees a version with no tag and does the rest.
A push that does not change the version costs one cheap job and stops.

Both documents describe setup that is *optional* in the sense that a release
still succeeds without it — an unsigned macOS binary, a Homebrew tap left
pointing at the previous version — and each says so where it applies.

## Licence

MIT, as in [LICENSE](../LICENSE). A change offered here is offered under the
same terms.
