# carp-protocol

The [CARP][carp] study protocol as a Rust domain model

A *study protocol* describes what a study measures — which devices take part,
which tasks run on them, what triggers those tasks, and what is asked of the
participants. The CARP Study App consumes it as a single `protocol.json`.

```rust
use carp_protocol::StudyProtocol;

let protocol: StudyProtocol = serde_json::from_str(&json)?;
println!("{} — {}", protocol.name, protocol.summary());

for diagnostic in carp_protocol::validate(&protocol) {
    println!("{} {}", diagnostic.severity.label(), diagnostic.message);
}
```

- **Round-trips.** CARP serialises with kotlinx.serialization, so every
  polymorphic value carries a `__type` holding a fully qualified Kotlin class
  name. Each enum maps its variants onto those exact strings, and each has an
  unknown-node fallback, so a document written by a newer CARP still
  round-trips instead of failing to parse.
- **Validates.** Referential and semantic checks the schema cannot express: a
  device name that does not resolve, an identifier used twice, a task nothing
  starts, a survey branch that jumps to a step that no longer exists.
- **Mutates safely.** `builder` is the mutation API — renaming a device moves
  every reference with it, and removing one takes the triggers and controls
  that could only have referred to it.

Tested against the `protocol.json` of every study in
`carp_study_app_configurations`: each is parsed, re-serialised and compared
field for field, and none may fall back to the preserve-verbatim path.

Part of [carp-cli](https://github.com/carp-dk/carp-cli). No dependency on a
terminal or a network.

[carp]: https://carp.dk
