// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

/// A weekly trigger writes `dayOfWeek` between `separationCount` and
/// `period`, and a daily one omits it entirely.
#[test]
fn a_weekly_trigger_round_trips_with_its_day() {
    let original = serde_json::json!({
        "__type": "dk.cachet.carp.common.application.triggers.RecurrentScheduledTrigger",
        "sourceDeviceRoleName": "Primary Phone",
        "type": "weekly",
        "time": { "hour": 10, "minute": 0, "second": 0 },
        "separationCount": 0,
        "dayOfWeek": 1,
        "period": 604_800_000_000i64
    });

    let trigger: Trigger = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(trigger.schedule_label(), "weekly on Monday at 10:00");
    assert_eq!(serde_json::to_value(&trigger).unwrap(), original);
}

#[test]
fn a_daily_trigger_omits_the_day_of_week() {
    let original = serde_json::json!({
        "__type": "dk.cachet.carp.common.application.triggers.RecurrentScheduledTrigger",
        "sourceDeviceRoleName": "Primary Phone",
        "type": "daily",
        "time": { "hour": 20, "minute": 0, "second": 0 },
        "separationCount": 0,
        "period": 86_400_000_000i64
    });

    let trigger: Trigger = serde_json::from_value(original.clone()).unwrap();
    let written = serde_json::to_value(&trigger).unwrap();
    assert!(written.get("dayOfWeek").is_none(), "got {written}");
    assert_eq!(written, original);
}

/// A sampling condition is an arbitrary data value; nothing may be lost.
#[test]
fn a_sampling_condition_is_preserved_verbatim() {
    let original = serde_json::json!({
        "__type": "dk.cachet.carp.common.application.triggers.SamplingEventTrigger",
        "sourceDeviceRoleName": "Primary Phone",
        "measureType": "dk.cachet.carp.movesense.state",
        "triggerCondition": {
            "__type": "dk.cachet.carp.movesense.state",
            "state": "tap",
            "timestamp": 258
        }
    });

    let trigger: Trigger = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(trigger.schedule_label(), "on state");
    assert_eq!(serde_json::to_value(&trigger).unwrap(), original);
}

/// Renaming a task has to reach the triggers watching it.
#[test]
fn a_watched_task_can_be_renamed() {
    let mut trigger: Trigger = serde_json::from_value(serde_json::json!({
        "__type": "dk.cachet.carp.common.application.triggers.UserTaskTrigger",
        "sourceDeviceRoleName": "Primary Phone",
        "taskName": "Old name",
        "triggerCondition": "done"
    }))
    .unwrap();

    assert_eq!(trigger.watched_task(), Some("Old name"));
    trigger.set_watched_task("New name");
    assert_eq!(trigger.watched_task(), Some("New name"));

    // A trigger that watches nothing is left alone.
    let mut immediate = Trigger::immediate("Primary Phone");
    immediate.set_watched_task("New name");
    assert_eq!(immediate.watched_task(), None);
}
