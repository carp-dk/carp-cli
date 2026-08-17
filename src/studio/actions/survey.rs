// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Actions on the open survey: reordering, adding and opening its steps.

use carp_protocol::survey::RpStep;
use carp_protocol::task::Task;

use crate::studio::{Section, Studio};

use super::edit_selected;

/// Move the selected survey step by `delta` places.
///
/// A survey is an ordered list and the order is the experience, so reordering
/// is a first-class operation rather than a matter of deleting and re-adding.
pub fn move_step(studio: &mut Studio, delta: isize) -> Option<String> {
    let task = studio.survey_task_name()?;
    let index = studio.lists.steps.selected()?;

    let steps = studio
        .protocol
        .tasks
        .iter_mut()
        .find(|candidate| candidate.name() == task)
        .and_then(Task::survey_mut)
        .and_then(carp_protocol::survey::RpTask::steps_mut)?;

    let target = (index as isize).saturating_add(delta);
    if target < 0 || target as usize >= steps.len() {
        return None;
    }
    let target = target as usize;

    studio.checkpoint();
    let steps = studio
        .protocol
        .tasks
        .iter_mut()
        .find(|candidate| candidate.name() == task)
        .and_then(Task::survey_mut)
        .and_then(carp_protocol::survey::RpTask::steps_mut)?;
    steps.swap(index, target);

    studio.lists.steps.select(Some(target));
    studio.changed();
    None
}

/// Open the Survey tab on the selected task, if it has a survey.
pub fn open_survey(studio: &mut Studio) -> Option<String> {
    let task = studio.lists.selected_task_name(&studio.protocol)?;
    if studio.protocol.task(&task)?.survey().is_none() {
        return Some(format!("{task} is not a survey task"));
    }

    studio.survey_task = Some(task);
    studio.section = Section::Survey;
    studio.lists.steps.select(Some(0));
    studio.sync_selection();
    None
}

/// Append a step of `kind` to the open survey and edit it.
pub fn add_step(studio: &mut Studio, step: RpStep) -> Option<String> {
    let task = studio.survey_task_name()?;

    studio.checkpoint();
    let index = studio
        .protocol
        .tasks
        .iter_mut()
        .find(|candidate| candidate.name() == task)
        .and_then(Task::survey_mut)
        .and_then(carp_protocol::survey::RpTask::steps_mut)
        .map(|steps| {
            steps.push(step);
            steps.len() - 1
        })?;

    studio.lists.steps.select(Some(index));
    studio.changed();
    edit_selected(studio)
}
