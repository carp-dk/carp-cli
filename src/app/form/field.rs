// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! One editable field.
//!
//! The protocol editor never edits JSON. Every value a user changes is a
//! [`Field`], and every field knows three things: what it is called, what it
//! currently holds, and what it will accept. That is what lets one form
//! renderer and one set of key bindings serve devices, tasks, triggers and
//! surveys alike - and what lets a bad value be refused where it is typed,
//! rather than surfacing later as a validation error.
//!
//! Values that come from CARP's vocabulary - measure types, health metrics -
//! are [`FieldValue::Catalog`], which defers to the synced catalogue rather
//! than to a list written here. See [`carp_catalog`].

pub mod read;

use carp_protocol::duration::Micros;
use carp_protocol::trigger::TimeOfDay;

/// Which of the catalogue's lists a field draws its options from.
///
/// Named rather than holding the list, so a form can be built before the
/// catalogue has loaded and still know what it will offer.
///
/// Only the lists a *field* picks from are here. The catalogue derives more
/// than this - the device role names and participant roles studies use, for
/// instance - but those are conventions to read in the Catalog tab rather
/// than values to choose between, and names are typed rather than picked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vocabulary {
    MeasureTypes,
    HealthDataTypes,
    InputDataTypes,
    AppTaskTypes,
    UserTaskConditions,
    UploadMethods,
    LocationAccuracies,
}

impl Vocabulary {
    /// What the picker calls itself.
    pub fn title(self) -> &'static str {
        match self {
            Self::MeasureTypes => "measure types",
            Self::HealthDataTypes => "health metrics",
            Self::InputDataTypes => "participant input types",
            Self::AppTaskTypes => "task types",
            Self::UserTaskConditions => "task conditions",
            Self::UploadMethods => "upload methods",
            Self::LocationAccuracies => "location accuracies",
        }
    }

    /// The matching list of a catalogue.
    pub fn entries(self, catalog: &carp_catalog::Catalog) -> &[carp_catalog::CatalogEntry] {
        match self {
            Self::MeasureTypes => &catalog.measure_types,
            Self::HealthDataTypes => &catalog.health_data_types,
            Self::InputDataTypes => &catalog.input_data_types,
            Self::AppTaskTypes => &catalog.app_task_types,
            Self::UserTaskConditions => &catalog.user_task_conditions,
            Self::UploadMethods => &catalog.upload_methods,
            Self::LocationAccuracies => &catalog.location_accuracies,
        }
    }
}

/// One option of a [`FieldValue::Choice`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// What is stored when this option is picked.
    pub value: String,
    /// What is shown.
    pub label: String,
    /// One line explaining it, shown beside the picker.
    pub description: String,
}

impl Choice {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: String::new(),
        }
    }

    pub fn described(
        value: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: description.into(),
        }
    }
}

/// What a field holds, and therefore how it is edited.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// Free text, typed directly.
    Text(String),
    /// A whole number within bounds. Out-of-range input is refused rather
    /// than clamped, so a mistyped digit is visible instead of silently
    /// becoming the maximum.
    Integer { value: i64, min: i64, max: i64 },
    /// Yes or no, toggled with space.
    Toggle(bool),
    /// A duration, typed as `30d` or `1h30m`. See [`Micros::parse`].
    Duration(Micros),
    /// A wall-clock time, typed as `HH:MM`.
    Time(TimeOfDay),
    /// One of a fixed list, chosen from a picker.
    Choice {
        options: Vec<Choice>,
        selected: usize,
    },
    /// One value from a catalogue list. Free text is still accepted, since a
    /// study may be the first to use a new type.
    Catalog {
        vocabulary: Vocabulary,
        value: String,
    },
    /// Several values from a catalogue list, e.g. the health metrics a task
    /// reads.
    CatalogSet {
        vocabulary: Vocabulary,
        values: Vec<String>,
    },
}

impl FieldValue {
    /// What the field currently holds, rendered for display.
    pub fn display(&self) -> String {
        match self {
            Self::Text(text) => {
                if text.is_empty() {
                    "—".to_owned()
                } else {
                    text.clone()
                }
            }
            Self::Integer { value, .. } => value.to_string(),
            Self::Toggle(on) => if *on { "yes" } else { "no" }.to_owned(),
            Self::Duration(duration) => duration.human(),
            Self::Time(time) => time.label(),
            Self::Choice { options, selected } => options
                .get(*selected)
                .map_or_else(|| "—".to_owned(), |choice| choice.label.clone()),
            Self::Catalog { value, .. } => {
                if value.is_empty() {
                    "—".to_owned()
                } else {
                    value.clone()
                }
            }
            Self::CatalogSet { values, .. } => match values.len() {
                0 => "—".to_owned(),
                1 => values[0].clone(),
                count => format!("{count} selected"),
            },
        }
    }

    /// The text shown while the field is being typed into.
    ///
    /// `None` for fields that are not typed at all - a toggle, a choice - so
    /// the form knows to handle them with a key rather than a text buffer.
    pub fn editable_text(&self) -> Option<String> {
        match self {
            Self::Text(text) => Some(text.clone()),
            Self::Integer { value, .. } => Some(value.to_string()),
            Self::Duration(duration) => Some(duration.human()),
            Self::Time(time) => Some(time.label()),
            Self::Toggle(_)
            | Self::Choice { .. }
            | Self::Catalog { .. }
            | Self::CatalogSet { .. } => None,
        }
    }

    /// Store `text`, if it is a value this field accepts.
    ///
    /// Returns the reason it was refused, so the form can say why rather than
    /// silently discarding what was typed.
    pub fn accept_text(&mut self, text: &str) -> Result<(), String> {
        match self {
            Self::Text(current) => {
                *current = text.to_owned();
                Ok(())
            }
            Self::Integer { value, min, max } => {
                let parsed: i64 = text
                    .trim()
                    .parse()
                    .map_err(|_| format!("{text:?} is not a whole number"))?;
                if parsed < *min || parsed > *max {
                    return Err(format!("must be between {min} and {max}"));
                }
                *value = parsed;
                Ok(())
            }
            Self::Duration(duration) => {
                *duration = Micros::parse(text)
                    .ok_or_else(|| format!("{text:?} is not a duration, try 30d or 1h30m"))?;
                Ok(())
            }
            Self::Time(time) => {
                *time = TimeOfDay::parse(text)
                    .ok_or_else(|| format!("{text:?} is not a time, try 20:00"))?;
                Ok(())
            }
            Self::Toggle(_)
            | Self::Choice { .. }
            | Self::Catalog { .. }
            | Self::CatalogSet { .. } => Err("this field is not typed into".to_owned()),
        }
    }
}

/// One labelled, editable value.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// Stable identifier, used to read the value back out of a form. Never
    /// shown.
    pub key: &'static str,
    /// What the field is called on screen.
    pub label: String,
    /// One line under the form explaining the field, shown while it is the
    /// selected one.
    pub help: String,
    pub value: FieldValue,
}

impl Field {
    pub fn new(key: &'static str, label: impl Into<String>, value: FieldValue) -> Self {
        Self {
            key,
            label: label.into(),
            help: String::new(),
            value,
        }
    }

    /// Attach the explanation shown while the field is selected.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = help.into();
        self
    }
}

#[cfg(test)]
mod tests;
