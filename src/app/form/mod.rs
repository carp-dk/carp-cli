// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The editing surface every part of a protocol is changed through.
//!
//! A [`Form`] is a list of [`Field`]s plus a cursor, and it has exactly two
//! states: *browsing*, where the arrow keys move between fields, and
//! *typing*, where keystrokes go into the selected field's buffer. Nothing
//! is written back to the protocol until the form is submitted, so `Esc`
//! reliably means "leave this as it was".
//!
//! # Layout
//!
//! - [`field`] - what a single value is, and what it accepts
//! - [`build`] - turning a device, task or trigger into a form
//! - [`apply`] - writing a submitted form back onto one
//! - [`picker`] - the overlay for fields chosen from a list
//!
//! Keeping build and apply as separate modules rather than a trait is
//! deliberate: the two are asymmetric. Building reads whatever a value has;
//! applying has to go through [`carp_protocol::builder`] so that renaming a
//! device also moves every reference to it.

pub mod apply;
pub mod build;
pub mod field;
pub mod picker;
pub mod read;

pub use field::{Choice, Field, FieldValue, Vocabulary};

/// What the form is doing with keystrokes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Arrow keys move between fields.
    Browsing,
    /// Keystrokes go into `buffer`, which replaces the selected field's value
    /// when the field is committed.
    Typing { buffer: String },
}

/// What the form is editing, so the submitted values can be written back to
/// the right place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subject {
    /// The protocol's own name, owner and description.
    Protocol,
    /// The CAMS `applicationData` block.
    ApplicationData,
    /// Where the study's data is written.
    DataEndPoint,
    /// The device with this role name.
    Device(String),
    /// The task with this name.
    Task(String),
    /// The trigger with this id.
    Trigger(u32),
    /// The participant role with this name.
    ParticipantRole(String),
    /// The expected-participant-data entry at this index.
    ExpectedData(usize),
    /// The survey step at this index of the task's survey.
    SurveyStep { task: String, step: usize },
    /// A measure at this index of the task's measures.
    Measure { task: String, measure: usize },
}

impl Subject {
    /// What the form's title bar says it is editing.
    pub fn title(&self) -> String {
        match self {
            Self::Protocol => "protocol".to_owned(),
            Self::ApplicationData => "study app settings".to_owned(),
            Self::DataEndPoint => "where the data goes".to_owned(),
            Self::Device(role) => format!("device · {role}"),
            Self::Task(name) => format!("task · {name}"),
            Self::Trigger(id) => format!("trigger · {id}"),
            Self::ParticipantRole(role) => format!("role · {role}"),
            Self::ExpectedData(_) => "expected participant data".to_owned(),
            Self::SurveyStep { task, step } => format!("step {} · {task}", step + 1),
            Self::Measure { task, .. } => format!("measure · {task}"),
        }
    }
}

/// An open editing form.
#[derive(Debug, Clone)]
pub struct Form {
    pub subject: Subject,
    pub fields: Vec<Field>,
    /// Index of the field the cursor is on.
    pub selected: usize,
    pub mode: Mode,
    /// Why the last commit was refused, cleared on the next keystroke.
    pub error: Option<String>,
    /// True once any field has been changed, so leaving can warn.
    pub dirty: bool,
}

impl Form {
    pub fn new(subject: Subject, fields: Vec<Field>) -> Self {
        Self {
            subject,
            fields,
            selected: 0,
            mode: Mode::Browsing,
            error: None,
            dirty: false,
        }
    }

    pub fn is_typing(&self) -> bool {
        matches!(self.mode, Mode::Typing { .. })
    }

    pub fn selected_field(&self) -> Option<&Field> {
        self.fields.get(self.selected)
    }

    /// Move the cursor by `delta`, stopping at the ends.
    ///
    /// Does nothing while typing: an arrow key there belongs to the text, and
    /// silently moving to another field would discard the buffer.
    pub fn move_selection(&mut self, delta: isize) {
        if self.is_typing() || self.fields.is_empty() {
            return;
        }
        let last = self.fields.len() as isize - 1;
        let next = (self.selected as isize)
            .saturating_add(delta)
            .clamp(0, last);
        self.selected = next as usize;
        self.error = None;
    }

    /// Begin typing into the selected field.
    ///
    /// Returns false for a field that is not typed into - a toggle, a picked
    /// value - so the caller can do whatever that field does instead.
    pub fn begin_typing(&mut self) -> bool {
        let Some(field) = self.fields.get(self.selected) else {
            return false;
        };
        let Some(buffer) = field.value.editable_text() else {
            return false;
        };
        self.mode = Mode::Typing { buffer };
        self.error = None;
        true
    }

    /// Append a character to the buffer.
    pub fn push(&mut self, character: char) {
        if let Mode::Typing { buffer } = &mut self.mode {
            buffer.push(character);
            self.error = None;
        }
    }

    /// Delete the last character of the buffer.
    pub fn backspace(&mut self) {
        if let Mode::Typing { buffer } = &mut self.mode {
            buffer.pop();
            self.error = None;
        }
    }

    /// Empty the buffer, as Ctrl-U does in a shell.
    pub fn clear_buffer(&mut self) {
        if let Mode::Typing { buffer } = &mut self.mode {
            buffer.clear();
            self.error = None;
        }
    }

    /// Store the buffer in the selected field and stop typing.
    ///
    /// A value the field refuses leaves the form typing, with the reason in
    /// [`Form::error`], so the text can be corrected rather than retyped.
    pub fn commit(&mut self) -> bool {
        let Mode::Typing { buffer } = &self.mode else {
            return false;
        };
        let buffer = buffer.clone();
        let Some(field) = self.fields.get_mut(self.selected) else {
            return false;
        };

        match field.value.accept_text(&buffer) {
            Ok(()) => {
                self.mode = Mode::Browsing;
                self.error = None;
                self.dirty = true;
                true
            }
            Err(reason) => {
                self.error = Some(reason);
                false
            }
        }
    }

    /// Stop typing, discarding the buffer.
    pub fn cancel_typing(&mut self) {
        self.mode = Mode::Browsing;
        self.error = None;
    }

    /// Flip a toggle, or step a choice to its next option.
    ///
    /// This is what space does, so the common edits need no picker at all.
    pub fn toggle_selected(&mut self) {
        let Some(field) = self.fields.get_mut(self.selected) else {
            return;
        };
        match &mut field.value {
            FieldValue::Toggle(on) => {
                *on = !*on;
                self.dirty = true;
            }
            FieldValue::Choice { options, selected } if !options.is_empty() => {
                *selected = (*selected + 1) % options.len();
                self.dirty = true;
            }
            _ => {}
        }
    }

    /// Store `value` in the selected field, as a picker's result.
    pub fn set_selected(&mut self, value: String) {
        let Some(field) = self.fields.get_mut(self.selected) else {
            return;
        };
        match &mut field.value {
            FieldValue::Catalog { value: current, .. } | FieldValue::Text(current) => {
                *current = value;
                self.dirty = true;
            }
            FieldValue::Choice { options, selected } => {
                if let Some(index) = options.iter().position(|option| option.value == value) {
                    *selected = index;
                    self.dirty = true;
                }
            }
            _ => {}
        }
    }

    /// Store a whole set in the selected field, as a multi-picker's result.
    pub fn set_selected_many(&mut self, values: Vec<String>) {
        if let Some(FieldValue::CatalogSet {
            values: current, ..
        }) = self
            .fields
            .get_mut(self.selected)
            .map(|field| &mut field.value)
        {
            *current = values;
            self.dirty = true;
        }
    }
}

#[cfg(test)]
mod tests;
