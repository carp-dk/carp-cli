// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Reading and writing the fields every survey step shares.

use super::KnownStep;

impl KnownStep {
    /// The identifier, whichever variant this is.
    pub fn identifier(&self) -> &str {
        match self {
            Self::Instruction { identifier, .. }
            | Self::Completion { identifier, .. }
            | Self::Question { identifier, .. }
            | Self::Form { identifier, .. }
            | Self::Tapping { identifier, .. }
            | Self::Flanker { identifier, .. }
            | Self::ReactionTime { identifier, .. } => identifier,
        }
    }

    pub(super) fn set_identifier(&mut self, value: String) {
        match self {
            Self::Instruction { identifier, .. }
            | Self::Completion { identifier, .. }
            | Self::Question { identifier, .. }
            | Self::Form { identifier, .. }
            | Self::Tapping { identifier, .. }
            | Self::Flanker { identifier, .. }
            | Self::ReactionTime { identifier, .. } => *identifier = value,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::Instruction { title, .. }
            | Self::Completion { title, .. }
            | Self::Question { title, .. }
            | Self::Form { title, .. }
            | Self::Tapping { title, .. }
            | Self::Flanker { title, .. }
            | Self::ReactionTime { title, .. } => title,
        }
    }

    pub(super) fn set_title(&mut self, value: String) {
        match self {
            Self::Instruction { title, .. }
            | Self::Completion { title, .. }
            | Self::Question { title, .. }
            | Self::Form { title, .. }
            | Self::Tapping { title, .. }
            | Self::Flanker { title, .. }
            | Self::ReactionTime { title, .. } => *title = value,
        }
    }

    pub fn type_label(&self) -> &'static str {
        match self {
            Self::Instruction { .. } => "RPInstructionStep",
            Self::Completion { .. } => "RPCompletionStep",
            Self::Question { .. } => "RPQuestionStep",
            Self::Form { .. } => "RPFormStep",
            Self::Tapping { .. } => "RPTappingActivity",
            Self::Flanker { .. } => "RPFlankerActivity",
            Self::ReactionTime { .. } => "RPReactionTimeActivity",
        }
    }
}
