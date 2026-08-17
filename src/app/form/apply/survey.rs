// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Applying the survey-step and measure forms.

use carp_protocol::StudyProtocol;
use carp_protocol::survey::{KnownStep, RpStep};

use crate::app::form::Form;

use super::Applied;

/// Write a survey-step form back.
///
/// Renaming a step's identifier moves the navigation rules that jump to it,
/// which is the survey's equivalent of renaming a task: the identifier is
/// what branches point at, and a rule left behind sends the participant to a
/// step that no longer exists.
pub fn apply_step(
    protocol: &mut StudyProtocol,
    form: &Form,
    task_name: &str,
    index: usize,
) -> Applied {
    let identifier = form.text("identifier");
    if identifier.trim().is_empty() {
        return Applied::Refused("a step needs an identifier".to_owned());
    }

    let Some(survey) = protocol
        .tasks
        .iter_mut()
        .find(|task| task.name() == task_name)
        .and_then(carp_protocol::task::Task::survey_mut)
    else {
        return Applied::Vanished;
    };

    let previous = match survey.steps().get(index) {
        Some(step) => step.identifier().to_owned(),
        None => return Applied::Vanished,
    };

    // A duplicate identifier would make two steps record under one key, and
    // the later answer would overwrite the earlier.
    let taken = survey
        .all_step_identifiers()
        .into_iter()
        .filter(|existing| *existing != previous)
        .any(|existing| existing == identifier);
    if taken {
        return Applied::Refused(format!("another step already uses {identifier:?}"));
    }

    let Some(steps) = survey.steps_mut() else {
        return Applied::Vanished;
    };
    let Some(step) = steps.get_mut(index) else {
        return Applied::Vanished;
    };

    step.set_identifier(identifier.clone());
    step.set_title(form.text("title"));
    apply_kind_fields(step, form);

    // Follow the rename into the branches pointing at this step.
    if identifier != previous
        && let Some(rules) = survey.navigation_rules_mut()
    {
        if let Some(rule) = rules.remove(&previous) {
            rules.insert(identifier.clone(), rule);
        }
        for rule in rules.values_mut() {
            rule.rename_destination(&previous, &identifier);
        }
    }

    Applied::Changed
}

/// The fields particular to one step type.
fn apply_kind_fields(step: &mut RpStep, form: &Form) {
    let RpStep::Known(known) = step else {
        return;
    };
    match known.as_mut() {
        KnownStep::Instruction { text, optional, .. }
        | KnownStep::Completion { text, optional, .. } => {
            *text = form.text("text");
            *optional = form.flag("optional");
        }
        KnownStep::Question {
            optional,
            auto_skip,
            timeout,
            auto_focus,
            ..
        } => {
            *optional = form.flag("optional");
            *auto_skip = form.flag("auto_skip");
            if let Some(seconds) = form.integer("timeout") {
                *timeout = seconds as u32;
            }
            *auto_focus = form.flag("auto_focus");
        }
        KnownStep::Form { optional, .. } => *optional = form.flag("optional"),
        KnownStep::Tapping {
            optional,
            length_of_test,
            include_instructions,
            include_results,
            ..
        } => {
            *optional = form.flag("optional");
            apply_activity(form, length_of_test, include_instructions, include_results);
        }
        KnownStep::Flanker {
            optional,
            length_of_test,
            number_of_cards,
            include_instructions,
            include_results,
            ..
        } => {
            *optional = form.flag("optional");
            apply_activity(form, length_of_test, include_instructions, include_results);
            if let Some(cards) = form.integer("number_of_cards") {
                *number_of_cards = cards as u32;
            }
        }
        KnownStep::ReactionTime {
            optional,
            length_of_test,
            switch_interval,
            include_instructions,
            include_results,
            ..
        } => {
            *optional = form.flag("optional");
            apply_activity(form, length_of_test, include_instructions, include_results);
            if let Some(interval) = form.integer("switch_interval") {
                *switch_interval = interval as u32;
            }
        }
    }
}

/// The three rows every cognitive activity shares.
fn apply_activity(
    form: &Form,
    length_of_test: &mut u32,
    include_instructions: &mut bool,
    include_results: &mut bool,
) {
    if let Some(seconds) = form.integer("length_of_test") {
        *length_of_test = seconds as u32;
    }
    *include_instructions = form.flag("include_instructions");
    *include_results = form.flag("include_results");
}

/// Write a measure form back.
pub fn apply_measure(
    protocol: &mut StudyProtocol,
    form: &Form,
    task_name: &str,
    index: usize,
) -> Applied {
    let data_type = form.text("type");
    if data_type.trim().is_empty() {
        return Applied::Refused("a measure needs a data type".to_owned());
    }

    let Some(measures) = protocol
        .tasks
        .iter_mut()
        .find(|task| task.name() == task_name)
        .and_then(carp_protocol::task::Task::measures_mut)
    else {
        return Applied::Vanished;
    };
    let Some(measure) = measures.get_mut(index) else {
        return Applied::Vanished;
    };

    // Replacing rather than mutating keeps whatever sampling override the
    // measure carried, since `Measure::with_sampling` takes it back.
    let sampling = measure.sampling().cloned();
    *measure = match sampling {
        Some(sampling) => carp_protocol::Measure::with_sampling(data_type, sampling),
        None => carp_protocol::Measure::data_stream(data_type),
    };

    Applied::Changed
}
