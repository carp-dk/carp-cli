// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Activity indicator driven by the tick counter.

const FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

pub fn frame(ticks: usize) -> &'static str {
    FRAMES[ticks % FRAMES.len()]
}

/// `⠹ loading` when busy, empty otherwise.
pub fn label(ticks: usize, busy: bool) -> String {
    if busy {
        format!("{} loading", frame(ticks))
    } else {
        String::new()
    }
}
