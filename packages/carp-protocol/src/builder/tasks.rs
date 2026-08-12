// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Adding, renaming and removing tasks.
//!
//! See [`super`] for why every mutation goes through this module.

use crate::control::TaskControl;
use crate::protocol::StudyProtocol;
use crate::task::TaskKind;
use crate::trigger::TriggerKind;

use super::{Removal, unique_name};

/// Add a task of `kind` and the trigger that starts it, on `device`.
///
/// A task with no trigger never runs, so creating the two together is the
/// only sensible default. Pass [`TriggerKind::NoOp`] for a task the app
/// starts itself.
///
/// Returns the task's name.
pub fn add_task(
    protocol: &mut StudyProtocol,
    kind: TaskKind,
    device: &str,
    trigger_kind: TriggerKind,
) -> String {
    let name = unique_name(default_task_name(kind), &protocol.task_names());
    protocol.tasks.push(kind.instantiate(name.clone()));

    let trigger_id = protocol.next_trigger_id();
    protocol
        .triggers
        .insert(trigger_id, trigger_kind.instantiate(device.to_owned()));
    protocol
        .task_controls
        .push(TaskControl::start(trigger_id, name.clone(), device));

    name
}

/// The name a new task of `kind` starts with.
fn default_task_name(kind: TaskKind) -> &'static str {
    match kind {
        TaskKind::Background => "Background Task",
        TaskKind::Monitoring => "Monitoring",
        TaskKind::App => "App Task",
        TaskKind::RpApp => "Survey",
        TaskKind::HealthApp => "Health Task",
        TaskKind::Web => "Web Task",
    }
}

/// Rename a task and every reference to it: the task controls that start it,
/// and the triggers that watch it.
pub fn rename_task(protocol: &mut StudyProtocol, from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }
    if to.trim().is_empty() || protocol.task(to).is_some() {
        return false;
    }

    let Some(task) = protocol.tasks.iter_mut().find(|task| task.name() == from) else {
        return false;
    };
    task.set_name(to);

    for control in &mut protocol.task_controls {
        if control.task_name == from {
            to.clone_into(&mut control.task_name);
        }
    }
    for trigger in protocol.triggers.values_mut() {
        if trigger.watched_task() == Some(from) {
            trigger.set_watched_task(to);
        }
    }
    true
}

/// Remove a task, the task controls that started it, and any trigger left
/// with nothing to start.
pub fn remove_task(protocol: &mut StudyProtocol, name: &str) -> Removal {
    let mut removal = Removal::default();

    let before = protocol.tasks.len();
    protocol.tasks.retain(|task| task.name() != name);
    removal.tasks = before - protocol.tasks.len();

    let orphaned: Vec<u32> = protocol
        .task_controls
        .iter()
        .filter(|control| control.task_name == name)
        .map(|control| control.trigger_id)
        .collect();

    let before = protocol.task_controls.len();
    protocol.task_controls.retain(|control| control.task_name != name);
    removal.task_controls = before - protocol.task_controls.len();

    // A trigger that started only this task now starts nothing.
    for trigger_id in orphaned {
        let still_used = protocol
            .task_controls
            .iter()
            .any(|control| control.trigger_id == trigger_id);
        if !still_used && protocol.triggers.remove(&trigger_id).is_some() {
            removal.triggers += 1;
        }
    }

    removal
}
