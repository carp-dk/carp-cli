// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Everything shown for one study, and for one participant of it.

use std::collections::HashMap;

use ratatui::widgets::TableState;

use carp_client::api::models::{
    Account, Export, ParticipantGroup, ParticipantGroupStatus, ParticipantSummary, StudyFile,
    StudyOverview,
};

use super::{ParticipantsState, StudyTab, clamp_selection, short};

/// Everything shown for one study.
#[derive(Debug)]
pub struct StudyState {
    pub study: StudyOverview,
    pub tab: StudyTab,
    pub researchers: Vec<Account>,
    pub assistants: Vec<Account>,
    pub staff_table: TableState,
    /// Private so it can only be replaced through [`StudyState::set_groups`],
    /// which keeps `group_by_participant` in step with it.
    groups: ParticipantGroupStatus,
    pub groups_table: TableState,
    /// participant id -> index into `groups.groups`.
    group_by_participant: HashMap<String, usize>,
    pub participants: ParticipantsState,
    pub files: Vec<StudyFile>,
    pub files_table: TableState,
    pub files_loading: bool,
    pub files_loaded: bool,
    pub exports: Vec<Export>,
    pub exports_table: TableState,
    pub exports_loading: bool,
    pub exports_loaded: bool,
    /// Staff and participant groups arrive from one request.
    pub details_loading: bool,
    pub details_loaded: bool,
}

impl StudyState {
    pub fn new(study: StudyOverview) -> Self {
        Self {
            study,
            tab: StudyTab::Overview,
            researchers: Vec::new(),
            assistants: Vec::new(),
            staff_table: TableState::default(),
            groups: ParticipantGroupStatus::default(),
            groups_table: TableState::default(),
            group_by_participant: HashMap::new(),
            participants: ParticipantsState::default(),
            files: Vec::new(),
            files_table: TableState::default(),
            files_loading: false,
            files_loaded: false,
            exports: Vec::new(),
            exports_table: TableState::default(),
            exports_loading: false,
            exports_loaded: false,
            details_loading: false,
            details_loaded: false,
        }
    }

    pub fn id(&self) -> String {
        self.study.study_id.to_string()
    }

    /// Researchers followed by assistants, as shown in the staff table.
    pub fn staff(&self) -> Vec<(&Account, &'static str)> {
        self.researchers
            .iter()
            .map(|account| (account, "researcher"))
            .chain(
                self.assistants
                    .iter()
                    .map(|account| (account, "research assistant")),
            )
            .collect()
    }

    pub fn selected_file(&self) -> Option<&StudyFile> {
        self.files.get(self.files_table.selected()?)
    }

    pub fn selected_export(&self) -> Option<&Export> {
        self.exports.get(self.exports_table.selected()?)
    }

    /// Store the participant groups and index them by member, so both tabs
    /// can answer "which deployment is this participant's?".
    pub fn set_groups(&mut self, groups: ParticipantGroupStatus) {
        self.group_by_participant = groups.index_by_participant();
        self.groups = groups;
        clamp_selection(&mut self.groups_table, self.groups.groups.len());
    }

    pub fn groups(&self) -> &ParticipantGroupStatus {
        &self.groups
    }

    /// The deployment collecting this participant's data.
    pub fn group_for(&self, participant_id: &str) -> Option<&ParticipantGroup> {
        let position = *self.group_by_participant.get(participant_id)?;
        self.groups.groups.get(position)
    }

    /// Names of a group's members, resolved through whatever participant
    /// pages have been loaded.
    pub fn group_members(&self, group: &ParticipantGroup) -> Vec<String> {
        group
            .participant_ids()
            .map(|id| {
                self.participants
                    .lookup(id)
                    .map_or_else(|| short(id).to_owned(), ParticipantSummary::display_name)
            })
            .collect()
    }

    /// Keep every table's cursor inside its data.
    ///
    /// Called before rendering: a list that has just gained rows must have a
    /// selected row, otherwise its detail panel would sit empty next to a
    /// full table.
    pub fn sync_selection(&mut self) {
        let staff = self.staff().len();
        clamp_selection(&mut self.staff_table, staff);
        clamp_selection(&mut self.groups_table, self.groups.groups.len());
        let participants = self.participants.items.len();
        clamp_selection(&mut self.participants.table, participants);
        clamp_selection(&mut self.files_table, self.files.len());
        clamp_selection(&mut self.exports_table, self.exports.len());
    }

    pub fn selected_group(&self) -> Option<&carp_client::api::models::ParticipantGroup> {
        self.groups.groups.get(self.groups_table.selected()?)
    }

    pub fn selected_staff(&self) -> Option<(&Account, &'static str)> {
        self.staff().get(self.staff_table.selected()?).copied()
    }
}

/// One participant, opened from the participants tab.
#[derive(Debug)]
pub struct ParticipantState {
    pub study: StudyOverview,
    pub participant: ParticipantSummary,
    /// The deployment this participant belongs to, resolved when the screen
    /// was opened.
    pub group: Option<ParticipantGroup>,
}
