// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Turning a creating picker's answer into the thing it names.

use carp_protocol::device::DeviceKind;
use carp_protocol::survey::{RpAnswerFormat, RpStep};
use carp_protocol::task::TaskKind;
use carp_protocol::trigger::TriggerKind;
use carp_protocol::{builder, task::kind::survey_identifier};

use crate::app::form::picker::Row;
use crate::studio::{Section, Studio, actions};

use super::Creating;

/// Build and add the thing a creating picker chose.
pub(super) fn create(studio: &mut Studio, creating: Creating, value: &str) -> Option<String> {
    match creating {
        Creating::Device => {
            let kind = DeviceKind::from_type_name(value)?;
            studio.checkpoint();
            let role = builder::add_device(&mut studio.protocol, kind);
            studio.changed();
            select_device(studio, &role);
            actions::edit_selected(studio)
        }
        Creating::Task => {
            let kind = TaskKind::from_type_name(value)?;
            // Every task needs a device to run on and a trigger to start it.
            let device = studio
                .protocol
                .primary_devices
                .first()
                .map(|device| device.role_name().to_owned())?;

            studio.checkpoint();
            let trigger = default_trigger_for(kind);
            let name = builder::add_task(&mut studio.protocol, kind, &device, trigger);
            studio.changed();
            select_task(studio, &name);
            actions::edit_selected(studio)
        }
        Creating::Trigger => {
            let kind = TriggerKind::from_type_name(value)?;
            let device = studio
                .protocol
                .primary_devices
                .first()
                .map(|device| device.role_name().to_owned())?;

            studio.checkpoint();
            let id = builder::add_trigger(&mut studio.protocol, kind, &device);
            studio.changed();
            select_trigger(studio, id);
            actions::edit_selected(studio)
        }
        Creating::SurveyStep => {
            let task = studio.survey_task_name()?;
            let identifier = next_step_identifier(studio, &task);
            actions::add_step(studio, build_step(value, &identifier)?)
        }
        Creating::Template => start_from_template(studio, value),
    }
}

/// Replace the edited protocol with a copy of the upstream study `value`.
///
/// The copy is *forked*, not cloned: it gets a new id and starts at revision
/// 0, so uploading it creates the researcher's own protocol rather than
/// filing a new version of the demo study. The owner is kept, since it is the
/// person doing the work rather than anything about the template.
///
/// Unsaved work is not silently discarded - the caller is asked to save first.
fn start_from_template(studio: &mut Studio, study: &str) -> Option<String> {
    if studio.dirty {
        return Some("save or discard the current protocol first".to_owned());
    }
    let snapshot = studio.snapshot.as_ref()?;

    let mut protocol = match snapshot.template(study) {
        Ok(protocol) => protocol,
        Err(error) => return Some(format!("could not read {study}: {error}")),
    };

    let owner = studio.protocol.owner_id.clone();
    let name = format!("{} (copy)", protocol.name);
    carp_protocol::version::fork(&mut protocol, name);
    protocol.owner_id = owner;

    studio.protocol = protocol;
    studio.path = None;
    studio.history.clear();
    studio.survey_task = None;
    studio.section = Section::Overview;
    studio.changed();

    Some(format!("started from the {study} study"))
}

/// The trigger a new task of `kind` is most usefully started by.
fn default_trigger_for(kind: TaskKind) -> TriggerKind {
    match kind {
        // A monitoring task is started by the runtime, not by a schedule.
        TaskKind::Monitoring => TriggerKind::NoOp,
        // A survey almost always wants a schedule; the editor opens on it so
        // the time can be set straight away.
        TaskKind::RpApp => TriggerKind::RecurrentScheduled,
        _ => TriggerKind::Immediate,
    }
}

/// A step identifier not already used in the survey.
fn next_step_identifier(studio: &Studio, task: &str) -> String {
    let taken = studio
        .protocol
        .task(task)
        .and_then(carp_protocol::task::Task::survey)
        .map(carp_protocol::survey::RpTask::all_step_identifiers)
        .unwrap_or_default();
    builder::unique_name(&format!("{}_step", survey_identifier(task)), &taken)
}

/// A step of the type the picker named.
fn build_step(type_name: &str, identifier: &str) -> Option<RpStep> {
    Some(match type_name {
        "RPInstructionStep" => RpStep::instruction(identifier, "Instructions", ""),
        "RPCompletionStep" => RpStep::completion(identifier, "Thank you", ""),
        "RPQuestionStep" => RpStep::question(
            identifier,
            "Question",
            RpAnswerFormat::single_choice(vec![
                carp_protocol::RpChoice::new("Yes", 1),
                carp_protocol::RpChoice::new("No", 0),
            ]),
        ),
        "RPScaleQuestionStep" => RpStep::question(
            identifier,
            "Question",
            RpAnswerFormat::slider(0.0, 10.0, 10),
        ),
        "RPTextQuestionStep" => {
            RpStep::question(identifier, "Question", RpAnswerFormat::text(None))
        }
        "RPIntegerQuestionStep" => {
            RpStep::question(identifier, "Question", RpAnswerFormat::integer(0, 100, ""))
        }
        "RPDateTimeQuestionStep" => RpStep::question(
            identifier,
            "Question",
            RpAnswerFormat::date_time("DateAndTime"),
        ),
        _ => return None,
    })
}

fn select_device(studio: &mut Studio, role: &str) {
    if let Some(index) = studio
        .protocol
        .devices()
        .position(|device| device.role_name() == role)
    {
        studio.lists.devices.select(Some(index));
    }
}

fn select_task(studio: &mut Studio, name: &str) {
    if let Some(index) = studio
        .protocol
        .tasks
        .iter()
        .position(|task| task.name() == name)
    {
        studio.lists.tasks.select(Some(index));
    }
}

fn select_trigger(studio: &mut Studio, id: u32) {
    if let Some(index) = studio.protocol.triggers.keys().position(|key| *key == id) {
        studio.lists.triggers.select(Some(index));
    }
}

pub(super) fn device_rows() -> Vec<Row> {
    DeviceKind::ALL
        .into_iter()
        .map(|kind| Row::new(kind.type_name(), kind.label(), kind.description()))
        .collect()
}

pub(super) fn task_rows() -> Vec<Row> {
    TaskKind::ALL
        .into_iter()
        .map(|kind| Row::new(kind.type_name(), kind.label(), kind.description()))
        .collect()
}

pub(super) fn trigger_rows() -> Vec<Row> {
    TriggerKind::ALL
        .into_iter()
        .map(|kind| Row::new(kind.type_name(), kind.label(), kind.description()))
        .collect()
}

/// Step types, with the question variants split out by answer format: they
/// are one class on the wire but four different things to author.
pub(super) fn step_rows() -> Vec<Row> {
    vec![
        Row::new(
            "RPQuestionStep",
            "Choice question",
            "Pick one of a list of options",
        ),
        Row::new(
            "RPScaleQuestionStep",
            "Scale question",
            "A slider over a range",
        ),
        Row::new("RPTextQuestionStep", "Text question", "Free text"),
        Row::new("RPIntegerQuestionStep", "Number question", "A whole number"),
        Row::new(
            "RPDateTimeQuestionStep",
            "Date/time question",
            "A date, a time, or both",
        ),
        Row::new(
            "RPInstructionStep",
            "Instructions",
            "A page of text before the questions",
        ),
        Row::new("RPCompletionStep", "Completion", "The closing page"),
    ]
}

pub(super) fn template_rows(studio: &Studio) -> Vec<Row> {
    studio
        .catalog
        .templates
        .iter()
        .map(|template| {
            Row::new(
                &template.study,
                &template.name,
                format!("{} · {}", template.study, template.summary),
            )
        })
        .collect()
}
