// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Applying a trigger form.

use carp_protocol::StudyProtocol;
use carp_protocol::trigger::{KnownTrigger, Recurrence, Trigger, kind::set_recurrence};

use crate::app::form::Form;

use super::Applied;

/// Write a trigger form back.
///
/// The recurrence is the one field that is not a straight assignment: CARP
/// stores it three times over - as a `type`, a `period` and a `dayOfWeek` -
/// and [`set_recurrence`] moves all three together so the editor and the
/// phone cannot disagree about when the trigger fires.
pub fn apply(protocol: &mut StudyProtocol, form: &Form, id: u32) -> Applied {
    if !protocol.triggers.contains_key(&id) {
        return Applied::Vanished;
    }

    let source = form.text("source_device");
    if !source.is_empty() && protocol.device(&source).is_none() {
        return Applied::Refused(format!("{source:?} is not a device in this protocol"));
    }

    // Read the recurrence before borrowing the trigger mutably, since
    // `set_recurrence` needs the trigger itself.
    let recurrence = Recurrence::parse(&form.text("recurrence"));
    let day = form
        .value("day_of_week")
        .and_then(crate::app::form::FieldValue::as_str)
        .and_then(|value| value.parse::<u8>().ok());

    let Some(trigger) = protocol.triggers.get_mut(&id) else {
        return Applied::Vanished;
    };
    if !source.is_empty() {
        trigger.set_source_device(source);
    }

    if let Trigger::Known(known) = trigger {
        match known.as_mut() {
            KnownTrigger::Periodic { period, .. } => {
                if let Some(value) = form.duration("period") {
                    *period = value;
                }
            }
            KnownTrigger::RecurrentScheduled {
                time,
                separation_count,
                ..
            } => {
                if let Some(value) = form.time("time") {
                    *time = value;
                }
                if let Some(count) = form.integer("separation_count") {
                    *separation_count = count as u32;
                }
            }
            KnownTrigger::CronScheduled {
                cron_expression, ..
            } => {
                let expression = form.text("cron");
                if expression.split_whitespace().count() != 5 {
                    return Applied::Refused(
                        "a cron expression has five fields: minute hour day month weekday"
                            .to_owned(),
                    );
                }
                *cron_expression = expression;
            }
            KnownTrigger::UserTask {
                task_name,
                trigger_condition,
                ..
            } => {
                *task_name = form.text("watched_task");
                *trigger_condition = form.text("condition");
            }
            KnownTrigger::NoUserTask { task_name, .. } => {
                *task_name = form.text("watched_task");
            }
            KnownTrigger::SamplingEvent { measure_type, .. } => {
                *measure_type = form.text("measure_type");
            }
            KnownTrigger::NoOp { .. }
            | KnownTrigger::Immediate { .. }
            | KnownTrigger::OneTime { .. } => {}
        }
    }

    // Done last, and through the helper, so `type`, `period` and `dayOfWeek`
    // end up agreeing whatever the form said individually.
    if let Some(recurrence) = recurrence
        && let Some(trigger) = protocol.triggers.get_mut(&id)
    {
        set_recurrence(trigger, recurrence, day);
    }

    Applied::Changed
}
