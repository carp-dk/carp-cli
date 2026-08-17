// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

#[test]
fn a_background_task_is_just_a_name_and_measures() {
    let original = serde_json::json!({
        "__type": "dk.cachet.carp.common.application.tasks.BackgroundTask",
        "name": "Task #7",
        "measures": [{
            "__type": "dk.cachet.carp.common.application.tasks.Measure.DataStream",
            "type": "dk.cachet.carp.heartbeat"
        }]
    });

    let task: Task = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(task.name(), "Task #7");
    assert!(!task.is_visible_to_participant());
    assert_eq!(task.measures().len(), 1);
    assert_eq!(serde_json::to_value(&task).unwrap(), original);
}

/// An app task without an estimated duration must not gain a null one:
/// the study app renders "null minutes" if it does.
#[test]
fn an_absent_duration_stays_absent() {
    let original = serde_json::json!({
        "__type": "dk.cachet.carp.common.application.tasks.AppTask",
        "name": "Task #1",
        "measures": [],
        "type": "audio",
        "title": "reading.title",
        "description": "reading.description",
        "instructions": "reading.instructions",
        "notification": false
    });

    let task: Task = serde_json::from_value(original.clone()).unwrap();
    let written = serde_json::to_value(&task).unwrap();
    assert!(
        written.get("minutesToComplete").is_none(),
        "an absent duration must not be written back, got {written}"
    );
    assert_eq!(written, original);
}

#[test]
fn a_web_task_round_trips() {
    let original = serde_json::json!({
        "__type": "dk.cachet.carp.common.application.tasks.WebTask",
        "name": "List Learning",
        "measures": [],
        "description": "Testing verbal memory (immediate recall).",
        "url": "https://icat.cachet.dk"
    });

    let task: Task = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(task.kind(), Some(TaskKind::Web));
    assert_eq!(serde_json::to_value(&task).unwrap(), original);
}
