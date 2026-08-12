// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The protocol editor.
//!
//! `carp_study_app_configurations` builds a study protocol by running a
//! Flutter project per study: a `main()` that assembles Dart objects, a test
//! that serialises them, and a JSON file checked in beside them. It works, but
//! authoring a protocol means writing Dart, and checking one means reading
//! JSON.
//!
//! The studio replaces that with an editor. It holds one
//! [`carp_protocol::StudyProtocol`] and edits it in place, so what a
//! researcher sees is devices, tasks and schedules rather than a document.
//!
//! # Layout
//!
//! - [`section`] - the tabs, and what each one's keys do
//! - [`lists`] - which row of each tab is selected
//! - [`actions`] - add, remove and edit, routed by section
//! - [`pickers`] - the overlays that choose what to add
//! - [`input`] - key handling
//! - [`storage`] - reading and writing `protocol.json`
//! - [`history`] - undo
//!
//! # Invariants
//!
//! Every structural change goes through [`carp_protocol::builder`], never
//! through the protocol's fields directly, so a rename or a deletion cannot
//! leave a dangling reference. [`Studio::recheck`] then re-runs validation, so
//! the Checks tab is never stale.

pub mod actions;
pub mod history;
pub mod input;
pub mod lists;
pub mod pickers;
pub mod section;
pub mod storage;

use std::path::PathBuf;

use carp_catalog::Catalog;
use carp_protocol::validate::Diagnostic;
use carp_protocol::{StudyProtocol, VersionTag};

use crate::app::form::picker::Picker;
use crate::app::form::{Form, apply};

pub use history::History;
pub use lists::Lists;
pub use section::Section;

/// What the catalogue pane is doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogState {
    /// Nothing has been synced yet.
    Absent,
    /// A sync is in flight.
    Syncing,
    /// The catalogue is loaded.
    Ready,
    /// The last sync or load failed.
    Failed(String),
}

/// The protocol editor's whole state.
pub struct Studio {
    /// The protocol being edited.
    pub protocol: StudyProtocol,
    /// Where it was opened from, and where `s` saves it.
    pub path: Option<PathBuf>,
    /// Whether there are changes not yet written to `path`.
    pub dirty: bool,
    /// The version tag the next upload will use.
    pub version_tag: VersionTag,

    pub section: Section,
    pub lists: Lists,

    /// The upstream vocabulary, and how it is doing.
    pub catalog: Catalog,
    pub catalog_state: CatalogState,
    /// The documents the catalogue was derived from, kept so a study can be
    /// opened as a template without another download.
    pub snapshot: Option<carp_catalog::Snapshot>,
    /// A newer upstream commit, once a check has found one.
    pub update_available: Option<carp_catalog::Commit>,

    /// The form overlay, when one is open.
    pub form: Option<Form>,
    /// The picker overlay, when one is open. May sit on top of a form.
    pub picker: Option<Picker>,
    /// What the open picker is creating, when it is creating rather than
    /// filling in a field. See [`pickers::Creating`].
    pub creating: Option<pickers::Creating>,

    /// Findings from the last [`Studio::recheck`].
    pub diagnostics: Vec<Diagnostic>,
    /// Which task's survey the Survey tab is showing.
    pub survey_task: Option<String>,

    /// Snapshots for undo.
    pub history: History,
}

impl Studio {
    /// An editor holding a blank protocol owned by `owner_id`.
    pub fn new(owner_id: String) -> Self {
        let mut studio = Self {
            protocol: StudyProtocol::new("New protocol", owner_id),
            path: None,
            dirty: false,
            version_tag: VersionTag::initial(),
            section: Section::Overview,
            lists: Lists::default(),
            catalog: Catalog::default(),
            catalog_state: CatalogState::Absent,
            snapshot: None,
            update_available: None,
            form: None,
            picker: None,
            creating: None,
            diagnostics: Vec::new(),
            survey_task: None,
            history: History::default(),
        };
        // A protocol with no primary device cannot deploy, and every study
        // has one, so the blank protocol starts with a phone rather than with
        // an error.
        carp_protocol::builder::add_device(
            &mut studio.protocol,
            carp_protocol::DeviceKind::Smartphone,
        );
        // Without this the device list has no cursor, and `e` on the Devices
        // tab would find nothing to edit.
        studio.sync_selection();
        studio.recheck();
        studio
    }

    /// An editor holding `protocol`, opened from `path`.
    pub fn opened(protocol: StudyProtocol, path: Option<PathBuf>) -> Self {
        let mut studio = Self::new(protocol.owner_id.clone());
        studio.protocol = protocol;
        studio.path = path;
        studio.dirty = false;
        studio.history = History::default();
        studio.sync_selection();
        studio.recheck();
        studio
    }

    /// Record the protocol's current state, so the next change can be undone.
    ///
    /// Called before a change rather than after, which is what makes undo
    /// restore the state the user last saw.
    pub fn checkpoint(&mut self) {
        self.history.push(self.protocol.clone());
    }

    /// Mark the protocol as changed and re-run the checks.
    pub fn changed(&mut self) {
        self.dirty = true;
        self.sync_selection();
        self.recheck();
    }

    /// Undo the last change, if there is one.
    ///
    /// Returns false when there is nothing to undo, so the caller can say so
    /// rather than appearing to do nothing.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.history.pop() else {
            return false;
        };
        self.protocol = previous;
        self.dirty = true;
        self.sync_selection();
        self.recheck();
        true
    }

    /// Re-run validation over the protocol.
    pub fn recheck(&mut self) {
        self.diagnostics = carp_protocol::validate(&self.protocol);
        crate::app::state::clamp_selection(&mut self.lists.checks, self.diagnostics.len());
    }

    /// Keep every list's cursor inside its data.
    ///
    /// The survey list is synced against the *resolved* task rather than the
    /// stored one, so reaching the Survey tab with Tab - which never calls
    /// [`actions::open_survey`] - still lands the cursor on a step.
    pub fn sync_selection(&mut self) {
        let survey_task = self.survey_task_name();
        self.lists.sync(&self.protocol, survey_task.as_deref());
    }

    /// Apply the open form to the protocol and close it.
    ///
    /// A refusal leaves the form open with the reason to show, since the
    /// value has to be corrected somewhere and the form is where it is.
    pub fn submit_form(&mut self) -> Option<String> {
        let form = self.form.take()?;
        if !form.dirty {
            // Nothing changed, so nothing is recorded for undo.
            return None;
        }

        self.checkpoint();
        let outcome = apply::apply(&mut self.protocol, &form);
        match &outcome {
            apply::Applied::Changed => {
                self.changed();
                None
            }
            apply::Applied::Refused(reason) => {
                // Put the form back so the value can be fixed.
                self.history.pop();
                let reason = reason.clone();
                self.form = Some(form);
                Some(reason)
            }
            apply::Applied::Vanished => {
                self.history.pop();
                outcome.message()
            }
        }
    }

    /// The task whose survey the Survey tab shows, defaulting to the first
    /// task that has one.
    pub fn survey_task_name(&self) -> Option<String> {
        if let Some(name) = &self.survey_task
            && self
                .protocol
                .task(name)
                .is_some_and(|task| task.survey().is_some())
        {
            return Some(name.clone());
        }
        self.protocol
            .tasks
            .iter()
            .find(|task| task.survey().is_some())
            .map(|task| task.name().to_owned())
    }

    /// A short description of where the protocol lives, for the header.
    pub fn location(&self) -> String {
        match &self.path {
            Some(path) => {
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string());
                if self.dirty {
                    format!("{name} *")
                } else {
                    name
                }
            }
            None => {
                if self.dirty {
                    "unsaved *".to_owned()
                } else {
                    "unsaved".to_owned()
                }
            }
        }
    }

    /// Errors, warnings and notices from the last check.
    pub fn check_counts(&self) -> (usize, usize, usize) {
        carp_protocol::validate::counts(&self.diagnostics)
    }
}

#[cfg(test)]
mod tests;
