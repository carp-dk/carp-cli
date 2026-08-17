// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Which row of each tab is selected.
//!
//! One cursor per list, kept across tab switches so returning to a tab lands
//! where it was left. [`Lists::sync`] pulls every cursor back inside its data
//! after a change, which is what stops a deletion leaving a detail panel
//! describing something that is no longer there.

use carp_protocol::StudyProtocol;
use ratatui::widgets::TableState;

use crate::app::state::{clamp_selection, move_selection};

use super::Section;

/// The cursor of every list in the editor.
#[derive(Debug, Default)]
pub struct Lists {
    pub devices: TableState,
    pub tasks: TableState,
    /// Measures of the selected task, shown beside it.
    pub measures: TableState,
    pub triggers: TableState,
    /// Steps of the survey being shown.
    pub steps: TableState,
    pub roles: TableState,
    /// Expected participant data, shown under the roles.
    pub expected: TableState,
    pub checks: TableState,
    /// Templates in the catalogue pane.
    pub templates: TableState,
}

impl Lists {
    /// Pull every cursor back inside its data.
    pub fn sync(&mut self, protocol: &StudyProtocol, survey_task: Option<&str>) {
        clamp_selection(&mut self.devices, protocol.devices().count());
        clamp_selection(&mut self.tasks, protocol.tasks.len());
        clamp_selection(&mut self.triggers, protocol.triggers.len());
        clamp_selection(&mut self.roles, protocol.participant_roles.len());
        clamp_selection(&mut self.expected, protocol.expected_participant_data.len());

        let measures = self
            .selected_task(protocol)
            .map_or(0, |task| task.measures().len());
        clamp_selection(&mut self.measures, measures);

        let steps = survey_task
            .and_then(|name| protocol.task(name))
            .and_then(carp_protocol::task::Task::survey)
            .map_or(0, |survey| survey.steps().len());
        clamp_selection(&mut self.steps, steps);
    }

    /// Move the cursor of `section`'s primary list.
    pub fn move_in(
        &mut self,
        section: Section,
        protocol: &StudyProtocol,
        survey_task: Option<&str>,
        checks: usize,
        templates: usize,
        delta: isize,
    ) {
        match section {
            Section::Overview => {}
            Section::Devices => {
                move_selection(&mut self.devices, protocol.devices().count(), delta);
            }
            Section::Tasks => {
                move_selection(&mut self.tasks, protocol.tasks.len(), delta);
                // The measures pane describes the selected task, so its
                // cursor belongs to the newly selected one.
                let measures = self
                    .selected_task(protocol)
                    .map_or(0, |task| task.measures().len());
                self.measures.select((measures > 0).then_some(0));
            }
            Section::Triggers => {
                move_selection(&mut self.triggers, protocol.triggers.len(), delta);
            }
            Section::Survey => {
                let steps = survey_task
                    .and_then(|name| protocol.task(name))
                    .and_then(carp_protocol::task::Task::survey)
                    .map_or(0, |survey| survey.steps().len());
                move_selection(&mut self.steps, steps, delta);
            }
            Section::Participants => {
                move_selection(&mut self.roles, protocol.participant_roles.len(), delta);
            }
            Section::Catalog => move_selection(&mut self.templates, templates, delta),
            Section::Checks => move_selection(&mut self.checks, checks, delta),
        }
    }

    /// The device under the cursor. Primary devices come first, matching how
    /// the list is rendered.
    pub fn selected_device<'a>(
        &self,
        protocol: &'a StudyProtocol,
    ) -> Option<&'a carp_protocol::Device> {
        protocol.devices().nth(self.devices.selected()?)
    }

    pub fn selected_device_role(&self, protocol: &StudyProtocol) -> Option<String> {
        self.selected_device(protocol)
            .map(|device| device.role_name().to_owned())
    }

    pub fn selected_task<'a>(
        &self,
        protocol: &'a StudyProtocol,
    ) -> Option<&'a carp_protocol::Task> {
        protocol.tasks.get(self.tasks.selected()?)
    }

    pub fn selected_task_name(&self, protocol: &StudyProtocol) -> Option<String> {
        self.selected_task(protocol)
            .map(|task| task.name().to_owned())
    }

    /// The trigger under the cursor, with its id.
    ///
    /// Triggers are a map keyed by id, and the list shows them in id order,
    /// so the cursor indexes into that ordering rather than into a `Vec`.
    pub fn selected_trigger<'a>(
        &self,
        protocol: &'a StudyProtocol,
    ) -> Option<(u32, &'a carp_protocol::Trigger)> {
        protocol
            .triggers
            .iter()
            .nth(self.triggers.selected()?)
            .map(|(id, trigger)| (*id, trigger))
    }

    pub fn selected_trigger_id(&self, protocol: &StudyProtocol) -> Option<u32> {
        self.selected_trigger(protocol).map(|(id, _)| id)
    }

    pub fn selected_role<'a>(
        &self,
        protocol: &'a StudyProtocol,
    ) -> Option<&'a carp_protocol::ParticipantRole> {
        protocol.participant_roles.get(self.roles.selected()?)
    }
}

#[cfg(test)]
mod tests;
