// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Which screen is on top, and which tab of it.

/// Which screen is on top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// List of studies.
    Studies,
    /// One study, with its tabs.
    Study,
    /// One participant of a study.
    Participant,
    /// Transfer queue.
    Downloads,
    /// The protocol editor.
    Studio,
}

/// Tabs of the study screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudyTab {
    Overview,
    Participants,
    Deployments,
    Staff,
    Files,
    Exports,
}

impl StudyTab {
    pub const ALL: [Self; 6] = [
        Self::Overview,
        Self::Participants,
        Self::Deployments,
        Self::Staff,
        Self::Files,
        Self::Exports,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Participants => "Participants",
            Self::Deployments => "Deployments",
            Self::Staff => "Staff",
            Self::Files => "Files",
            Self::Exports => "Exports",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|tab| *tab == self).unwrap_or(0)
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
