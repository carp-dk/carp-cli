# Publishing

One version, three channels. All come from `[workspace.package] version` in the
root `Cargo.toml`, so they cannot drift apart.

| Channel | What | Name |
| --- | --- | --- |
| GitHub Releases | the `carp` binary, one archive per platform | `carp-<version>-<target>.tar.gz` |
| crates.io | the binary crate and its three libraries | [`carp-dk`](https://crates.io/crates/carp-dk), `carp-client`, `carp-protocol`, `carp-catalog` |
| PyPI | the Python extension module | [`carp-cli`](https://pypi.org/p/carp-cli) |

### Why three different names

The registries were not free to agree.

- **crates.io**: `carp` was taken in 2016 by an unrelated CARP (the Common
  Address Redundancy Protocol), and `carp-cli` by an actively published crate
  for something else entirely. Hence `carp-dk`, after the organisation.
- **PyPI**: `carp` was taken in 2012 by an abandoned templating package.
  `carp-cli` was free, and a distribution name has never had to match what you
  import — so `pip install carp-cli` still gives `import carp`.
- The **command** is `carp` everywhere. That is what people type, and it is
  unaffected by either.

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

The order matters and is deliberate:

1. **test** — the Linux suite, on the exact commit about to be published
2. **build / wheels / sdist** — binaries per platform, wheels per platform, one
   source distribution
3. **publish-pypi** and **publish-crates**, in parallel — the two registries
4. **publish** — creates the GitHub release *and the tag*

Both registries are irreversible: neither a PyPI version nor a crates.io
version can ever be re-uploaded, even after yanking. The git tag *is*
reversible — and the tag is what the gate checks. So the steps that cannot be
repeated run while a re-run is still possible. If a registry fails, no tag
exists and the run can simply be re-run.

Nothing is published until every artefact exists, which is why `publish-crates`
and `publish-pypi` both wait on `build` even though neither uses a binary.

### If a run dies partway

Re-run it. The gate lets it through as long as no tag was created, and both
registry jobs tolerate what they already uploaded — `skip-existing` on PyPI,
and an explicit already-published check per crate on crates.io.

If the tag *was* created but something is wrong, delete the release and the tag
on GitHub, then re-run. Versions already on a registry stay there — pick a new
version rather than trying to replace one.

## One-time setup: PyPI Trusted Publishing

**The first release will fail until this is done.** It stores no token
anywhere: GitHub proves the workflow's identity with a short-lived OIDC token,
and PyPI checks it against the publisher configured below.

Because `carp-cli` does not exist on PyPI yet, this is a *pending* publisher —
it creates the project on the first successful upload.

1. Sign in at <https://pypi.org> with an account that will own the project.
2. Go to **Your account → Publishing → Add a new pending publisher**, or
   <https://pypi.org/manage/account/publishing/>.
3. Fill in exactly:

   | Field | Value |
   | --- | --- |
   | PyPI Project Name | `carp-cli` |
   | Owner | `carp-dk` |
   | Repository name | `carp-cli` |
   | Workflow name | `release.yml` |
   | Environment name | `pypi` |

   The environment name is not optional here: `release.yml` declares
   `environment: pypi`, and PyPI rejects a token whose environment does not
   match what was configured.

4. Nothing to add to GitHub. The `pypi` environment is created by the first run
   that references it.

## One-time setup: crates.io Trusted Publishing

Same idea as PyPI, with one significant difference: **crates.io has no pending
publishers.** A trusted publisher can only be configured on a crate that
already exists, so the first version of each of the four crates has to be
published by hand. After that, CI does it.

### 1. Publish each crate once, manually

In dependency order, from a clean checkout of the commit you want released:

```sh
cargo login                 # a token from https://crates.io/settings/tokens
cargo publish --dry-run --workspace   # verifies all four before anything is sent

cargo publish -p carp-protocol
cargo publish -p carp-catalog
cargo publish -p carp-client
cargo publish -p carp-dk
```

Order is not optional: each crate depends on the published version of the ones
before it, and `cargo publish` waits for the index before returning. The
dry-run resolves all four against a temporary registry, so it catches a
metadata or packaging error before the first real upload — which matters,
because a failure halfway through leaves some crates published at a version the
others never reach.

`carp-python` is not in the list. It is `publish = false`: a cdylib built for
one interpreter ABI is of no use as a Rust dependency, and it goes to PyPI
instead.

### 2. Configure a trusted publisher on each crate

For each of `carp-protocol`, `carp-catalog`, `carp-client` and `carp-dk`, open
its **Settings → Trusted Publishing** page on crates.io and add:

| Field | Value |
| --- | --- |
| Repository owner | `carp-dk` |
| Repository name | `carp-cli` |
| Workflow filename | `release.yml` |
| Environment | `crates-io` |

Four crates, four times. The environment must match what `release.yml`
declares, exactly as with PyPI.

### 3. Revoke the manual token

Once trusted publishing works, delete the token from
<https://crates.io/settings/tokens>. Leaving a long-lived token around is the
thing this setup exists to avoid.

### Adding an approval step later

Both registry uploads are currently automatic. To require a human first, go to
**Settings → Environments** in the GitHub repository and add yourself under
**Required reviewers** on `pypi`, `crates-io`, or both. The workflow needs no
change — the job waits for approval before uploading, and the GitHub release
waits behind it.

This is worth doing if a mistaken version bump ever reaches a registry, since
that version is then spent permanently.

### Rehearsing against TestPyPI

To try the whole path without spending a version on the real index, configure a
second pending publisher at <https://test.pypi.org> with the same values, and
run the publish step against it once:

```yaml
- uses: pypa/gh-action-pypi-publish@release/v1
  with:
    packages-dir: dist
    repository-url: https://test.pypi.org/legacy/
```

Remove it again afterwards. TestPyPI is periodically pruned, so nothing there
is a lasting record.

## One-time setup: macOS signing

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

## What is *not* published

`carp-python` goes to PyPI as `carp-cli`, never to crates.io. It is a cdylib
for one interpreter ABI; nothing written in Rust could depend on it. Its
`Cargo.toml` says `publish = false`, so `cargo publish --workspace` skips it.
