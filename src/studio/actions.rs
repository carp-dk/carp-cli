// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! What the editor's keys do, routed by section.
//!
//! Every function here takes a checkpoint before changing anything, so the
//! change can be undone, and calls [`Studio::changed`] afterwards so the
//! checks and the cursors follow. Structural changes go through
//! [`carp_protocol::builder`], never through the protocol's fields, which is
//! what keeps references intact.

pub mod survey;

pub use survey::{add_step, move_step, open_survey};

use carp_protocol::task::Task;
use carp_protocol::{Measure, builder};

use crate::app::form::build;

use super::{Section, Studio};

/// Open the form for whatever is selected in `section`.
pub fn edit_selected(studio: &mut Studio) -> Option<String> {
    let protocol = &studio.protocol;
    studio.form = match studio.section {
        Section::Overview => Some(build::protocol(protocol)),
        Section::Devices => studio.lists.selected_device(protocol).map(build::device),
        Section::Tasks => studio.lists.selected_task(protocol).map(build::task),
        Section::Triggers => studio
            .lists
            .selected_trigger(protocol)
            .map(|(id, trigger)| build::trigger(id, trigger, protocol)),
        Section::Participants => studio
            .lists
            .selected_role(protocol)
            .map(build::participant_role),
        Section::Survey => {
            let task = studio.survey_task_name()?;
            let index = studio.lists.steps.selected()?;
            let step = protocol.task(&task)?.survey()?.steps().get(index)?;
            Some(build::survey_step(&task, index, step))
        }
        Section::Catalog | Section::Checks => None,
    };

    if studio.form.is_none() {
        return Some("nothing to edit here".to_owned());
    }
    None
}

/// Open the form for the measure under the measures cursor.
pub fn edit_selected_measure(studio: &mut Studio) -> Option<String> {
    let task = studio.lists.selected_task_name(&studio.protocol)?;
    let index = studio.lists.measures.selected()?;
    let measure = studio.protocol.task(&task)?.measures().get(index)?;
    studio.form = Some(build::measure(&task, index, measure));
    None
}

/// Open the form for the expected-data entry under its cursor.
pub fn edit_selected_expected(studio: &mut Studio) -> Option<String> {
    let index = studio.lists.expected.selected()?;
    let expected = studio.protocol.expected_participant_data.get(index)?;
    studio.form = Some(build::expected_data(index, expected, &studio.protocol));
    None
}

/// Remove whatever is selected in `section`, reporting what went with it.
pub fn remove_selected(studio: &mut Studio) -> Option<String> {
    match studio.section {
        Section::Devices => remove_device(studio),
        Section::Tasks => remove_task(studio),
        Section::Triggers => remove_trigger(studio),
        Section::Participants => remove_role(studio),
        Section::Survey => remove_step(studio),
        Section::Overview | Section::Catalog | Section::Checks => {
            Some("nothing to remove here".to_owned())
        }
    }
}

fn remove_device(studio: &mut Studio) -> Option<String> {
    let role = studio.lists.selected_device_role(&studio.protocol)?;
    studio.checkpoint();
    let removal = builder::remove_device(&mut studio.protocol, &role);
    studio.changed();

    Some(if removal.is_empty() {
        format!("removed {role}")
    } else {
        format!("removed {role}, and with it {}", removal.summary())
    })
}

fn remove_task(studio: &mut Studio) -> Option<String> {
    let name = studio.lists.selected_task_name(&studio.protocol)?;
    studio.checkpoint();
    let removal = builder::remove_task(&mut studio.protocol, &name);
    if studio.survey_task.as_deref() == Some(name.as_str()) {
        studio.survey_task = None;
    }
    studio.changed();

    Some(if removal.triggers > 0 || removal.task_controls > 0 {
        format!("removed {name}, and with it {}", removal.summary())
    } else {
        format!("removed {name}")
    })
}

fn remove_trigger(studio: &mut Studio) -> Option<String> {
    let id = studio.lists.selected_trigger_id(&studio.protocol)?;
    studio.checkpoint();
    let removal = builder::remove_trigger(&mut studio.protocol, id);
    studio.changed();

    Some(if removal.task_controls > 0 {
        format!("removed trigger {id}, and with it {}", removal.summary())
    } else {
        format!("removed trigger {id}")
    })
}

fn remove_role(studio: &mut Studio) -> Option<String> {
    let role = studio.lists.selected_role(&studio.protocol)?.role.clone();
    studio.checkpoint();
    builder::remove_participant_role(&mut studio.protocol, &role);
    studio.changed();
    Some(format!("removed the {role} role"))
}

fn remove_step(studio: &mut Studio) -> Option<String> {
    let task = studio.survey_task_name()?;
    let index = studio.lists.steps.selected()?;

    studio.checkpoint();
    let removed = studio
        .protocol
        .tasks
        .iter_mut()
        .find(|candidate| candidate.name() == task)
        .and_then(Task::survey_mut)
        .and_then(carp_protocol::survey::RpTask::steps_mut)
        .filter(|steps| index < steps.len())
        .map(|steps| steps.remove(index));

    match removed {
        Some(step) => {
            studio.changed();
            Some(format!("removed step {}", step.identifier()))
        }
        None => {
            studio.history.pop();
            None
        }
    }
}

/// Remove the measure under the measures cursor.
pub fn remove_selected_measure(studio: &mut Studio) -> Option<String> {
    let task = studio.lists.selected_task_name(&studio.protocol)?;
    let index = studio.lists.measures.selected()?;

    studio.checkpoint();
    let removed = studio
        .protocol
        .tasks
        .iter_mut()
        .find(|candidate| candidate.name() == task)
        .and_then(Task::measures_mut)
        .filter(|measures| index < measures.len())
        .map(|measures| measures.remove(index));

    match removed {
        Some(measure) => {
            studio.changed();
            Some(format!("removed the {} measure", measure.short_name()))
        }
        None => {
            studio.history.pop();
            None
        }
    }
}

/// Remove the expected-data entry under its cursor.
pub fn remove_selected_expected(studio: &mut Studio) -> Option<String> {
    let index = studio.lists.expected.selected()?;
    if index >= studio.protocol.expected_participant_data.len() {
        return None;
    }

    studio.checkpoint();
    let removed = studio.protocol.expected_participant_data.remove(index);
    studio.changed();
    Some(format!("no longer asking for {}", removed.input_data_type()))
}

/// Add a measure to the selected task, opening a picker for its type.
pub fn add_measure(studio: &mut Studio) -> Option<String> {
    let task = studio.lists.selected_task_name(&studio.protocol)?;
    // An unmodelled task keeps its measures verbatim; there is nowhere to add
    // one, and saying so beats appearing to add one that never arrives.
    if matches!(studio.protocol.task(&task)?, Task::Unknown(_)) {
        return Some("this task's measures cannot be edited here".to_owned());
    }

    studio.checkpoint();
    let index = studio
        .protocol
        .tasks
        .iter_mut()
        .find(|candidate| candidate.name() == task)
        .and_then(Task::measures_mut)
        .map(|measures| {
            // Added empty and immediately opened for editing, so the picker
            // is what names it rather than a placeholder nobody notices.
            measures.push(Measure::data_stream(String::new()));
            measures.len() - 1
        })?;

    studio.lists.measures.select(Some(index));
    studio.changed();
    edit_selected_measure(studio)
}

/// Add an expected-data entry, opening its form.
pub fn add_expected(studio: &mut Studio) -> Option<String> {
    if studio.protocol.participant_roles.is_empty() {
        return Some("add a participant role first".to_owned());
    }

    studio.checkpoint();
    let role = studio.protocol.participant_roles[0].role.clone();
    studio
        .protocol
        .expected_participant_data
        .push(carp_protocol::participant::ExpectedParticipantData::for_roles(
            "dk.carp.webservices.input.informed_consent",
            vec![role],
        ));
    let index = studio.protocol.expected_participant_data.len() - 1;
    studio.lists.expected.select(Some(index));
    studio.changed();
    edit_selected_expected(studio)
}

/// Add a participant role and open its form.
pub fn add_role(studio: &mut Studio) -> Option<String> {
    studio.checkpoint();
    let name = builder::add_participant_role(&mut studio.protocol, "Participant");
    let index = studio
        .protocol
        .participant_roles
        .iter()
        .position(|role| role.role == name)?;
    studio.lists.roles.select(Some(index));
    studio.changed();
    edit_selected(studio)
}

