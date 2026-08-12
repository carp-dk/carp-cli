// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The overlay for choosing a value from a list.
//!
//! Opened on any field whose value comes from somewhere other than the
//! keyboard: a catalogue list, a fixed set of options, or the protocol's own
//! devices and tasks. It filters as you type, which matters because the
//! measure-type list runs to several dozen entries with a shared prefix.
//!
//! A picker over a catalogue accepts a value that is not in the list. Some
//! study has to be the first to use a new measure type, and refusing would
//! make this tool the reason it cannot be. The interface says when a value is
//! unfamiliar rather than preventing it.

/// One row of a picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// What is stored when this row is chosen.
    pub value: String,
    /// What is shown as the row's main text.
    pub label: String,
    /// Secondary text: how widely a catalogue entry is used, or what an
    /// option means.
    pub detail: String,
}

impl Row {
    pub fn new(value: impl Into<String>, label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            detail: detail.into(),
        }
    }
}

/// What the picker's result is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKind {
    /// One value, replacing the field's contents.
    Single,
    /// Several values, toggled with space and confirmed with enter.
    Multiple,
    /// One value, but the picker is choosing a *new* thing to add rather
    /// than editing an existing field - a device class, a task kind.
    Create,
}

/// An open picker overlay.
#[derive(Debug, Clone)]
pub struct Picker {
    /// What the overlay calls itself.
    pub title: String,
    pub kind: PickerKind,
    /// Every row, before filtering.
    pub rows: Vec<Row>,
    /// Indices into `rows` that pass the filter.
    pub visible: Vec<usize>,
    pub filter: String,
    /// Index into `visible`.
    pub selected: usize,
    /// Chosen values, for a [`PickerKind::Multiple`] picker.
    pub chosen: Vec<String>,
    /// Whether typing a value not in the list is allowed.
    pub accepts_free_text: bool,
}

impl Picker {
    /// A picker over `rows`, with `current` selected if it is among them.
    pub fn new(title: impl Into<String>, kind: PickerKind, rows: Vec<Row>, current: &str) -> Self {
        let mut picker = Self {
            title: title.into(),
            kind,
            rows,
            visible: Vec::new(),
            filter: String::new(),
            selected: 0,
            chosen: Vec::new(),
            accepts_free_text: false,
        };
        picker.refilter();
        if let Some(position) = picker
            .visible
            .iter()
            .position(|index| picker.rows[*index].value == current)
        {
            picker.selected = position;
        }
        picker
    }

    /// A multi-select picker starting from `chosen`.
    pub fn multiple(title: impl Into<String>, rows: Vec<Row>, chosen: Vec<String>) -> Self {
        let mut picker = Self::new(title, PickerKind::Multiple, rows, "");
        picker.chosen = chosen;
        picker
    }

    /// Allow a value that is not among the rows, typed into the filter box.
    pub fn allowing_free_text(mut self) -> Self {
        self.accepts_free_text = true;
        self
    }

    /// Recompute the visible rows for the current filter.
    pub fn refilter(&mut self) {
        let needle = self.filter.trim().to_lowercase();
        self.visible = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                needle.is_empty()
                    || row.value.to_lowercase().contains(&needle)
                    || row.label.to_lowercase().contains(&needle)
            })
            .map(|(index, _)| index)
            .collect();
        self.selected = self.selected.min(self.visible.len().saturating_sub(1));
    }

    pub fn push(&mut self, character: char) {
        self.filter.push(character);
        self.refilter();
    }

    pub fn backspace(&mut self) {
        self.filter.pop();
        self.refilter();
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.refilter();
    }

    /// Move the cursor by `delta`, stopping at the ends.
    pub fn move_selection(&mut self, delta: isize) {
        if self.visible.is_empty() {
            self.selected = 0;
            return;
        }
        let last = self.visible.len() as isize - 1;
        let next = (self.selected as isize).saturating_add(delta).clamp(0, last);
        self.selected = next as usize;
    }

    /// The row under the cursor.
    pub fn selected_row(&self) -> Option<&Row> {
        self.rows.get(*self.visible.get(self.selected)?)
    }

    /// Add or remove the selected row from the chosen set.
    pub fn toggle_selected(&mut self) {
        let Some(value) = self.selected_row().map(|row| row.value.clone()) else {
            return;
        };
        match self.chosen.iter().position(|chosen| *chosen == value) {
            Some(position) => {
                self.chosen.remove(position);
            }
            None => self.chosen.push(value),
        }
    }

    pub fn is_chosen(&self, value: &str) -> bool {
        self.chosen.iter().any(|chosen| chosen == value)
    }

    /// The value the picker settles on.
    ///
    /// The highlighted row, or - when free text is allowed and nothing
    /// matches - whatever was typed into the filter. That is what lets a new
    /// measure type be entered in the same place the known ones are chosen.
    pub fn resolve(&self) -> Option<String> {
        match self.selected_row() {
            Some(row) => Some(row.value.clone()),
            None if self.accepts_free_text && !self.filter.trim().is_empty() => {
                Some(self.filter.trim().to_owned())
            }
            None => None,
        }
    }

    /// Rows built from a catalogue list.
    pub fn rows_from_catalog(entries: &[carp_catalog::CatalogEntry]) -> Vec<Row> {
        entries
            .iter()
            .map(|entry| Row::new(&entry.value, entry.short_value(), entry.usage()))
            .collect()
    }
}

#[cfg(test)]
mod tests;
