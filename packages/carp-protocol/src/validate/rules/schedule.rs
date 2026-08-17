// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Whether the triggers and task controls resolve to real things.
//!
//! These are the rules that matter most: the whole graph is joined by name,
//! and JSON cannot say that a name has to resolve.

use std::collections::HashSet;

use super::super::Diagnostic;
use crate::device::Device;
use crate::protocol::StudyProtocol;
use crate::task::Task;
use crate::trigger::Trigger;

/// Triggers name a device that exists, and a task that exists when they watch
/// one.
pub fn triggers(protocol: &StudyProtocol, out: &mut Vec<Diagnostic>) {
    let devices: HashSet<&str> = protocol.devices().map(Device::role_name).collect();
    let tasks: HashSet<&str> = protocol.tasks.iter().map(Task::name).collect();

    for (id, trigger) in &protocol.triggers {
        let source = trigger.source_device();
        if source.trim().is_empty() {
            out.push(Diagnostic::error(
                format!("trigger {id}"),
                "names no source device",
            ));
        } else if !devices.contains(source) {
            out.push(
                Diagnostic::error(
                    format!("trigger {id}"),
                    format!("fires on {source:?}, which is not a device in this protocol"),
                )
                .with_hint("point it at one of the protocol's devices"),
            );
        }

        if let Some(watched) = trigger.watched_task() {
            if watched.trim().is_empty() {
                out.push(Diagnostic::error(
                    format!("trigger {id}"),
                    "watches a task but names none",
                ));
            } else if !tasks.contains(watched) {
                out.push(Diagnostic::error(
                    format!("trigger {id}"),
                    format!("watches {watched:?}, which is not a task in this protocol"),
                ));
            }
        }

        if let Some(kind) = trigger.kind()
            && kind == crate::trigger::TriggerKind::SamplingEvent
            && sampling_measure(trigger).is_none_or(str::is_empty)
        {
            out.push(Diagnostic::error(
                format!("trigger {id}"),
                "watches no measure type",
            ));
        }
    }
}

/// The measure type a sampling-event trigger watches.
fn sampling_measure(trigger: &Trigger) -> Option<&str> {
    use crate::trigger::KnownTrigger;
    match trigger {
        Trigger::Known(known) => match known.as_ref() {
            KnownTrigger::SamplingEvent { measure_type, .. } => Some(measure_type),
            _ => None,
        },
        Trigger::Unknown(_) => None,
    }
}

/// Task controls join three names that all have to resolve, and every task
/// and trigger should be reachable through one.
pub fn task_controls(protocol: &StudyProtocol, out: &mut Vec<Diagnostic>) {
    let devices: HashSet<&str> = protocol.devices().map(Device::role_name).collect();
    let tasks: HashSet<&str> = protocol.tasks.iter().map(Task::name).collect();

    let mut controlled_tasks: HashSet<&str> = HashSet::new();
    let mut used_triggers: HashSet<u32> = HashSet::new();

    for control in &protocol.task_controls {
        let location = format!("control {} -> {:?}", control.trigger_id, control.task_name);

        if !protocol.triggers.contains_key(&control.trigger_id) {
            out.push(Diagnostic::error(
                &location,
                format!("trigger {} does not exist", control.trigger_id),
            ));
        } else {
            used_triggers.insert(control.trigger_id);
        }

        if tasks.contains(control.task_name.as_str()) {
            controlled_tasks.insert(control.task_name.as_str());
        } else {
            out.push(
                Diagnostic::error(
                    &location,
                    format!("task {:?} does not exist", control.task_name),
                )
                .with_hint("a task was probably renamed or removed"),
            );
        }

        if !devices.contains(control.destination_device_role_name.as_str()) {
            out.push(Diagnostic::error(
                &location,
                format!(
                    "device {:?} does not exist",
                    control.destination_device_role_name
                ),
            ));
        }
    }

    for task in &protocol.tasks {
        if !controlled_tasks.contains(task.name()) {
            out.push(
                Diagnostic::warning(
                    format!("task {:?}", task.name()),
                    "is not started by any trigger",
                )
                .with_hint("add a task control, or the task never runs"),
            );
        }
    }

    for id in protocol.triggers.keys() {
        if !used_triggers.contains(id) {
            out.push(
                Diagnostic::warning(format!("trigger {id}"), "starts no task")
                    .with_hint("add a task control, or remove the trigger"),
            );
        }
    }
}
