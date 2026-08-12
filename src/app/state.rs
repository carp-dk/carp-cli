//! Screen state. Rendering reads these structs and never mutates anything
//! except the selection state ratatui needs.


use ratatui::widgets::TableState;


pub mod prompt;
pub mod route;
pub mod status;
pub mod studies;
pub mod study;

pub use prompt::{Prompt, PromptKind, StudySort};
pub use route::{Route, StudyTab};
pub use status::{Status, StatusKind};
pub use studies::{ParticipantsState, StudiesState};
pub use study::{ParticipantState, StudyState};

/// First segment of an id, for when a name cannot be resolved.
pub fn short(id: &str) -> &str {
    id.split('-').next().unwrap_or(id)
}

/// Keep a table's selection inside `len`, selecting the first row when the
/// table gains its first entries.
pub fn clamp_selection(table: &mut TableState, len: usize) {
    if len == 0 {
        table.select(None);
        return;
    }
    match table.selected() {
        Some(selected) if selected >= len => table.select(Some(len - 1)),
        Some(_) => {}
        None => table.select(Some(0)),
    }
}

/// Move a table cursor by `delta` rows, stopping at the ends.
pub fn move_selection(table: &mut TableState, len: usize, delta: isize) {
    if len == 0 {
        table.select(None);
        return;
    }
    // `g`/`G` pass extreme deltas to jump to an end, so saturate.
    let current = table.selected().unwrap_or(0) as isize;
    let next = current.saturating_add(delta).clamp(0, len as isize - 1);
    table.select(Some(next as usize));
}

#[cfg(test)]
mod tests;
