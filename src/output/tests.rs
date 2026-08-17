// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::table;

fn rendered(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut buffer = Vec::new();
    table::write(&mut buffer, headers, rows).unwrap();
    String::from_utf8(buffer).unwrap()
}

fn csv(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut buffer = Vec::new();
    table::write_csv(&mut buffer, headers, rows).unwrap();
    String::from_utf8(buffer).unwrap()
}

fn row(cells: &[&str]) -> Vec<String> {
    cells.iter().map(|cell| (*cell).to_owned()).collect()
}

#[test]
fn columns_line_up_under_their_headings() {
    let out = rendered(
        &["id", "name", "stage"],
        &[
            row(&["7f3a", "Sleep and mood", "running"]),
            row(&["c001", "A much longer study name", "draft"]),
        ],
    );

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "ID    NAME                      STAGE");
    assert_eq!(lines[1], "7f3a  Sleep and mood            running");
    assert_eq!(lines[2], "c001  A much longer study name  draft");

    // Every row starts each column at the same offset.
    for line in &lines[1..] {
        assert_eq!(line.find("  ").unwrap(), 4, "{line}");
    }
}

/// Trailing spaces are invisible on screen and a nuisance everywhere the
/// output is pasted, so the last column is not padded.
#[test]
fn no_row_ends_in_whitespace() {
    let out = rendered(
        &["id", "note"],
        &[row(&["a", "short"]), row(&["b", "a much longer note"])],
    );
    for line in out.lines() {
        assert_eq!(line, line.trim_end(), "{line:?} ends in whitespace");
    }
}

/// An empty result is a fact, not silence: a caller who typed the command
/// should see that it ran.
#[test]
fn an_empty_result_says_so() {
    assert_eq!(rendered(&["id", "name"], &[]), "no results\n");
}

/// Padding counts display columns, so a name in a double-width script cannot
/// push the columns after it out of line.
#[test]
fn alignment_counts_display_width_not_characters() {
    let out = rendered(
        &["name", "stage"],
        &[row(&["東京大学", "running"]), row(&["Aarhus", "draft"])],
    );
    let lines: Vec<&str> = out.lines().collect();

    // "東京大学" is 4 characters but 8 columns wide; "Aarhus" is 6 of each.
    // Both rows must therefore start their stage at the same screen column —
    // which counting characters, rather than width, would get wrong.
    let stage_column = |line: &str, stage: &str| {
        let start = line.find(stage).unwrap_or_else(|| panic!("{line:?}"));
        unicode_width::UnicodeWidthStr::width(&line[..start])
    };
    assert_eq!(
        stage_column(lines[1], "running"),
        stage_column(lines[2], "draft"),
        "{lines:?}"
    );
}

/// A description can run to hundreds of characters. One of them must not push
/// every other column off the screen.
#[test]
fn an_overlong_cell_is_cut_short_visibly() {
    let long = "x".repeat(200);
    let out = rendered(&["id", "about"], &[row(&["a", &long])]);
    let about = out.lines().nth(1).unwrap();

    assert!(about.ends_with('…'), "{about}");
    assert!(
        unicode_width::UnicodeWidthStr::width(about) <= 4 + 60,
        "cut to {} columns",
        unicode_width::UnicodeWidthStr::width(about)
    );
}

/// A newline inside a cell would end the row wherever it fell, silently
/// splitting one record into two.
#[test]
fn a_newline_inside_a_cell_cannot_end_the_row() {
    let out = rendered(&["id", "note"], &[row(&["a", "first\nsecond"])]);
    assert_eq!(out.lines().count(), 2, "{out:?}");
    assert!(out.contains("first second"), "{out:?}");
}

/// A comma in a study name is ordinary. Getting the quoting wrong would shift
/// every column after it, which is the kind of error that survives into an
/// analysis unnoticed.
#[test]
fn csv_quotes_whatever_would_otherwise_end_a_field() {
    let out = csv(
        &["id", "name"],
        &[
            row(&["a", "plain"]),
            row(&["b", "Sleep, mood and cognition"]),
            row(&["c", "The \"pilot\" study"]),
            row(&["d", "two\nlines"]),
        ],
    );

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "id,name");
    assert_eq!(lines[1], "a,plain");
    assert_eq!(lines[2], "b,\"Sleep, mood and cognition\"");
    assert_eq!(lines[3], "c,\"The \"\"pilot\"\" study\"");
    // The embedded newline stays inside the quoted field.
    assert_eq!(lines[4], "d,\"two");
    assert_eq!(lines[5], "lines\"");
}

/// CSV is the machine-readable one of the two column formats, so it must not
/// truncate: a value cut to 60 characters would be wrong data, not a display
/// choice.
#[test]
fn csv_does_not_truncate() {
    let long = "x".repeat(200);
    let out = csv(&["about"], &[row(&[&long])]);
    assert!(out.contains(&long), "csv shortened a value");
}
