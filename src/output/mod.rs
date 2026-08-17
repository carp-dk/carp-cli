// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! How a result reaches whoever asked for it.
//!
//! Two audiences want different things from the same command. A person at a
//! terminal wants a table they can read; a script — or the Python module, or
//! `jq` — wants the whole record, unabridged and parseable. Rather than make
//! everyone pass a flag, the default follows where the output is going: a
//! terminal gets the table, a pipe gets JSON. `--format` overrides it when the
//! guess is wrong.
//!
//! The table is deliberately the lossy one. It shows the columns worth
//! scanning; JSON shows every field the server sent.

pub mod table;

use std::io::{self, IsTerminal, Write};

use color_eyre::Result;
use serde::Serialize;

pub use table::Rows;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Format {
    /// Aligned columns, for reading. Shows selected fields.
    Table,
    /// One JSON document holding the whole result.
    Json,
    /// One JSON document per line, for streaming into a reader that takes
    /// records one at a time.
    Ndjson,
    /// Comma-separated, RFC 4180 quoted. Same columns as `table`.
    Csv,
    /// Follow where the output is going: `table` to a terminal, `json` to
    /// anything else.
    #[default]
    Auto,
}

impl std::fmt::Display for Format {
    /// The name clap accepts for this variant, so `--help` can show the
    /// default as something that could be typed back.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Table => "table",
            Self::Json => "json",
            Self::Ndjson => "ndjson",
            Self::Csv => "csv",
            Self::Auto => "auto",
        })
    }
}

impl Format {
    /// Settle `Auto` against the stream the output is actually going to.
    pub fn resolve(self) -> Self {
        match self {
            Self::Auto if io::stdout().is_terminal() => Self::Table,
            Self::Auto => Self::Json,
            explicit => explicit,
        }
    }

    /// Whether errors should be reported as JSON rather than as a sentence.
    /// Follows the same reasoning: if the caller is parsing the output, it is
    /// parsing the failures too.
    pub fn is_machine_readable(self) -> bool {
        !matches!(self.resolve(), Self::Table)
    }
}

/// Print a list of records.
///
/// `Rows` supplies the columns for the human-facing formats; `Serialize`
/// supplies everything for the machine-facing ones. Both come from the same
/// value, so a table can never disagree with the JSON beside it.
pub fn rows<T: Rows + Serialize>(items: &[T], format: Format) -> Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    match format.resolve() {
        Format::Table | Format::Auto => table::write(&mut out, T::HEADERS, &cells(items)),
        Format::Csv => table::write_csv(&mut out, T::HEADERS, &cells(items)),
        Format::Json => {
            serde_json::to_writer_pretty(&mut out, items)?;
            writeln!(out)?;
            Ok(())
        }
        Format::Ndjson => {
            for item in items {
                serde_json::to_writer(&mut out, item)?;
                writeln!(out)?;
            }
            Ok(())
        }
    }?;
    out.flush()?;
    Ok(())
}

/// Print one record: a labelled list rather than a one-row table.
///
/// `detail` is what the human-facing formats show, in the order given. The
/// machine-facing ones ignore it and serialise `item` whole.
pub fn detail<T: Serialize>(item: &T, detail: &[(&str, String)], format: Format) -> Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    match format.resolve() {
        Format::Table | Format::Csv | Format::Auto => {
            let width = detail
                .iter()
                .map(|(label, _)| label.chars().count())
                .max()
                .unwrap_or(0);
            for (label, value) in detail {
                writeln!(out, "{label:width$}  {value}")?;
            }
            Ok::<_, color_eyre::Report>(())
        }
        Format::Json => {
            serde_json::to_writer_pretty(&mut out, item)?;
            writeln!(out)?;
            Ok(())
        }
        Format::Ndjson => {
            serde_json::to_writer(&mut out, item)?;
            writeln!(out)?;
            Ok(())
        }
    }?;
    out.flush()?;
    Ok(())
}

/// Print a raw JSON value exactly as the server sent it.
///
/// The escape hatch for a payload this build does not model — see
/// `carp data query --raw`.
pub fn raw(value: &serde_json::Value, format: Format) -> Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    match format.resolve() {
        Format::Ndjson => serde_json::to_writer(&mut out, value)?,
        _ => serde_json::to_writer_pretty(&mut out, value)?,
    }
    writeln!(out)?;
    out.flush()?;
    Ok(())
}

/// Say something to the person running the command, not to the pipe.
///
/// Progress and confirmations go to stderr so they cannot corrupt a result
/// being parsed downstream — `carp export create x | jq` must see only JSON.
pub fn note(message: impl std::fmt::Display) {
    eprintln!("{message}");
}

fn cells<T: Rows>(items: &[T]) -> Vec<Vec<String>> {
    items.iter().map(Rows::cells).collect()
}

#[cfg(test)]
mod tests;
