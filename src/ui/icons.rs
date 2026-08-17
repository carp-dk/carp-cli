// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Icons for the interface, in three sets.
//!
//! Emoji occupy two terminal cells and terminals disagree about exactly how
//! wide some of them are, which shows up as drifting table columns and broken
//! borders. The default set therefore uses single-width symbols, which every
//! terminal measures the same way, and emoji are opt-in through
//! `CARP_ICONS=emoji`. Only single-codepoint emoji are used - no variation
//! selectors or ZWJ sequences, which are the ones that misreport their width.
//!
//! Set the style once at startup with [`use_set`]; rendering reads it.

use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum IconSet {
    /// Single-width geometric symbols. Safe everywhere.
    #[default]
    Symbols,
    /// Double-width colour emoji.
    Emoji,
    /// No icons at all.
    None,
}

impl IconSet {
    /// Parse `CARP_ICONS` / `--icons`.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "symbols" | "symbol" | "unicode" => Some(Self::Symbols),
            "emoji" | "emojis" => Some(Self::Emoji),
            "none" | "off" | "plain" => Some(Self::None),
            _ => None,
        }
    }

    /// Cells an icon occupies, including the space after it. Fixed-width
    /// table columns add this so an icon never eats the text beside it.
    pub const fn cell_width(self) -> u16 {
        match self {
            Self::Symbols => 2,
            Self::Emoji => 3,
            Self::None => 0,
        }
    }
}

static ICONS: OnceLock<IconSet> = OnceLock::new();

/// Choose the set for this run. The first call wins.
pub fn use_set(set: IconSet) {
    let _ = ICONS.set(set);
}

pub fn set() -> IconSet {
    *ICONS.get().unwrap_or(&IconSet::Symbols)
}

/// Width to add to a fixed-width column that carries an icon.
pub fn cell_width() -> u16 {
    set().cell_width()
}

/// Join an icon and its text, dropping the gap when icons are off.
pub fn with(icon: &str, text: impl AsRef<str>) -> String {
    let text = text.as_ref();
    if icon.is_empty() {
        text.to_owned()
    } else {
        format!("{icon} {text}")
    }
}

macro_rules! icon {
    ($(#[$doc:meta])* $name:ident, $symbol:expr, $emoji:expr) => {
        $(#[$doc])*
        pub fn $name() -> &'static str {
            match set() {
                IconSet::Symbols => $symbol,
                IconSet::Emoji => $emoji,
                IconSet::None => "",
            }
        }
    };
}

icon!(/// The app itself, on the header badge.
    app, "◈", "🔬");
icon!(/// Study list and study screen.
    study, "▦", "📋");
icon!(/// Participants.
    participants, "◍", "👥");
icon!(/// Deployments and participant groups.
    deployments, "▣", "📱");
icon!(/// Researchers and assistants.
    staff, "✦", "🎓");
icon!(/// Uploaded study files.
    files, "▤", "📁");
icon!(/// Data exports.
    exports, "⇩", "📦");
icon!(/// Transfers.
    downloads, "⇓", "📥");
icon!(/// Keys and help.
    help, "?", "❓");

icon!(/// Finished, available, registered.
    ok, "✓", "✅");
icon!(/// Failed.
    error, "✗", "❌");
icon!(/// Working on it.
    pending, "⟳", "⏳");
icon!(/// Waiting on someone else.
    waiting, "◌", "📨");
icon!(/// Running.
    running, "▶", "🟢");
icon!(/// Finished with, no longer live.
    stopped, "■", "🔴");
icon!(/// Not set, not started, unknown.
    idle, "○", "⚪");
icon!(/// Ready but not yet live.
    partial, "◐", "🟡");
icon!(/// Something optional.
    optional, "◦", "➖");

/// Stage of a study, as reported by [`carp_client::api::models::StudyOverview`].
pub fn study_stage(stage: &str) -> &'static str {
    match stage {
        "live" => running(),
        "configured" => partial(),
        _ => idle(),
    }
}

/// Lifecycle of a participant group's deployment.
pub fn deployment_state(state: &str) -> &'static str {
    match state {
        "Running" | "DeviceDeployed" => running(),
        "Invited" => waiting(),
        "DeployingDevices" => pending(),
        "Stopped" => stopped(),
        _ => idle(),
    }
}

/// One device inside a deployment.
pub fn device(registered: bool, is_optional: bool) -> &'static str {
    if registered {
        ok()
    } else if is_optional {
        optional()
    } else {
        waiting()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_symbol_is_one_cell_wide() {
        // Symbols must not disturb column widths, which is the whole reason
        // they are the default.
        let symbols = [
            "◈", "▦", "◍", "▣", "✦", "▤", "⇩", "⇓", "?", "✓", "✗", "⟳", "◌", "▶", "■", "○", "◐",
            "◦",
        ];
        for symbol in symbols {
            let width: usize = symbol.chars().map(|c| c.len_utf8().min(1)).sum();
            assert_eq!(width, 1, "{symbol} should be a single character");
        }
    }

    #[test]
    fn the_set_is_selectable_by_name() {
        assert_eq!(IconSet::parse("emoji"), Some(IconSet::Emoji));
        assert_eq!(IconSet::parse(" NONE "), Some(IconSet::None));
        assert_eq!(IconSet::parse("symbols"), Some(IconSet::Symbols));
        assert_eq!(IconSet::parse("sparkles"), None);
    }

    #[test]
    fn text_is_untouched_when_icons_are_off() {
        assert_eq!(with("", "live"), "live");
        assert_eq!(with("●", "live"), "● live");
    }
}
