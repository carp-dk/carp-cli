// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! A visual dump of every screen, for eyeballing the layout.
//!
//! Not an assertion: it prints what each screen renders so a change to the
//! spacing or the colours can be looked at rather than guessed at. Ignored by
//! default, since it produces pages of output.
//!
//! Run with `cargo test dump -- --ignored --nocapture`.

use ratatui::Terminal;
use ratatui::backend::TestBackend;

use super::tests::*;

#[test]
#[ignore = "visual check: cargo test dump -- --ignored --nocapture"]
fn print_screens() {
crate::ui::icons::use_set(
    std::env::var("CARP_ICONS")
        .ok()
        .and_then(|value| crate::ui::icons::IconSet::parse(&value))
        .unwrap_or_default(),
);
for label in [
    "help",
    "participant",
    "studies",
    "overview",
    "participants",
    "deployments",
    "staff",
    "files",
    "exports",
] {
    let mut terminal = Terminal::new(TestBackend::new(110, 26)).unwrap();
    let mut app = screen(label);
    terminal.draw(|frame| super::draw(frame, &mut app)).unwrap();
    println!("\n===== {label} =====");
    for row in 0..terminal.backend().buffer().area.height {
        let mut line = String::new();
        for column in 0..terminal.backend().buffer().area.width {
            line.push_str(terminal.backend().buffer()[(column, row)].symbol());
        }
        println!("{}", line.trim_end());
    }
}
}
