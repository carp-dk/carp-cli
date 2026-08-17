// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Opening the right picker, and acting on what it returns.
//!
//! Two kinds of picker meet here, and they are resolved differently:
//!
//! - a picker over a **field** writes its value back into the open form. The
//!   form knows which field is selected, so the picker only has to hand back
//!   a string.
//! - a picker over a **thing to create** - a device class, a task kind, a
//!   step type - has to build that thing, which is what [`resolve`] does by
//!   reading the [`Creating`] the picker was opened with.

pub mod create;

use create::{create, device_rows, step_rows, task_rows, template_rows, trigger_rows};

use crate::app::form::FieldValue;
use crate::app::form::picker::{Picker, PickerKind, Row};

use super::{Section, Studio, actions};

/// What a creating picker is choosing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Creating {
    Device,
    Task,
    Trigger,
    SurveyStep,
    /// A study from the catalogue, to start a protocol from.
    Template,
}

/// Open the picker for adding something to `section`.
pub fn open_add(studio: &mut Studio) -> Option<String> {
    let (creating, title, rows) = match studio.section {
        Section::Devices => (Creating::Device, "add a device", device_rows()),
        Section::Tasks => (Creating::Task, "add a task", task_rows()),
        Section::Triggers => (Creating::Trigger, "add a trigger", trigger_rows()),
        Section::Survey => (Creating::SurveyStep, "add a step", step_rows()),
        Section::Catalog => (
            Creating::Template,
            "start from a study",
            template_rows(studio),
        ),
        Section::Participants => return actions::add_role(studio),
        Section::Overview | Section::Checks => {
            return Some("nothing to add here".to_owned());
        }
    };

    if rows.is_empty() {
        return Some("nothing available to add".to_owned());
    }

    studio.creating = Some(creating);
    studio.picker = Some(Picker::new(title, PickerKind::Create, rows, ""));
    None
}

/// Open the picker for the form's currently selected field.
///
/// Returns false when the field is not one a picker serves, so the caller can
/// fall back to typing.
pub fn open_for_field(studio: &mut Studio) -> bool {
    let Some(form) = studio.form.as_ref() else {
        return false;
    };
    let Some(field) = form.selected_field() else {
        return false;
    };

    let picker = match &field.value {
        FieldValue::Catalog { vocabulary, value } => {
            let rows = Picker::rows_from_catalog(vocabulary.entries(&studio.catalog));
            // A new measure type has to be enterable somewhere, and the
            // picker is where the known ones are.
            Picker::new(vocabulary.title(), PickerKind::Single, rows, value).allowing_free_text()
        }
        FieldValue::CatalogSet { vocabulary, values } => {
            let rows = Picker::rows_from_catalog(vocabulary.entries(&studio.catalog));
            Picker::multiple(vocabulary.title(), rows, values.clone())
        }
        FieldValue::Choice { options, selected } => {
            let rows = options
                .iter()
                .map(|option| Row::new(&option.value, &option.label, &option.description))
                .collect();
            let current = options
                .get(*selected)
                .map(|option| option.value.clone())
                .unwrap_or_default();
            Picker::new(field.label.clone(), PickerKind::Single, rows, &current)
        }
        _ => return false,
    };

    studio.creating = None;
    studio.picker = Some(picker);
    true
}

/// Act on the open picker's result and close it.
pub fn resolve(studio: &mut Studio) -> Option<String> {
    let picker = studio.picker.take()?;

    // A multi-select hands back its whole set rather than one row.
    if picker.kind == PickerKind::Multiple {
        if let Some(form) = studio.form.as_mut() {
            form.set_selected_many(picker.chosen);
        }
        return None;
    }

    let value = picker.resolve()?;

    // A field picker only has to hand the value to the form.
    let Some(creating) = studio.creating.take() else {
        if let Some(form) = studio.form.as_mut() {
            form.set_selected(value);
        }
        return None;
    };

    create(studio, creating, &value)
}
