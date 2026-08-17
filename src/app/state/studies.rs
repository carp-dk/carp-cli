// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! State of the study list and of a study's participants tab.

use std::collections::HashMap;

use ratatui::widgets::TableState;

use carp_client::api::models::{ParticipantQuery, ParticipantSummary, StudyOverview};

use super::{StudySort, clamp_selection};

/// The study list.
#[derive(Debug, Default)]
pub struct StudiesState {
    pub items: Vec<StudyOverview>,
    /// Indices into `items` that pass the current filter.
    pub visible: Vec<usize>,
    pub table: TableState,
    pub filter: String,
    pub sort: StudySort,
    pub loading: bool,
    /// True while the list shown is the cached one.
    pub from_cache: bool,
}

impl StudiesState {
    pub fn set_items(&mut self, items: Vec<StudyOverview>, from_cache: bool) {
        let previous = self.selected().map(|study| study.study_id.clone());
        self.items = items;
        self.from_cache = from_cache;
        self.refilter();
        // Keep the cursor on the same study across refreshes.
        if let Some(previous) = previous
            && let Some(position) = self
                .visible
                .iter()
                .position(|index| self.items[*index].study_id == previous)
        {
            self.table.select(Some(position));
        }
    }

    pub fn refilter(&mut self) {
        let filter = self.filter.trim();
        self.visible = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, study)| filter.is_empty() || study.matches(filter))
            .map(|(index, _)| index)
            .collect();

        let items = &self.items;
        let sort = self.sort;
        self.visible.sort_by(|left, right| {
            let (left, right) = (&items[*left], &items[*right]);
            match sort {
                StudySort::Name => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
                // Newest first; studies without a date sort last.
                StudySort::Created => right.created_on.cmp(&left.created_on),
                StudySort::Stage => left
                    .stage()
                    .cmp(right.stage())
                    .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase())),
            }
        });

        clamp_selection(&mut self.table, self.visible.len());
    }

    pub fn selected(&self) -> Option<&StudyOverview> {
        let position = self.table.selected()?;
        self.items.get(*self.visible.get(position)?)
    }

    pub fn len(&self) -> usize {
        self.visible.len()
    }
}

/// The participants tab of a study.
#[derive(Debug, Default)]
pub struct ParticipantsState {
    pub items: Vec<ParticipantSummary>,
    pub table: TableState,
    pub query: ParticipantQuery,
    pub total: u32,
    pub loading: bool,
    pub from_cache: bool,
    pub loaded: bool,
    /// Everyone seen so far, across pages, so a deployment can name its
    /// members even when they are not on the page being shown.
    pub directory: HashMap<String, ParticipantSummary>,
}

impl ParticipantsState {
    pub fn selected(&self) -> Option<&ParticipantSummary> {
        self.items.get(self.table.selected()?)
    }

    pub fn lookup(&self, participant_id: &str) -> Option<&ParticipantSummary> {
        self.directory.get(participant_id)
    }

    pub fn page_count(&self) -> u32 {
        let size = self.query.size.max(1);
        self.total.div_ceil(size).max(1)
    }

    pub fn set_items(&mut self, items: Vec<ParticipantSummary>, total: u32, from_cache: bool) {
        for participant in &items {
            self.directory
                .insert(participant.participant_id.clone(), participant.clone());
        }
        self.items = items;
        self.total = total;
        self.from_cache = from_cache;
        self.loaded = true;
        clamp_selection(&mut self.table, self.items.len());
    }
}
