// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Whether the tasks are named and say what they do.

use std::collections::HashSet;

use super::super::Diagnostic;
use crate::protocol::StudyProtocol;

/// Task names are unique and non-empty, and app tasks say what they are.
pub fn tasks(protocol: &StudyProtocol, out: &mut Vec<Diagnostic>) {
    let mut seen: HashSet<&str> = HashSet::new();
    for task in &protocol.tasks {
        let name = task.name();
        if name.trim().is_empty() {
            out.push(Diagnostic::error(
                format!("task <{}>", task.type_label()),
                "has no name",
            ));
            continue;
        }
        if !seen.insert(name) {
            out.push(
                Diagnostic::error(format!("task {name:?}"), "name is used twice")
                    .with_hint("task controls address tasks by name"),
            );
        }

        if task.measures().is_empty() && !matches!(task.kind(), Some(crate::task::TaskKind::Web)) {
            out.push(
                Diagnostic::warning(format!("task {name:?}"), "collects no measures")
                    .with_hint("the task runs but records nothing"),
            );
        }

        if let Some(app) = task.app()
            && app.title.trim().is_empty()
        {
            out.push(
                Diagnostic::warning(format!("task {name:?}"), "has no title")
                    .with_hint("the title is the heading on the participant's card"),
            );
        }
    }
}
