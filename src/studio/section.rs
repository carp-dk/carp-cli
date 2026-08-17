// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The tabs of the protocol editor.
//!
//! The order is the order a protocol is built in: what it is, what it
//! collects from, what it does, when, and to whom - then the two panes that
//! are about the work rather than the protocol, the catalogue and the checks.

/// One tab of the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Name, owner, and the study-app settings.
    Overview,
    /// Primary and connected devices.
    Devices,
    /// Tasks and their measures.
    Tasks,
    /// Triggers, and which task each one starts.
    Triggers,
    /// The survey of the selected `RPAppTask`.
    Survey,
    /// Participant roles and the data expected of them.
    Participants,
    /// The synced upstream vocabulary and its version.
    Catalog,
    /// Validation findings.
    Checks,
}

impl Section {
    pub const ALL: [Self; 8] = [
        Self::Overview,
        Self::Devices,
        Self::Tasks,
        Self::Triggers,
        Self::Survey,
        Self::Participants,
        Self::Catalog,
        Self::Checks,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Devices => "Devices",
            Self::Tasks => "Tasks",
            Self::Triggers => "Triggers",
            Self::Survey => "Survey",
            Self::Participants => "Participants",
            Self::Catalog => "Catalog",
            Self::Checks => "Checks",
        }
    }

    /// The keys this section responds to, shown in the status bar. Kept next
    /// to the section rather than in the renderer so a new binding and its
    /// hint cannot drift apart.
    pub fn hints(self) -> &'static str {
        match self {
            Self::Overview => {
                "e edit · A app settings · D data endpoint · v tag · s save · u upload"
            }
            Self::Devices => "a add · e edit · x remove · c connect · s save",
            Self::Tasks => "a add · e edit · x remove · m measures · Enter survey · s save",
            Self::Triggers => "a add · e edit · x remove · t attach task · s save",
            Self::Survey => "a add step · e edit · x remove · J/K reorder · Esc back",
            Self::Participants => "a add role · A expect data · e edit · x remove",
            Self::Catalog => "S sync · Enter use as template · r reload",
            Self::Checks => "r recheck · Enter go to the finding",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|section| *section == self)
            .unwrap_or(0)
    }

    pub fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}

#[cfg(test)]
mod tests;
