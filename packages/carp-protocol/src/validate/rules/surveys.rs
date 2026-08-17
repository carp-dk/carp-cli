// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Whether a survey's steps and branches hold together, and what this
//! build cannot show.

use std::collections::HashSet;

use super::super::Diagnostic;
use crate::device::Device;
use crate::protocol::StudyProtocol;
use crate::task::Task;
use crate::trigger::Trigger;

/// Survey step identifiers are unique, and navigation rules point at steps
/// that exist.
pub fn surveys(protocol: &StudyProtocol, out: &mut Vec<Diagnostic>) {
    for task in &protocol.tasks {
        let Some(survey) = task.survey() else {
            continue;
        };
        let location = format!("survey in task {:?}", task.name());

        if survey.identifier().trim().is_empty() {
            out.push(Diagnostic::error(&location, "has no identifier"));
        }

        let identifiers = survey.all_step_identifiers();
        if identifiers.is_empty() {
            out.push(
                Diagnostic::warning(&location, "has no steps")
                    .with_hint("the participant sees an empty survey"),
            );
        }

        let mut seen: HashSet<&str> = HashSet::new();
        for identifier in &identifiers {
            if identifier.trim().is_empty() {
                out.push(Diagnostic::error(&location, "a step has no identifier"));
            } else if !seen.insert(identifier.as_str()) {
                out.push(
                    Diagnostic::error(
                        &location,
                        format!("step identifier {identifier:?} is used twice"),
                    )
                    .with_hint("answers are recorded under the identifier"),
                );
            }
        }

        let Some(rules) = survey.navigation_rules() else {
            continue;
        };
        for (step, rule) in rules {
            if !seen.contains(step.as_str()) {
                out.push(Diagnostic::error(
                    &location,
                    format!("a navigation rule is attached to {step:?}, which is not a step"),
                ));
            }
            for destination in rule.destinations() {
                if !seen.contains(destination) {
                    out.push(Diagnostic::error(
                        &location,
                        format!(
                            "a branch of {step:?} jumps to {destination:?}, which is not a step"
                        ),
                    ));
                }
            }
        }
    }
}

/// Report values carried verbatim because this build does not model them.
///
/// They are preserved on save, but the editor cannot show their fields, so
/// saying so beats letting someone wonder why a device has no settings.
pub fn unmodelled_types(protocol: &StudyProtocol, out: &mut Vec<Diagnostic>) {
    for device in protocol.devices() {
        if let Device::Unknown(node) = device {
            out.push(
                Diagnostic::info(
                    format!("device {:?}", device.role_name()),
                    format!("{} is not known to this version", node.short_type()),
                )
                .with_hint("it is preserved unchanged, but cannot be edited here"),
            );
        }
    }
    for task in &protocol.tasks {
        if let Task::Unknown(node) = task {
            out.push(
                Diagnostic::info(
                    format!("task {:?}", task.name()),
                    format!("{} is not known to this version", node.short_type()),
                )
                .with_hint("it is preserved unchanged, but cannot be edited here"),
            );
        }
    }
    for (id, trigger) in &protocol.triggers {
        if let Trigger::Unknown(node) = trigger {
            out.push(
                Diagnostic::info(
                    format!("trigger {id}"),
                    format!("{} is not known to this version", node.short_type()),
                )
                .with_hint("it is preserved unchanged, but cannot be edited here"),
            );
        }
    }
}
