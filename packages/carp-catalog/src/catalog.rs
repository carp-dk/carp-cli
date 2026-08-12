// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The vocabulary the editor offers, and where each entry came from.
//!
//! Everything here is *observed*, not declared. A measure type is in the
//! catalogue because some study upstream measures it; a health metric is
//! offered because some study reads it. That has a useful consequence: an
//! entry can say how many studies use it, so the editor can put the twenty
//! measure types people actually use above the long tail, without anyone
//! maintaining a list of favourites.
//!
//! See [`crate::derive`] for how a catalogue is produced from a
//! [`crate::Snapshot`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::source::Commit;

/// Which upstream commit a catalogue was derived from.
///
/// Kept beside the vocabulary rather than alongside it, so a catalogue handed
/// to the editor always knows how to answer "which version of CARP's
/// conventions is this?".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogVersion {
    /// `owner/repo` the vocabulary was derived from.
    pub repository: String,
    /// The commit itself.
    pub commit: Commit,
    /// When the snapshot behind it was downloaded, as an ISO-8601 instant.
    pub fetched_at: String,
    /// How many study protocols contributed.
    pub studies: usize,
}

impl CatalogVersion {
    /// A one-line description for the interface, e.g.
    /// `158cdcb · 2026-07-07 · 10 studies`.
    pub fn label(&self) -> String {
        format!(
            "{} · {} · {} stud{}",
            self.commit.short_sha(),
            self.commit.date.split('T').next().unwrap_or(&self.commit.date),
            self.studies,
            if self.studies == 1 { "y" } else { "ies" }
        )
    }

    /// How long ago the snapshot was downloaded, in whole days.
    ///
    /// `None` when `fetched_at` cannot be read, which only happens to a
    /// hand-edited cache.
    pub fn age_in_days(&self) -> Option<i64> {
        let fetched = chrono::DateTime::parse_from_rfc3339(&self.fetched_at).ok()?;
        Some((chrono::Utc::now() - fetched.to_utc()).num_days())
    }
}

/// One value the editor can offer, and how widely it is used upstream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// The value itself, exactly as it appears in a protocol.
    pub value: String,
    /// How many times it occurs across every study.
    pub occurrences: usize,
    /// Which studies use it, sorted. Doubles as the answer to "who else does
    /// this?", which is the question someone copying a convention is asking.
    pub studies: Vec<String>,
}

impl CatalogEntry {
    /// The readable tail of a namespaced value: `stepcount` for
    /// `dk.cachet.carp.stepcount`.
    pub fn short_value(&self) -> &str {
        carp_protocol::node::short_type(&self.value)
    }

    /// `used by 3 studies`, or `used by demo` when there is only one.
    pub fn usage(&self) -> String {
        match self.studies.len() {
            0 => "unused".to_owned(),
            1 => format!("used by {}", self.studies[0]),
            count => format!("used by {count} studies"),
        }
    }
}

/// A named list of values the editor offers.
pub type Vocabulary = Vec<CatalogEntry>;

/// Everything derived from one snapshot.
///
/// Each field is a list the editor turns into a picker, ordered by how widely
/// used its entries are so the common answer is the first one.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Catalog {
    /// Which commit this was derived from. `None` only for a catalogue built
    /// in a test.
    pub version: Option<CatalogVersion>,

    /// Data stream types a measure can collect, e.g.
    /// `dk.cachet.carp.stepcount`.
    pub measure_types: Vocabulary,
    /// Device classes seen upstream. Includes any this build does not model,
    /// which is how the editor knows to say so.
    pub device_types: Vocabulary,
    /// Health metrics a `HealthSamplingConfiguration` can read, e.g. `STEPS`.
    pub health_data_types: Vocabulary,
    /// Input types expected of a participant, e.g.
    /// `dk.cachet.carp.input.sex`.
    pub input_data_types: Vocabulary,
    /// The `type` of an app task, which the study app picks a card style
    /// from: `survey`, `audio`, `health`.
    pub app_task_types: Vocabulary,
    /// Participant role names in use, e.g. `Participant`, `Father`.
    pub participant_roles: Vocabulary,
    /// Device role names in use, so a new device can follow the naming
    /// everyone else uses.
    pub device_role_names: Vocabulary,
    /// `questionType` values seen on answer formats.
    pub question_types: Vocabulary,
    /// Conditions a `UserTaskTrigger` waits for, e.g. `done`.
    pub user_task_conditions: Vocabulary,
    /// Upload methods a CARP data endpoint accepts, e.g. `stream`.
    pub upload_methods: Vocabulary,
    /// Location accuracies a `LocationService` accepts, e.g. `balanced`.
    pub location_accuracies: Vocabulary,

    /// Studies that can be copied as a starting point, with a summary of each.
    pub templates: Vec<Template>,

    /// Documents in the snapshot that did not parse, if any. Shown so a
    /// catalogue that is quietly incomplete cannot look complete.
    #[serde(default)]
    pub skipped: Vec<String>,
}

/// A study offered as a starting point for a new protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    /// Directory name upstream, e.g. `neuropathy`.
    pub study: String,
    /// The protocol's own name, e.g. `CARP Neuropathy Tracker Protocol`.
    pub name: String,
    /// `2 devices, 3 tasks, 3 triggers`.
    pub summary: String,
    /// Whether the study app extensions are present, which almost every
    /// study wants and the browser-only ones do not have.
    pub has_application_data: bool,
}

impl Catalog {
    /// Whether anything was derived. A catalogue with no measure types came
    /// from an empty or unreadable snapshot.
    pub fn is_empty(&self) -> bool {
        self.measure_types.is_empty() && self.templates.is_empty()
    }

    /// The entries of `vocabulary` containing `needle`, case-insensitively.
    ///
    /// Used by every picker's filter box. The whole value is matched, so
    /// typing `stepcount` finds `dk.cachet.carp.stepcount` without anyone
    /// having to type the namespace, and typing `carp.` still narrows to it.
    pub fn search<'a>(vocabulary: &'a [CatalogEntry], needle: &str) -> Vec<&'a CatalogEntry> {
        let needle = needle.trim().to_lowercase();
        if needle.is_empty() {
            return vocabulary.iter().collect();
        }
        vocabulary
            .iter()
            .filter(|entry| entry.value.to_lowercase().contains(&needle))
            .collect()
    }

    /// Whether `value` occurs anywhere in `vocabulary`.
    ///
    /// The editor accepts values outside the catalogue - a study may be the
    /// first to use a new measure - but says when one is unfamiliar.
    pub fn contains(vocabulary: &[CatalogEntry], value: &str) -> bool {
        vocabulary.iter().any(|entry| entry.value == value)
    }
}

/// Counts occurrences and the studies they occurred in, then sorts into the
/// order the editor presents.
///
/// Ordering is by occurrences descending, then value ascending: the common
/// answer first, and ties broken predictably so the list does not reshuffle
/// between syncs.
#[derive(Debug, Default)]
pub struct VocabularyBuilder {
    counts: BTreeMap<String, (usize, std::collections::BTreeSet<String>)>,
}

impl VocabularyBuilder {
    /// Record one occurrence of `value` in `study`.
    pub fn record(&mut self, value: impl Into<String>, study: &str) {
        let value = value.into();
        if value.trim().is_empty() {
            return;
        }
        let entry = self.counts.entry(value).or_default();
        entry.0 += 1;
        entry.1.insert(study.to_owned());
    }

    /// Finish, producing the sorted vocabulary.
    pub fn build(self) -> Vocabulary {
        let mut entries: Vocabulary = self
            .counts
            .into_iter()
            .map(|(value, (occurrences, studies))| CatalogEntry {
                value,
                occurrences,
                studies: studies.into_iter().collect(),
            })
            .collect();
        entries.sort_by(|left, right| {
            right
                .occurrences
                .cmp(&left.occurrences)
                .then_with(|| left.value.cmp(&right.value))
        });
        entries
    }
}

#[cfg(test)]
mod tests;
