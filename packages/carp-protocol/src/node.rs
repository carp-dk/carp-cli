// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Forward compatibility for the polymorphic parts of the schema.
//!
//! CARP evolves: `dk.cachet.carp.common.application.devices.PolarDevice` did
//! not exist a year ago, and whatever ships next year does not exist now. A
//! parser that rejected an unrecognised `__type` would make this crate stop
//! working the moment upstream adds a sampling package - exactly when a
//! researcher most wants to open the protocol that uses it.
//!
//! So every polymorphic enum in this crate is shaped like:
//!
//! ```text
//! #[serde(untagged)]
//! enum Device {
//!     Known(KnownDevice),   // #[serde(tag = "__type")] over the modelled types
//!     Unknown(UnknownNode), // anything else, kept verbatim
//! }
//! ```
//!
//! `serde`'s untagged representation tries `Known` first; an unmodelled
//! `__type` fails that attempt and lands in [`UnknownNode`], which keeps the
//! discriminator and every other field as raw JSON. Re-serialising writes it
//! back unchanged, so a document survives a load/save cycle through a version
//! of this tool that predates it.
//!
//! The cost is that a *modelled* type with a malformed field would also fall
//! through to `Unknown` rather than raising an error. That is why the corpus
//! test in `tests/roundtrip.rs` asserts the reference protocols parse with no
//! unknown nodes at all: a silent fallback shows up as a failing test rather
//! than as data quietly turning into an opaque blob.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A polymorphic value whose `__type` this version of the crate does not model.
///
/// Everything is preserved: the discriminator in [`UnknownNode::type_name`],
/// the remaining fields in [`UnknownNode::fields`]. Re-serialising emits the
/// original object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnknownNode {
    /// The `__type` discriminator, e.g. a device class added upstream.
    #[serde(rename = "__type")]
    pub type_name: String,
    /// Every other field of the object, untouched.
    #[serde(flatten)]
    pub fields: Map<String, Value>,
}

impl UnknownNode {
    /// Build a node for `type_name` with no further fields.
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
            fields: Map::new(),
        }
    }

    /// The part of the discriminator a human reads: `PolarDevice` rather than
    /// `dk.cachet.carp.common.application.devices.PolarDevice`.
    pub fn short_type(&self) -> &str {
        short_type(&self.type_name)
    }

    /// A field of the preserved object, if present.
    pub fn field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }

    /// `roleName` if the node has one. Devices and several other types carry
    /// it, and the editor lists nodes by role name where it can.
    pub fn role_name(&self) -> Option<&str> {
        self.field("roleName").and_then(Value::as_str)
    }

    /// `name` if the node has one, which is how tasks identify themselves.
    pub fn name(&self) -> Option<&str> {
        self.field("name").and_then(Value::as_str)
    }
}

/// Last dot-separated segment of a fully qualified Kotlin class name.
///
/// Nested classes keep their parent: the last segment of
/// `…tasks.Measure.DataStream` is `DataStream`, which alone is ambiguous, so
/// a segment starting with an upper-case letter that follows another such
/// segment is kept too.
pub fn short_type(type_name: &str) -> &str {
    let is_class = |segment: &str| segment.chars().next().is_some_and(char::is_uppercase);

    // Walk backwards over the dots, keeping every trailing segment that looks
    // like a class name, i.e. has an upper-case initial.
    let mut kept_from = None;
    for (dot, _) in type_name.rmatch_indices('.') {
        let segment = type_name[dot + 1..].split('.').next().unwrap_or_default();
        if !is_class(segment) {
            break;
        }
        kept_from = Some(dot + 1);
    }

    match kept_from {
        Some(start) => &type_name[start..],
        // Nothing class-like at the end (`dk.cachet.carp.movesense.state`):
        // the last segment is the best answer available.
        None => type_name.rsplit('.').next().unwrap_or(type_name),
    }
}

#[cfg(test)]
mod tests;
