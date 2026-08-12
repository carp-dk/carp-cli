// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Checking a protocol before it reaches a participant's phone.
//!
//! A protocol is a graph joined by names, and JSON cannot express "this name
//! must resolve". A task control naming a task that was renamed is valid JSON
//! and a study that silently never runs that task - the kind of fault found
//! weeks later, in missing data.
//!
//! So everything the schema cannot say is checked here. Findings come back as
//! [`Diagnostic`]s at three levels:
//!
//! - [`Severity::Error`] - the protocol is broken. A name does not resolve, an
//!   identifier is used twice, a required field is empty. Uploading it would
//!   produce a study that misbehaves.
//! - [`Severity::Warning`] - legal but almost certainly a mistake. A task
//!   nothing starts, a device nothing measures.
//! - [`Severity::Info`] - worth knowing. A type from a newer CARP than this
//!   tool models, which will be preserved but cannot be edited.
//!
//! Rules live in [`rules`], one function each, so the list of what is checked
//! reads as a list.

pub mod rules;

use std::fmt;

use crate::protocol::StudyProtocol;

/// How much a finding matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// The protocol is broken and should not be uploaded.
    Error,
    /// Legal, but almost certainly not what was meant.
    Warning,
    /// Worth knowing about.
    Info,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

/// One finding about a protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    /// Where the problem is, in terms the editor can navigate to, e.g.
    /// `task "Sleep Diary"` or `trigger 3`.
    pub location: String,
    /// What is wrong, in one sentence.
    pub message: String,
    /// What to do about it, when there is an obvious answer.
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn error(location: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            location: location.into(),
            message: message.into(),
            hint: None,
        }
    }

    pub fn warning(location: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            location: location.into(),
            message: message.into(),
            hint: None,
        }
    }

    pub fn info(location: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            location: location.into(),
            message: message.into(),
            hint: None,
        }
    }

    /// Attach advice on how to fix the finding.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}: {}", self.severity, self.location, self.message)?;
        if let Some(hint) = &self.hint {
            write!(formatter, " ({hint})")?;
        }
        Ok(())
    }
}

/// Run every rule over `protocol`, most severe findings first.
///
/// The order within a severity is the order the rules are listed below, which
/// walks the protocol roughly as the editor's tabs do.
pub fn validate(protocol: &StudyProtocol) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    rules::identity(protocol, &mut diagnostics);
    rules::devices(protocol, &mut diagnostics);
    rules::connections(protocol, &mut diagnostics);
    rules::tasks(protocol, &mut diagnostics);
    rules::triggers(protocol, &mut diagnostics);
    rules::task_controls(protocol, &mut diagnostics);
    rules::participants(protocol, &mut diagnostics);
    rules::surveys(protocol, &mut diagnostics);
    rules::unmodelled_types(protocol, &mut diagnostics);

    // A stable sort keeps each rule's own findings in the order it produced
    // them, so a protocol with several problems reads consistently.
    diagnostics.sort_by_key(|diagnostic| diagnostic.severity);
    diagnostics
}

/// Whether `protocol` has anything that would break a deployment.
pub fn is_deployable(protocol: &StudyProtocol) -> bool {
    !validate(protocol)
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
}

/// How many findings there are at each severity, for a status line.
pub fn counts(diagnostics: &[Diagnostic]) -> (usize, usize, usize) {
    let count = |severity| {
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == severity)
            .count()
    };
    (
        count(Severity::Error),
        count(Severity::Warning),
        count(Severity::Info),
    )
}

#[cfg(test)]
mod tests;
