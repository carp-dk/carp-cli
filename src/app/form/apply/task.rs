// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Applying a task form.

use carp_protocol::Micros;
use carp_protocol::StudyProtocol;
use carp_protocol::builder;
use carp_protocol::task::{KnownTask, Task};

use crate::app::form::Form;

use super::Applied;

/// Write a task form back, renaming through [`builder`] so that the task
/// controls starting it and the triggers watching it move with it.
pub fn apply(protocol: &mut StudyProtocol, form: &Form, name: &str) -> Applied {
    if protocol.task(name).is_none() {
        return Applied::Vanished;
    }

    let new_name = form.text("name");
    if new_name.trim().is_empty() {
        return Applied::Refused("a task needs a name".to_owned());
    }
    if new_name != name && !builder::rename_task(protocol, name, &new_name) {
        return Applied::Refused(format!("another task is already called {new_name:?}"));
    }

    let Some(task) = protocol
        .tasks
        .iter_mut()
        .find(|task| task.name() == new_name)
    else {
        return Applied::Vanished;
    };

    if let Some(app) = task.app_mut() {
        app.r#type = form.text("type");
        app.title = form.text("title");
        app.description = form.text("description");
        app.instructions = form.text("instructions");
    }

    let Task::Known(known) = task else {
        return Applied::Changed;
    };

    match known.as_mut() {
        KnownTask::App {
            minutes_to_complete,
            notification,
            ..
        } => {
            *minutes_to_complete = minutes(form);
            *notification = form.flag("notification");
        }
        KnownTask::RpApp {
            minutes_to_complete,
            expire,
            notification,
            rp_task,
            ..
        } => {
            *minutes_to_complete = minutes(form);
            *notification = form.flag("notification");

            // Zero stands for "never expires", which the wire format writes
            // by omitting the field rather than by storing a zero.
            *expire = match form.duration("expire") {
                Some(duration) if duration > Micros::ZERO => Some(duration),
                _ => None,
            };

            let identifier = form.text("survey_identifier");
            if !identifier.trim().is_empty() {
                rp_task.set_identifier(identifier);
            }
            // Turning branching off discards the rules, since an ordered
            // survey has nowhere to keep them. The steps are untouched.
            rp_task.set_navigable(form.flag("survey_navigable"));
        }
        KnownTask::HealthApp {
            notification,
            types,
            ..
        } => {
            *notification = form.flag("notification");
            *types = form.set("types");
        }
        KnownTask::Web {
            description, url, ..
        } => {
            *description = form.text("web_description");
            *url = form.text("url");
        }
        KnownTask::Background { .. } | KnownTask::Monitoring { .. } => {}
    }

    Applied::Changed
}

/// The estimate on a task card, absent when zero.
fn minutes(form: &Form) -> Option<u32> {
    match form.integer("minutes_to_complete") {
        Some(value) if value > 0 => Some(value as u32),
        _ => None,
    }
}
