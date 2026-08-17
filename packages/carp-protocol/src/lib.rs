// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The CARP study protocol, as a Rust domain model.
//!
//! A *study protocol* describes what a study measures: which devices take
//! part, which tasks run on them, what triggers those tasks, and what is asked
//! of the participants. The CARP Study App consumes it as a single
//! `protocol.json` document, which until now was produced by running a Flutter
//! project per study (`carp_study_app_configurations`). This crate models the
//! same document directly, so it can be built, checked and written without a
//! Dart toolchain.
//!
//! # Layout
//!
//! - [`protocol`] - [`StudyProtocol`], the root document
//! - [`application_data`] - the CARP Mobile Sensing extensions (`applicationData`)
//! - [`device`] - phones, wearables and the services modelled as devices
//! - [`task`] - what is measured, and the app tasks shown to a participant
//! - [`trigger`] - when a task starts
//! - [`survey`] - the Research Package survey tree carried by `RPAppTask`
//! - [`participant`] - participant roles and the data expected of them
//! - [`control`] - the trigger/task/device wiring
//! - [`mod@validate`] - referential and semantic checks
//! - [`builder`] - the mutation API the interactive editor drives
//! - [`version`] - protocol revisions and version tags
//!
//! # Wire format
//!
//! CARP serialises with kotlinx.serialization, so every polymorphic value
//! carries a `__type` discriminator holding the fully qualified Kotlin class
//! name. Each enum here maps its variants onto those exact strings, and each
//! also has an [`node::UnknownNode`] fallback so a document written by a newer
//! CARP release still round-trips instead of failing to parse. See [`node`].
//!
//! Durations are microseconds on the wire; [`duration::Micros`] wraps that so
//! the unit cannot be mistaken. See [`duration`].
//!
//! # Example
//!
//! ```
//! use carp_protocol::StudyProtocol;
//!
//! let json = std::fs::read_to_string(
//!     concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus/neuropathy.json"),
//! )
//! .unwrap();
//!
//! let protocol: StudyProtocol = serde_json::from_str(&json).unwrap();
//! assert_eq!(protocol.name, "CARP Neuropathy Tracker Protocol");
//! assert_eq!(protocol.primary_devices.len(), 1);
//!
//! // Re-serialising reproduces the original document.
//! let round_tripped = serde_json::to_value(&protocol).unwrap();
//! assert_eq!(round_tripped, serde_json::from_str::<serde_json::Value>(&json).unwrap());
//! ```

pub mod application_data;
pub mod builder;
pub mod control;
pub mod device;
pub mod duration;
pub mod node;
pub mod participant;
pub mod protocol;
pub mod survey;
pub mod task;
pub mod trigger;
pub mod validate;
pub mod version;

pub use application_data::{ApplicationData, DataEndPoint, StudyDescription, StudyResponsible};
pub use control::{Control, DeviceConnection, TaskControl};
pub use device::{Device, DeviceKind, SamplingConfiguration};
pub use duration::Micros;
pub use node::UnknownNode;
pub use participant::{AssignedTo, ExpectedParticipantData, ParticipantAttribute, ParticipantRole};
pub use protocol::StudyProtocol;
pub use survey::{RpAnswerFormat, RpChoice, RpStep, RpTask};
pub use task::{Measure, Task, TaskKind};
pub use trigger::{Trigger, TriggerKind};
pub use validate::{Diagnostic, Severity, validate};
pub use version::{ProtocolVersion, VersionTag};

/// Errors raised when a protocol document cannot be read or written.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The document is not valid JSON, or does not match the protocol schema.
    #[error("malformed protocol document: {0}")]
    Malformed(#[from] serde_json::Error),

    /// A value was rejected by the domain, e.g. a duration that does not parse.
    #[error("{0}")]
    Invalid(String),
}

/// Result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Parse a `protocol.json` document.
///
/// Unknown `__type` values are preserved rather than rejected, so a protocol
/// authored against a newer CARP release still loads; [`mod@validate`] reports
/// them as warnings.
pub fn parse(json: &str) -> Result<StudyProtocol> {
    Ok(serde_json::from_str(json)?)
}

/// Render a protocol as the pretty-printed JSON the study app expects.
///
/// Two-space indentation matches what the Dart generator produced, so a
/// protocol migrated from `carp_study_app_configurations` gives a readable
/// diff rather than a whole-file rewrite.
pub fn to_json(protocol: &StudyProtocol) -> Result<String> {
    let mut buffer = Vec::with_capacity(16 * 1024);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut serializer = serde_json::Serializer::with_formatter(&mut buffer, formatter);
    serde::Serialize::serialize(protocol, &mut serializer)?;
    // The serialiser only ever writes UTF-8.
    Ok(String::from_utf8(buffer).expect("serde_json emits UTF-8"))
}
