// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

#[test]
fn tabs_cycle_in_both_directions() {
    assert_eq!(Section::Overview.previous(), Section::Checks);
    assert_eq!(Section::Checks.next(), Section::Overview);

    let mut section = Section::Overview;
    for _ in 0..Section::ALL.len() {
        section = section.next();
    }
    assert_eq!(section, Section::Overview);
}

/// The number keys select tabs, so the mapping has to cover every one.
#[test]
fn every_tab_is_reachable_by_index() {
    for (index, section) in Section::ALL.into_iter().enumerate() {
        assert_eq!(Section::from_index(index), Some(section));
        assert_eq!(section.index(), index);
    }
    assert_eq!(Section::from_index(Section::ALL.len()), None);
}

/// Every tab has to say what its keys do, or a section is unusable
/// without reading the source.
#[test]
fn every_tab_documents_its_keys() {
    for section in Section::ALL {
        assert!(!section.title().is_empty());
        assert!(!section.hints().is_empty(), "{:?}", section);
    }
}
