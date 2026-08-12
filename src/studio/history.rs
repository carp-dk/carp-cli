// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Undo.
//!
//! Removing a device takes its triggers and task controls with it - that is
//! the right behaviour, and it is also the behaviour that most warrants being
//! reversible. Rather than inverting each operation, the editor keeps whole
//! snapshots: a [`carp_protocol::StudyProtocol`] is a few tens of kilobytes
//! and clones in microseconds, so the simple approach is also the fast one,
//! and it cannot get an inverse wrong.
//!
//! The depth is bounded because a long editing session would otherwise hold
//! every state it ever passed through.

use carp_protocol::StudyProtocol;

/// How many steps back the editor can go.
///
/// Deep enough to cover a wrong turn, shallow enough that the memory is
/// bounded at a few megabytes for the largest protocols.
pub const DEPTH: usize = 50;

/// Snapshots of the protocol, most recent last.
#[derive(Debug, Default)]
pub struct History {
    states: Vec<StudyProtocol>,
}

impl History {
    /// Record a state to return to.
    ///
    /// Once [`DEPTH`] states are held, the oldest is dropped.
    pub fn push(&mut self, protocol: StudyProtocol) {
        if self.states.len() >= DEPTH {
            self.states.remove(0);
        }
        self.states.push(protocol);
    }

    /// Take the most recent state back off.
    pub fn pop(&mut self) -> Option<StudyProtocol> {
        self.states.pop()
    }

    /// How many steps can be undone.
    pub fn depth(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Forget everything, as opening another protocol does.
    pub fn clear(&mut self) {
        self.states.clear();
    }
}

#[cfg(test)]
mod tests;
