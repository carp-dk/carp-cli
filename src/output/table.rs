// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Aligned columns, and the comma-separated form of the same.

use std::io::Write;

use color_eyre::Result;
use unicode_width::UnicodeWidthStr;

/// How wide one cell may get before it is cut short.
///
/// A study description or an error message can run to hundreds of characters,
/// and one of them is enough to push every other column off the screen. The
/// table is the readable view, so it stays readable; `--format json` is where
/// the whole value lives, and the ellipsis is the sign to go there.
const MAX_CELL: usize = 60;

/// A record that can be shown as a row.
///
/// Implemented on the API models next to the command that lists them, so the
/// choice of columns sits with the command it serves rather than with the
/// model, which several commands share.
pub trait Rows {
    /// Column names, in order. Shown uppercased.
    const HEADERS: &'static [&'static str];

    /// This record's cells, in `HEADERS` order and the same length.
    fn cells(&self) -> Vec<String>;
}

/// Write `rows` as aligned columns under `headers`.
pub fn write(out: &mut impl Write, headers: &[&str], rows: &[Vec<String>]) -> Result<()> {
    if rows.is_empty() {
        // Not an error, and not silence either: a caller who sees nothing
        // should know the question was asked and answered.
        writeln!(out, "no results")?;
        return Ok(());
    }

    let rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| row.iter().map(|cell| truncate(cell, MAX_CELL)).collect())
        .collect();

    let widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(column, header)| {
            rows.iter()
                .filter_map(|row| row.get(column))
                .map(|cell| cell.width())
                .chain(std::iter::once(header.width()))
                .max()
                .unwrap_or(0)
        })
        .collect();

    let line = |out: &mut dyn Write, cells: &[String]| -> Result<()> {
        let mut rendered = String::new();
        for (column, cell) in cells.iter().enumerate() {
            if column > 0 {
                rendered.push_str("  ");
            }
            rendered.push_str(cell);
            // The last column is not padded: trailing spaces are invisible
            // and would only survive into whatever the output is pasted into.
            if column + 1 < cells.len() {
                let width = widths.get(column).copied().unwrap_or(0);
                rendered.push_str(&" ".repeat(width.saturating_sub(cell.width())));
            }
        }
        writeln!(out, "{}", rendered.trim_end())?;
        Ok(())
    };

    let heading: Vec<String> = headers.iter().map(|h| h.to_uppercase()).collect();
    line(out, &heading)?;
    for row in &rows {
        line(out, row)?;
    }
    Ok(())
}

/// Write `rows` as RFC 4180 comma-separated values, headers included.
pub fn write_csv(out: &mut impl Write, headers: &[&str], rows: &[Vec<String>]) -> Result<()> {
    let record =
        |cells: &mut dyn Iterator<Item = &str>| cells.map(quote).collect::<Vec<_>>().join(",");
    writeln!(out, "{}", record(&mut headers.iter().copied()))?;
    for row in rows {
        writeln!(out, "{}", record(&mut row.iter().map(String::as_str)))?;
    }
    Ok(())
}

/// Quote a CSV field when it holds something that would otherwise end it.
///
/// RFC 4180: wrap in double quotes, and double any double quote inside. A
/// study name with a comma in it is not exotic, so getting this wrong would
/// silently shift every later column of that row.
fn quote(field: &str) -> String {
    if field.contains(['"', ',', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_owned()
    }
}

/// Cut `value` to `limit` display columns, marking that it was cut.
///
/// Counts display width rather than characters, because the padding does: a
/// name in CJK or with emoji would otherwise be measured at half its width and
/// break the alignment of every row after it.
fn truncate(value: &str, limit: usize) -> String {
    // Newlines would end the row wherever they fell.
    let value = value.replace(['\n', '\r'], " ");
    if value.width() <= limit {
        return value;
    }
    let mut out = String::new();
    let mut width = 0;
    for c in value.chars() {
        let next = c.to_string().width();
        if width + next > limit.saturating_sub(1) {
            break;
        }
        out.push(c);
        width += next;
    }
    out.push('…');
    out
}
