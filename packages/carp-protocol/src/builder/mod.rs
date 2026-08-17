// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Editing a protocol without breaking it.
//!
//! A protocol's parts refer to each other by name, and those names appear in
//! several places at once: a device role name is repeated by every trigger
//! that fires on it, every task control that targets it and every connection
//! that reaches it. Renaming the device in one place and not the others gives
//! a protocol that still parses and no longer works.
//!
//! Every mutation the editor performs therefore goes through this module, and
//! each one leaves the graph consistent:
//!
//! - renaming updates every reference in the same call
//! - removing takes the now-dangling references with it, and says what it took
//! - adding wires up what the new part needs to be reachable
//!
//! [`mod@crate::validate`] still checks the result. This module is what stops the
//! editor creating a problem; the validator is what catches one that arrived
//! by other means, such as a hand-edited file.

pub mod devices;
pub mod people;
pub mod tasks;
pub mod triggers;

pub use devices::{add_device, remove_device, rename_device};
pub use people::{add_participant_role, remove_participant_role, rename_participant_role};
pub use tasks::{add_task, remove_task, rename_task};
pub use triggers::{add_task_control, add_trigger, remove_trigger};

/// What a removal took with it, so the editor can say so.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Removal {
    /// Task controls that referenced the removed part.
    pub task_controls: usize,
    /// Triggers removed because they only existed for the removed part.
    pub triggers: usize,
    /// Device connections removed with a device.
    pub connections: usize,
    /// Tasks removed with the device that ran them.
    pub tasks: usize,
}

impl Removal {
    /// Whether anything beyond the named part was removed.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// A phrase for the status line, e.g. `2 task controls, 1 trigger`.
    pub fn summary(&self) -> String {
        let parts = [
            (self.tasks, "task"),
            (self.triggers, "trigger"),
            (self.task_controls, "task control"),
            (self.connections, "connection"),
        ];
        parts
            .into_iter()
            .filter(|(count, _)| *count > 0)
            .map(|(count, noun)| {
                let plural = if count == 1 { "" } else { "s" };
                format!("{count} {noun}{plural}")
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// `base`, or `base 2`, `base 3` … until it is not in `taken`.
///
/// Names are what the protocol's references are built from, so a new part
/// silently taking an existing name would re-point every reference to it.
pub fn unique_name(base: &str, taken: &[String]) -> String {
    let base = if base.trim().is_empty() {
        "Unnamed"
    } else {
        base
    };
    if !taken.iter().any(|name| name == base) {
        return base.to_owned();
    }
    (2u32..)
        .map(|suffix| format!("{base} {suffix}"))
        .find(|candidate| !taken.iter().any(|name| name == candidate))
        .unwrap_or_else(|| base.to_owned())
}

#[cfg(test)]
mod tests;
