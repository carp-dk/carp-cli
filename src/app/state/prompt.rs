// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The prompt line at the bottom of the screen, and the study list's sort order.

/// A prompt open at the bottom of the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptKind {
    /// Filter the study list locally. Applied on every keystroke.
    StudyFilter,
    /// Search participants server-side. Applied on enter, since it costs a
    /// request.
    ParticipantSearch,
    /// Destructive actions ask first.
    ConfirmDeleteExport { export_id: String, name: String },
    /// Path of a protocol to open in the editor.
    OpenProtocol,
    /// Leaving the editor with unsaved work asks first.
    ConfirmDiscardProtocol,
    /// The version tag the next upload is filed under.
    ProtocolVersionTag,
}

#[derive(Debug, Clone)]
pub struct Prompt {
    pub kind: PromptKind,
    pub value: String,
    /// What to restore if the prompt is cancelled.
    pub original: String,
}

impl Prompt {
    pub fn new(kind: PromptKind, value: String) -> Self {
        Self {
            original: value.clone(),
            kind,
            value,
        }
    }

    pub fn confirm(kind: PromptKind) -> Self {
        Self {
            kind,
            value: String::new(),
            original: String::new(),
        }
    }

    /// True when the prompt is a yes/no question rather than a text field.
    pub fn is_confirmation(&self) -> bool {
        matches!(
            self.kind,
            PromptKind::ConfirmDeleteExport { .. } | PromptKind::ConfirmDiscardProtocol
        )
    }

    pub fn label(&self) -> String {
        match &self.kind {
            PromptKind::StudyFilter => "filter studies".to_owned(),
            PromptKind::ParticipantSearch => "search participants".to_owned(),
            PromptKind::ConfirmDeleteExport { name, .. } => format!("delete export {name}?"),
            PromptKind::OpenProtocol => {
                "open protocol (path to a file or study directory)".to_owned()
            }
            PromptKind::ConfirmDiscardProtocol => "leave without saving?".to_owned(),
            PromptKind::ProtocolVersionTag => "version tag for the next upload".to_owned(),
        }
    }
}

/// Sort order of the study list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StudySort {
    #[default]
    Name,
    /// Newest first: the study someone is working on is usually the newest.
    Created,
    Stage,
}

impl StudySort {
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Created => "newest",
            Self::Stage => "stage",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Created,
            Self::Created => Self::Stage,
            Self::Stage => Self::Name,
        }
    }
}
