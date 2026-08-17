// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! A visual dump of the protocol editor, for eyeballing the layout.
//!
//! Not an assertion: [`super::tests`] covers whether the screens render, and
//! this prints what they render so the spacing, the column widths and the
//! colours can be looked at rather than guessed at.
//!
//! Run with `cargo test studio_screens -- --ignored --nocapture`.

use ratatui::Terminal;
use ratatui::backend::TestBackend;

use crate::studio::Section;

#[test]
#[ignore = "visual check: cargo test studio_screens -- --ignored --nocapture"]
fn studio_screens() {
    for section in Section::ALL {
        let mut studio = super::tests::loaded();
        studio.section = section;

        let mut terminal = Terminal::new(TestBackend::new(110, 26)).unwrap();
        terminal
            .draw(|frame| super::render(frame, frame.area(), &mut studio))
            .unwrap();

        println!("\n===== {} =====", section.title());
        print(&terminal);
    }

    // The two overlays, over the tab each is most often opened from.
    let mut studio = super::tests::loaded();
    studio.section = Section::Devices;
    studio.form = Some(crate::app::form::build::device(
        studio.protocol.devices().next().unwrap(),
    ));
    let mut terminal = Terminal::new(TestBackend::new(110, 26)).unwrap();
    terminal
        .draw(|frame| super::render(frame, frame.area(), &mut studio))
        .unwrap();
    println!("\n===== form =====");
    print(&terminal);

    let mut studio = super::tests::loaded();
    studio.section = Section::Tasks;
    crate::studio::pickers::open_add(&mut studio);
    let mut terminal = Terminal::new(TestBackend::new(110, 26)).unwrap();
    terminal
        .draw(|frame| super::render(frame, frame.area(), &mut studio))
        .unwrap();
    println!("\n===== picker =====");
    print(&terminal);
}

fn print(terminal: &Terminal<TestBackend>) {
    let buffer = terminal.backend().buffer();
    for row in 0..buffer.area.height {
        let mut line = String::new();
        for column in 0..buffer.area.width {
            line.push_str(buffer[(column, row)].symbol());
        }
        println!("{}", line.trim_end());
    }
}
