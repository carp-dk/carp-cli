// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! `carp participants` - who is enrolled in a study.

use carp_client::api::endpoints::participants;
use carp_client::api::models::{ParticipantQuery, ParticipantSummary, format_instant};
use color_eyre::Result;

use crate::cli::{Global, ParticipantsCommand, ParticipantsList};
use crate::commands::{Session, connect};
use crate::output::{self, Rows};

/// Stop after this many pages of `--all`, so a mistake against a large study
/// cannot spin indefinitely. Reported when it bites, never silent.
const MAX_PAGES: u32 = 1000;

pub async fn run(command: Option<&ParticipantsCommand>, global: &Global) -> Result<()> {
    let session = connect(global).await?;
    match command {
        Some(ParticipantsCommand::List(args)) => list(&session, args).await,
        // Bare `carp participants` cannot know which study, so clap's own
        // error is the right answer; this arm is unreachable in practice.
        None => Err(color_eyre::eyre::eyre!(
            "`carp participants` needs a study - try `carp participants list <study>`"
        )),
    }
}

impl Rows for ParticipantSummary {
    const HEADERS: &'static [&'static str] =
        &["id", "name", "identity", "state", "account", "invited"];

    fn cells(&self) -> Vec<String> {
        vec![
            self.participant_id.clone(),
            self.display_name(),
            self.identity().to_owned(),
            self.deployment_label().to_owned(),
            self.account_label().to_owned(),
            format_instant(self.invited_on),
        ]
    }
}

async fn list(session: &Session, args: &ParticipantsList) -> Result<()> {
    let query = ParticipantQuery {
        page: args.page,
        size: args.size.max(1),
        search: args.search.clone(),
        ..ParticipantQuery::default()
    };

    let first = participants::query(&session.client, &args.study, &query).await?;
    if !args.all {
        return output::rows(&first.content, session.format);
    }

    // `--all` walks the pages itself rather than asking for one enormous one:
    // the page size is the server's to cap, and a study can hold thousands.
    let pages = first.page_count(query.size);
    let mut everyone = first.content;
    for page in 1..pages.min(MAX_PAGES) {
        let next = participants::query(
            &session.client,
            &args.study,
            &ParticipantQuery {
                page,
                ..query.clone()
            },
        )
        .await?;
        if next.content.is_empty() {
            break;
        }
        everyone.extend(next.content);
    }

    if pages > MAX_PAGES {
        output::note(format!(
            "note: stopped at {MAX_PAGES} pages ({} of {} participants)",
            everyone.len(),
            first.total
        ));
    }
    output::rows(&everyone, session.format)
}
