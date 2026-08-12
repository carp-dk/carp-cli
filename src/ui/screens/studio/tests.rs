// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Rendering tests for the protocol editor.
//!
//! The editor has eight tabs, two overlays and two very different protocols to
//! show - an empty one and a full one - so the combinations add up fast. These
//! tests take the whole grid rather than a few examples, because a panic in a
//! renderer takes the terminal down with it and the cheapest place to find one
//! is here.

use ratatui::Terminal;
use ratatui::backend::TestBackend;

use super::*;
use crate::studio::{CatalogState, Section, Studio};
use carp_protocol::StudyProtocol;

/// The sizes the app supports, from generous to the documented minimum.
const SIZES: [(u16, u16); 4] = [(180, 50), (120, 32), (90, 24), (62, 14)];

fn blank() -> Studio {
    Studio::new("979b408d-784e-4b1b-bb1e-ff9204e072f3".to_owned())
}

/// A real production protocol, opened as the editor would open it, with a
/// catalogue behind it.
pub(super) fn loaded() -> Studio {
    let protocol: StudyProtocol = serde_json::from_str(include_str!(
        "../../../../packages/carp-protocol/tests/corpus/demo.json"
    ))
    .expect("the vendored corpus parses");

    let mut studio = Studio::opened(
        protocol,
        Some(std::path::PathBuf::from("demo/protocol.json")),
    );
    studio.catalog = catalog();
    studio.catalog_state = CatalogState::Ready;
    studio.survey_task = studio.survey_task_name();
    studio
}

/// A catalogue derived from part of the vendored corpus.
fn catalog() -> carp_catalog::Catalog {
    let snapshot = carp_catalog::Snapshot::new(
        "carp-dk/carp_study_app_configurations".to_owned(),
        carp_catalog::Commit {
            sha: "74f543e65bc18300c61a967cf6c3f13e228eabf9".to_owned(),
            date: "2026-08-11T12:38:39Z".to_owned(),
            subject: "Merge pull request #45".to_owned(),
        },
        vec![carp_catalog::ProtocolDocument {
            study: "demo".to_owned(),
            path: "demo/carp/resources/protocol.json".to_owned(),
            json: include_str!("../../../../packages/carp-protocol/tests/corpus/demo.json")
                .to_owned(),
        }],
    );
    carp_catalog::derive::catalog(&snapshot)
}

/// Draw `studio` at `size` and return the buffer as one string.
fn draw(studio: &mut Studio, size: (u16, u16)) -> String {
    let mut terminal = Terminal::new(TestBackend::new(size.0, size.1)).unwrap();
    terminal
        .draw(|frame| render(frame, frame.area(), studio))
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

/// Every tab must render, on an empty protocol and on a real one, at every
/// supported size.
#[test]
fn every_tab_renders_at_every_size() {
    for size in SIZES {
        for section in Section::ALL {
            for mut studio in [blank(), loaded()] {
                studio.section = section;
                draw(&mut studio, size);
            }
        }
    }
}

/// The overlays draw on top of whatever tab is beneath them, so they have to
/// be exercised against each one.
#[test]
fn the_overlays_render_over_every_tab() {
    for size in SIZES {
        for section in Section::ALL {
            let mut studio = loaded();
            studio.section = section;

            // A form on its own.
            studio.form = Some(crate::app::form::build::protocol(&studio.protocol));
            draw(&mut studio, size);

            // A picker on top of that form, which is the deepest the stack
            // goes.
            crate::studio::pickers::open_for_field(&mut studio);
            draw(&mut studio, size);

            // A picker with no form under it, as `a` opens.
            studio.form = None;
            studio.picker = None;
            crate::studio::pickers::open_add(&mut studio);
            draw(&mut studio, size);
        }
    }
}

/// The tab bar has to show which tab is current and how the checks stand.
#[test]
fn the_tab_bar_shows_the_check_summary() {
    let mut studio = blank();
    let rendered = draw(&mut studio, (120, 32));
    assert!(rendered.contains("Overview"), "{rendered}");
    assert!(
        rendered.contains("no findings"),
        "a blank protocol is sound"
    );

    // Removing the only primary device makes it unsound, which the bar says.
    carp_protocol::builder::remove_device(&mut studio.protocol, "Primary Phone");
    studio.changed();
    let rendered = draw(&mut studio, (120, 32));
    assert!(rendered.contains("1 error"), "{rendered}");
}

/// An empty tab must say what to do rather than showing an empty box.
#[test]
fn empty_tabs_say_what_to_do() {
    let expected = [
        (Section::Tasks, "press a to add one"),
        (Section::Participants, "press a to add one"),
        (Section::Survey, "no survey in this protocol"),
        (Section::Catalog, "press S to download"),
    ];

    for (section, message) in expected {
        let mut studio = blank();
        studio.section = section;
        let rendered = draw(&mut studio, (120, 32));
        assert!(
            rendered.contains(message),
            "{section:?} should say {message:?}, got:\n{rendered}"
        );
    }
}

/// The Catalog tab is the answer to "where do these options come from?", so
/// it has to name the commit.
#[test]
fn the_catalog_tab_names_its_version() {
    let mut studio = loaded();
    studio.section = Section::Catalog;
    let rendered = draw(&mut studio, (140, 40));

    assert!(rendered.contains("74f543e"), "the short commit: {rendered}");
    assert!(
        rendered.contains("carp_study_app_configurations"),
        "{rendered}"
    );
    assert!(rendered.contains("demo"), "the study it learned from");
}

/// The hint line changes with the overlay, since the keys do.
#[test]
fn the_hints_follow_the_overlay() {
    let mut studio = loaded();
    assert!(hints(&studio).contains("Esc leave"));

    studio.form = Some(crate::app::form::build::protocol(&studio.protocol));
    assert!(hints(&studio).contains("w save"));

    crate::studio::pickers::open_for_field(&mut studio);
    // The protocol form's first field is typed, not picked, so open a picker
    // explicitly to check the third state.
    studio.picker = Some(crate::app::form::picker::Picker::new(
        "measure types",
        crate::app::form::picker::PickerKind::Single,
        Vec::new(),
        "",
    ));
    assert!(hints(&studio).contains("type to filter"));
}

/// A protocol whose devices this build does not model must still render, and
/// say so rather than showing a blank panel.
#[test]
fn an_unmodelled_device_renders_with_an_explanation() {
    let mut studio = blank();
    studio
        .protocol
        .connected_devices
        .push(carp_protocol::Device::Unknown(carp_protocol::UnknownNode {
            type_name: "dk.carp.cams.devices.FutureSensor".to_owned(),
            fields: serde_json::Map::from_iter([(
                "roleName".to_owned(),
                serde_json::Value::String("Future Sensor".to_owned()),
            )]),
        }));
    studio.changed();
    studio.section = Section::Devices;
    studio.lists.devices.select(Some(1));

    let rendered = draw(&mut studio, (140, 40));
    assert!(rendered.contains("FutureSensor"), "{rendered}");
    assert!(rendered.contains("newer than this build"), "{rendered}");
}
