// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

/// Errors have to sort before warnings, which sort before info: the
/// editor shows the first few findings and they must be the worst ones.
#[test]
fn severities_order_worst_first() {
    let mut severities = [Severity::Info, Severity::Error, Severity::Warning];
    severities.sort_unstable();
    assert_eq!(
        severities,
        [Severity::Error, Severity::Warning, Severity::Info]
    );
}

#[test]
fn counts_split_by_severity() {
    let diagnostics = vec![
        Diagnostic::error("a", "one"),
        Diagnostic::error("b", "two"),
        Diagnostic::warning("c", "three"),
    ];
    assert_eq!(counts(&diagnostics), (2, 1, 0));
}

#[test]
fn a_diagnostic_reads_as_a_sentence() {
    let diagnostic = Diagnostic::error("task \"Sleep\"", "has no measures")
        .with_hint("add at least one measure");
    assert_eq!(
        diagnostic.to_string(),
        "error: task \"Sleep\": has no measures (add at least one measure)"
    );
}
