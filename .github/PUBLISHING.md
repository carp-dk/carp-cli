# Publishing

All versions are coming from `[workspace.package] version` in the
root `Cargo.toml`, so they cannot drift apart.

| Channel | What | Name |
| --- | --- | --- |
| GitHub Releases | the `carp` binary, one archive per platform | `carp-<version>-<target>.tar.gz` |
| crates.io | the binary crate and its three libraries | [`carp-dk`](https://crates.io/crates/carp-dk), `carp-client`, `carp-protocol`, `carp-catalog` |
| PyPI | the Python extension module | [`carp-cli`](https://pypi.org/p/carp-cli) |
| Homebrew | a formula naming the release's own archives | [`carp-dk/tap/carp`](https://github.com/carp-dk/homebrew-tap) |

## Releasing

Bump the version and merge to `main`:

```toml
# Cargo.toml
[workspace.package]
version = "0.3.0"
```

That is the whole procedure. `release.yml` reads the version, sees no `v0.3.0`
tag, and builds. A push that does not change the version costs one cheap job
and stops.

Nothing is published until every artefact exists, which is why `publish-crates`
and `publish-pypi` both wait on `build` even though neither uses a binary.

### If a run dies partway

Re-run it. The gate lets it through as long as no tag was created, and both
registry jobs tolerate what they already uploaded — `skip-existing` on PyPI,
and an explicit already-published check per crate on crates.io.

If the tag *was* created but something is wrong, delete the release and the tag
on GitHub, then re-run. Versions already on a registry stay there — pick a new
version rather than trying to replace one.

The tap is the one part that does not need the workflow at all. It reads a
release that is already published, so it can be run against any version, from
anywhere, as often as you like:

## Homebrew tap

```sh
.github/homebrew/tap.sh 0.2.1
```

To install then run:

```sh
brew install carp-dk/tap/carp
carp --version
brew test carp
```

## macOS signing

Optional, and independent of the above. Without it the macOS binaries are
published unsigned and users must clear the quarantine attribute by hand. See
[MACOS_SIGNING.md](MACOS_SIGNING.md).

## Version numbers

Every crate and the Python package share one version, from
`[workspace.package]`. That is a deliberate simplification, not an accident: it
means a bug fix in `carp-protocol` bumps `carp-dk` too, and all four go out
together.

The cost is version churn in the libraries. If `carp-protocol` ever gains
users who track it independently, give it its own `version` in its own
`Cargo.toml` and drop `version.workspace = true` — the release workflow reads
only `carp-dk`'s version, so nothing else has to change.
