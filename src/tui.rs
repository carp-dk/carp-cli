// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Terminal plumbing: turns crossterm events and a periodic tick into
//! [`Message`]s on the application channel.

use std::time::Duration;

use futures_util::StreamExt;
use ratatui::crossterm::event::{Event, EventStream, KeyEventKind};
use tokio::sync::mpsc::UnboundedSender;

use crate::app::message::Message;

/// Drives spinners and status expiry; also bounds how long the app can sit
/// idle without noticing a closed channel.
const TICK: Duration = Duration::from_millis(200);

/// Read terminal events until the channel closes.
pub fn spawn_event_loop(tx: UnboundedSender<Message>) {
    tokio::spawn(async move {
        let mut events = EventStream::new();
        let mut ticker = tokio::time::interval(TICK);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if tx.send(Message::Tick).is_err() {
                        break;
                    }
                }
                event = events.next() => match event {
                    // Key releases and repeats are reported on some terminals;
                    // only presses should act.
                    Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                        if tx.send(Message::Key(key)).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Event::Resize(_, _))) => {
                        if tx.send(Message::Redraw).is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(error)) => {
                        let _ = tx.send(Message::Error(format!("terminal input failed: {error}")));
                        break;
                    }
                    None => break,
                },
            }
        }
    });
}
