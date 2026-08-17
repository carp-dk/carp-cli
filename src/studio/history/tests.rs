// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

fn protocol(name: &str) -> StudyProtocol {
    StudyProtocol::new(name, "979b408d-784e-4b1b-bb1e-ff9204e072f3")
}

#[test]
fn states_come_back_in_reverse_order() {
    let mut history = History::default();
    history.push(protocol("first"));
    history.push(protocol("second"));

    assert_eq!(history.pop().unwrap().name, "second");
    assert_eq!(history.pop().unwrap().name, "first");
    assert!(history.pop().is_none());
}

/// An unbounded history would grow for as long as the editor is open.
#[test]
fn the_history_is_bounded() {
    let mut history = History::default();
    for index in 0..DEPTH + 10 {
        history.push(protocol(&format!("state {index}")));
    }

    assert_eq!(history.depth(), DEPTH);
    // The most recent states are the ones kept.
    assert_eq!(history.pop().unwrap().name, format!("state {}", DEPTH + 9));
}

#[test]
fn clearing_forgets_everything() {
    let mut history = History::default();
    history.push(protocol("first"));
    history.clear();

    assert!(history.is_empty());
    assert!(history.pop().is_none());
}
