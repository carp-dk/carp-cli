// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! `carp data` - what a study actually recorded.
//!
//! `summary` and `statistics` count; `query` returns the measurements
//! themselves. `query` is the one anything analysing a study reaches for, and
//! the one whose output is shaped for that: `--format ndjson` gives one
//! measurement per line, `--format csv` gives a table with the stream folded
//! into every row.

use carp_client::api::endpoints::data_streams::{self, SummaryQuery};
use carp_client::api::models::{
    DataPointCount, DataStreamId, MeasurementRow, NamespacedId, format_instant,
};
use chrono::{DateTime, Utc};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::Serialize;

use crate::cli::{DataCommand, DataQuery, DataSummary, Global};
use crate::commands::{Session, connect};
use crate::output::{self, Rows};
use carp_client::time::Moment;

pub async fn run(command: &DataCommand, global: &Global) -> Result<()> {
    let session = connect(global).await?;
    match command {
        DataCommand::Summary(args) => summary(&session, args).await,
        DataCommand::Query(args) => query(&session, args).await,
        DataCommand::Statistics { deployments } => statistics(&session, deployments).await,
    }
}

/// Resolve a `--from`/`--to` pair, refusing one that runs backwards.
///
/// An inverted window returns nothing rather than failing, which reads exactly
/// like a study with no data — worth catching here instead.
fn window(from: Moment, to: Option<Moment>) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
    let to = to.map_or_else(Utc::now, Moment::resolve);
    let from = from.resolve();
    if from > to {
        return Err(eyre!(
            "the window ends before it starts: --from {} is after --to {}",
            from.to_rfc3339(),
            to.to_rfc3339()
        ));
    }
    Ok((from, to))
}

impl Rows for DataPointCount {
    const HEADERS: &'static [&'static str] = &["date", "task", "measurements"];

    fn cells(&self) -> Vec<String> {
        vec![
            self.date
                .map_or_else(|| "-".to_owned(), |d| d.to_local_date()),
            self.task.clone(),
            self.quantity.to_string(),
        ]
    }
}

async fn summary(session: &Session, args: &DataSummary) -> Result<()> {
    let (from, to) = window(args.from, args.to)?;
    let summary = data_streams::summary(
        &session.client,
        &SummaryQuery {
            study_id: args.study.clone(),
            deployment_id: args.deployment.clone(),
            participant_id: args.participant.clone(),
            scope: args.scope.clone(),
            kind: args.kind.clone(),
            from,
            to,
        },
    )
    .await?;

    // The counts are the record; the window and total are context, so they go
    // to stderr and leave stdout parseable.
    if !session.format.is_machine_readable() {
        output::note(format!(
            "{} to {} · {} measurement(s)",
            format_instant(summary.from),
            format_instant(summary.to),
            summary.total()
        ));
    }
    output::rows(&summary.data, session.format)
}

impl Rows for MeasurementRow {
    const HEADERS: &'static [&'static str] =
        &["sequence", "start", "end", "device", "type", "data"];

    fn cells(&self) -> Vec<String> {
        vec![
            self.sequence_id.to_string(),
            self.start
                .map_or_else(|| self.sensor_start_time.to_string(), |t| t.to_rfc3339()),
            self.end.map_or_else(|| "-".to_owned(), |t| t.to_rfc3339()),
            self.device_role_name.clone(),
            self.data_type.clone(),
            // The reading is a whole object whose shape depends on the measure.
            // Compacted for the table; `--format json` has it structured.
            self.data.to_string(),
        ]
    }
}

async fn query(session: &Session, args: &DataQuery) -> Result<()> {
    let (from, to) = window(args.from, args.to)?;
    let data_type: NamespacedId = args
        .data_type
        .parse()
        .map_err(|message: String| eyre!("--type: {message}"))?;
    let stream = DataStreamId::new(args.deployment.clone(), args.device.clone(), data_type);

    // `--raw` bypasses the model entirely. The measurement payload is not in
    // the OpenAPI document, so this is how to see what a deployment really
    // sends — and the way out if this build's reading of it is ever wrong.
    if args.raw {
        let batch: serde_json::Value = session
            .client
            .post_json_with_query(
                "/api/data-stream-service/query-by-time",
                &stream,
                &[("from", from.to_rfc3339()), ("to", to.to_rfc3339())],
            )
            .await?;
        return output::raw(&batch, session.format);
    }

    let batch = data_streams::query_by_time(&session.client, &stream, from, to).await?;
    let rows = batch.rows();

    if !session.format.is_machine_readable() {
        output::note(format!(
            "{} to {} · {} measurement(s) in {} sequence(s)",
            from.to_rfc3339(),
            to.to_rfc3339(),
            batch.measurement_count(),
            batch.sequences.len()
        ));
    }
    output::rows(&rows, session.format)
}

/// Upload counts for one deployment, flattened from the nested response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatisticRow {
    deployment_id: String,
    statistic: String,
    count: i32,
    uploads: usize,
}

impl Rows for StatisticRow {
    const HEADERS: &'static [&'static str] = &["deployment", "statistic", "count", "uploads"];

    fn cells(&self) -> Vec<String> {
        vec![
            self.deployment_id.clone(),
            self.statistic.clone(),
            self.count.to_string(),
            self.uploads.to_string(),
        ]
    }
}

async fn statistics(session: &Session, deployments: &[String]) -> Result<()> {
    let statistics =
        data_streams::deployment_statistics(&session.client, deployments.to_vec()).await?;

    let rows: Vec<StatisticRow> = statistics
        .statistics
        .iter()
        .flat_map(|(deployment_id, named)| {
            named.iter().map(move |(name, statistic)| StatisticRow {
                deployment_id: deployment_id.clone(),
                statistic: name.clone(),
                count: statistic.count,
                uploads: statistic.uploads.len(),
            })
        })
        .collect();

    output::rows(&rows, session.format)
}
