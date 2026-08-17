# carp-catalog

The vocabulary a CARP study protocol can be written in, derived from the
studies that already exist.

A protocol names measure types, device classes, health metrics and input types
as fully qualified strings. Which of those a given CARP release actually
supports is not in any schema — it is in the studies running on it. This crate
downloads [`carp_study_app_configurations`][configs], records the commit it
came from, and derives the vocabulary from what those studies use.

```rust
let report = carp_catalog::sync::sync(&data_dir).await?;
println!("{}", report.summary());

let catalog = carp_catalog::sync::load(&data_dir).await?;
println!("{} measure types", catalog.measure_types.len());
```

- **Versioned.** A snapshot names the commit it was taken at, so a protocol
  editor can say when upstream has moved past it.
- **Explicit.** Syncing is something a caller asks for, never something that
  happens underneath it: a value that was offered a moment ago should not
  vanish mid-edit.
- **Offline after the first sync.** The snapshot is on disk; loading it needs
  no network.

Part of [carp-cli](https://github.com/carp-dk/carp-cli), where it supplies the
protocol editor's pickers and its Catalog tab.

> The upstream repository is private. Set `GITHUB_TOKEN` to a token with access
> to it.

[configs]: https://github.com/carp-dk/carp_study_app_configurations
