// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The participants of a study: a paged, searchable, sortable list with the
//! highlighted participant described beside it.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Row;

use crate::app::state::StudyState;
use crate::ui::screens::study::{DETAIL_WEIGHT, LIST_WEIGHT, deployments, tab_title};
use crate::ui::widgets::{detail, master_detail, table};
use crate::ui::{icons, theme};
use carp_client::api::models::{ParticipantSummary, format_instant};

pub fn render(frame: &mut Frame, area: Rect, study: &mut StudyState, ticks: usize) {
    let (list_area, detail_area) = master_detail(area, LIST_WEIGHT, DETAIL_WEIGHT);
    if let Some(detail_area) = detail_area {
        render_detail(frame, detail_area, study);
    }
    render_list(frame, list_area, study, ticks);
}

fn render_list(frame: &mut Frame, area: Rect, study: &mut StudyState, ticks: usize) {
    let mut title = tab_title(
        "Participants",
        study.participants.total.to_string(),
        ticks,
        study.participants.loading,
    );
    if study.participants.from_cache {
        title.push_str(" · cached");
    }
    // Paging, sort and filter live on the bottom border: they belong to the
    // list, and the query is never applied invisibly.
    let block = theme::focused_block(&title).title_bottom(footer(study));

    if study.participants.items.is_empty() {
        let message = if study.participants.loading {
            "loading participants…"
        } else if study.participants.query.search.is_some() {
            "no participant matches the search - press / to change it"
        } else {
            "this study has no participants yet"
        };
        frame.render_widget(table::placeholder(message, block), area);
        return;
    }

    let rows: Vec<Row> = study
        .participants
        .items
        .iter()
        .map(|participant| {
            // The deployment column is the participant's own deployment: the
            // group they were invited in, and how far its devices have got.
            let group = study.group_for(&participant.participant_id);
            let deployment = group.map_or_else(
                || Line::styled("not deployed".to_owned(), theme::dim()),
                |group| {
                    Line::styled(
                        icons::with(
                            icons::deployment_state(group.state()),
                            format!("{} {}", group.state(), group.device_progress()),
                        ),
                        deployments::state_style(group.state()),
                    )
                },
            );
            Row::new(vec![
                Line::styled(
                    icons::with(
                        icons::device(participant.deployed, false),
                        participant.display_name(),
                    ),
                    theme::value(),
                ),
                Line::raw(participant.identity().to_owned()),
                deployment,
            ])
        })
        .collect();

    let header = table::header(["Participant", "Identity", "Deployment"]);
    let widths = vec![
        Constraint::Fill(2),
        Constraint::Fill(3),
        Constraint::Length(16 + icons::cell_width()),
    ];

    let len = rows.len();
    table::render(
        frame,
        area,
        table::table(header, rows, widths, block),
        &mut study.participants.table,
        len,
    );
}

fn render_detail(frame: &mut Frame, area: Rect, study: &StudyState) {
    let block = theme::block("Participant");

    let Some(participant) = study.participants.selected() else {
        frame.render_widget(detail::empty(block, "no participant selected"), area);
        return;
    };

    let mut lines = vec![
        detail::section("identity"),
        detail::field("name", participant.display_name()),
        detail::field("identity", participant.identity().to_owned()),
        detail::field("participant id", participant.participant_id.clone()),
        detail::blank(),
        detail::section("status"),
        detail::field_styled(
            "deployment",
            icons::with(
                icons::device(participant.deployed, false),
                participant.deployment_label(),
            ),
            deployment_style(participant),
        ),
        detail::field_highlighted("account", participant.account_label().to_owned()),
        detail::field("invited", format_instant(participant.invited_on)),
        detail::blank(),
    ];

    lines.extend(deployment_lines(study, participant));
    lines.push(detail::blank());
    lines.push(detail::note("enter opens the full participant view"));

    frame.render_widget(detail::panel(block, lines), area);
}

/// The deployment this participant belongs to. Shared with the full
/// participant screen so both tell the same story.
pub fn deployment_lines(
    study: &StudyState,
    participant: &ParticipantSummary,
) -> Vec<ratatui::text::Line<'static>> {
    let mut lines = vec![detail::section("deployment")];

    let Some(group) = study.group_for(&participant.participant_id) else {
        lines.push(detail::note(if study.details_loaded {
            "This participant is not a member of any participant group, so no \
             deployment is collecting their data."
        } else {
            "loading the participant groups…"
        }));
        return lines;
    };

    let status = &group.deployment_status;
    lines.push(detail::field("group", group.short_id().to_owned()));
    lines.push(detail::field_styled(
        "state",
        icons::with(icons::deployment_state(group.state()), group.state()),
        deployments::state_style(group.state()),
    ));
    lines.push(detail::field("devices", status.device_progress()));
    lines.push(detail::field("created", format_instant(status.created_on)));

    let assigned = group.assigned_devices(&participant.participant_id);
    if !assigned.is_empty() {
        lines.push(detail::field("assigned", assigned.join(", ")));
    }

    let others: Vec<String> = study
        .group_members(group)
        .into_iter()
        .filter(|name| *name != participant.display_name())
        .collect();
    if !others.is_empty() {
        lines.push(detail::field("shared with", others.join(", ")));
    }

    lines
}

/// Page, sort and filter state, so the list is never silently narrowed.
fn footer(study: &StudyState) -> Line<'static> {
    let query = &study.participants.query;
    let page = query.page + 1;
    let pages = study.participants.page_count();

    let mut spans = vec![
        Span::styled(" page ", theme::dim()),
        Span::styled(format!("{page}/{pages}"), theme::label()),
        Span::styled(" · ", theme::dim()),
        Span::styled(
            format!(
                "sort {} {}",
                query.sort_by.label(),
                query.sort_direction.label()
            ),
            theme::label(),
        ),
        Span::styled(" · ", theme::dim()),
        Span::styled(
            match query.deployed {
                None => "all".to_owned(),
                Some(true) => "deployed only".to_owned(),
                Some(false) => "not deployed only".to_owned(),
            },
            theme::label(),
        ),
    ];

    if let Some(search) = &query.search {
        spans.push(Span::styled(" · ", theme::dim()));
        spans.push(Span::styled(format!("search \"{search}\""), theme::warn()));
    }
    spans.push(Span::styled(" ", theme::dim()));

    Line::from(spans)
}

fn deployment_style(participant: &ParticipantSummary) -> ratatui::style::Style {
    if participant.deployed {
        theme::ok()
    } else {
        theme::warn()
    }
}
