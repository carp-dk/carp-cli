// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! [`TriggerKind`]: the trigger classes the editor can create.

use super::{KnownTrigger, Recurrence, TimeOfDay, Trigger};
use crate::duration::Micros;

/// A trigger class that can be added to a protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriggerKind {
    NoOp,
    Immediate,
    OneTime,
    Periodic,
    RecurrentScheduled,
    CronScheduled,
    UserTask,
    NoUserTask,
    SamplingEvent,
}

impl TriggerKind {
    /// Every kind, ordered as the editor's picker shows them: the ones a
    /// study most often wants first.
    pub const ALL: [Self; 9] = [
        Self::Immediate,
        Self::RecurrentScheduled,
        Self::Periodic,
        Self::OneTime,
        Self::CronScheduled,
        Self::UserTask,
        Self::NoUserTask,
        Self::SamplingEvent,
        Self::NoOp,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::NoOp => "NoOpTrigger",
            Self::Immediate => "ImmediateTrigger",
            Self::OneTime => "OneTimeTrigger",
            Self::Periodic => "PeriodicTrigger",
            Self::RecurrentScheduled => "RecurrentScheduledTrigger",
            Self::CronScheduled => "CronScheduledTrigger",
            Self::UserTask => "UserTaskTrigger",
            Self::NoUserTask => "NoUserTaskTrigger",
            Self::SamplingEvent => "SamplingEventTrigger",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::NoOp => "Never fires; the app starts the task itself",
            Self::Immediate => "As soon as the study starts, and on every restart",
            Self::OneTime => "Once, on the first run of the study only",
            Self::Periodic => "Every fixed interval since the study started",
            Self::RecurrentScheduled => "At a wall-clock time, daily to monthly",
            Self::CronScheduled => "On a cron expression",
            Self::UserTask => "When another task reaches a state, e.g. done",
            Self::NoUserTask => "When another task leaves the participant's list",
            Self::SamplingEvent => "When a measure produces matching data",
        }
    }

    pub fn type_name(self) -> &'static str {
        match self {
            Self::NoOp => "dk.cachet.carp.common.application.triggers.NoOpTrigger",
            Self::Immediate => "dk.cachet.carp.common.application.triggers.ImmediateTrigger",
            Self::OneTime => "dk.cachet.carp.common.application.triggers.OneTimeTrigger",
            Self::Periodic => "dk.cachet.carp.common.application.triggers.PeriodicTrigger",
            Self::RecurrentScheduled => {
                "dk.cachet.carp.common.application.triggers.RecurrentScheduledTrigger"
            }
            Self::CronScheduled => {
                "dk.cachet.carp.common.application.triggers.CronScheduledTrigger"
            }
            Self::UserTask => "dk.cachet.carp.common.application.triggers.UserTaskTrigger",
            Self::NoUserTask => "dk.cachet.carp.common.application.triggers.NoUserTaskTrigger",
            Self::SamplingEvent => {
                "dk.cachet.carp.common.application.triggers.SamplingEventTrigger"
            }
        }
    }

    /// Whether the trigger has to name another task, which the editor then
    /// has to offer a picker for.
    pub fn watches_a_task(self) -> bool {
        matches!(self, Self::UserTask | Self::NoUserTask)
    }

    pub fn from_type_name(type_name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.type_name() == type_name)
    }

    /// Build a trigger of this kind, evaluated on `device`.
    ///
    /// Kinds that need more than a device get defaults a study would plausibly
    /// use - a daily 10:00 schedule, an hourly period - which the editor then
    /// lets the user change.
    pub fn instantiate(self, device: String) -> Trigger {
        let trigger = match self {
            Self::NoOp => KnownTrigger::NoOp {
                source_device_role_name: device,
            },
            Self::Immediate => KnownTrigger::Immediate {
                source_device_role_name: device,
            },
            Self::OneTime => KnownTrigger::OneTime {
                source_device_role_name: device,
            },
            Self::Periodic => KnownTrigger::Periodic {
                source_device_role_name: device,
                period: Micros::from_hours(1),
            },
            Self::RecurrentScheduled => KnownTrigger::RecurrentScheduled {
                source_device_role_name: device,
                r#type: Recurrence::Daily.wire_name().to_owned(),
                time: TimeOfDay::new(10, 0),
                separation_count: 0,
                day_of_week: None,
                period: Recurrence::Daily.period(),
            },
            Self::CronScheduled => KnownTrigger::CronScheduled {
                source_device_role_name: device,
                cron_expression: "0 10 * * *".to_owned(),
            },
            Self::UserTask => KnownTrigger::UserTask {
                source_device_role_name: device,
                task_name: String::new(),
                trigger_condition: "done".to_owned(),
            },
            Self::NoUserTask => KnownTrigger::NoUserTask {
                source_device_role_name: device,
                task_name: String::new(),
            },
            Self::SamplingEvent => KnownTrigger::SamplingEvent {
                source_device_role_name: device,
                measure_type: String::new(),
                trigger_condition: serde_json::Value::Null,
            },
        };
        Trigger::Known(Box::new(trigger))
    }
}

/// Set a recurrent trigger's recurrence, keeping `period` and `dayOfWeek`
/// consistent with it.
///
/// CARP stores the recurrence three times over - as a `type` string, as a
/// `period` duration and, for weekly schedules, as a `dayOfWeek`. Changing one
/// without the others produces a protocol that reads one way in the editor and
/// behaves another way on the phone, so the change is made here as one step.
pub fn set_recurrence(trigger: &mut Trigger, recurrence: Recurrence, day: Option<u8>) {
    let Trigger::Known(known) = trigger else {
        return;
    };
    if let KnownTrigger::RecurrentScheduled {
        r#type,
        day_of_week,
        period,
        ..
    } = known.as_mut()
    {
        *r#type = recurrence.wire_name().to_owned();
        *period = recurrence.period();
        *day_of_week = if recurrence.needs_day_of_week() {
            // Default to Monday rather than leaving a weekly trigger without
            // the day it needs.
            Some(day.unwrap_or(1))
        } else {
            None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_serialises_as_its_own_type_name() {
        for kind in TriggerKind::ALL {
            let trigger = kind.instantiate("Primary Phone".to_owned());
            let json = serde_json::to_value(&trigger).unwrap();
            assert_eq!(
                json["__type"].as_str(),
                Some(kind.type_name()),
                "{} serialised as {}",
                kind.label(),
                json["__type"]
            );
            assert_eq!(TriggerKind::from_type_name(kind.type_name()), Some(kind));
        }
    }

    #[test]
    fn a_created_trigger_reads_back_unchanged() {
        for kind in TriggerKind::ALL {
            let trigger = kind.instantiate("Primary Phone".to_owned());
            let json = serde_json::to_string(&trigger).unwrap();
            let parsed: Trigger = serde_json::from_str(&json).unwrap();

            assert_eq!(parsed, trigger, "{} did not round trip", kind.label());
            assert_eq!(parsed.source_device(), "Primary Phone");
            assert_eq!(parsed.kind(), Some(kind));
        }
    }

    /// Switching a schedule must move `period` and `dayOfWeek` with it, or the
    /// phone keeps the old cadence.
    #[test]
    fn changing_the_recurrence_keeps_every_representation_in_step() {
        let mut trigger = TriggerKind::RecurrentScheduled.instantiate("Primary Phone".to_owned());

        set_recurrence(&mut trigger, Recurrence::Weekly, Some(7));
        let json = serde_json::to_value(&trigger).unwrap();
        assert_eq!(json["type"], "weekly");
        assert_eq!(json["period"], 604_800_000_000i64);
        assert_eq!(json["dayOfWeek"], 7);

        // Going back to daily has to drop the day, not leave it stale.
        set_recurrence(&mut trigger, Recurrence::Daily, None);
        let json = serde_json::to_value(&trigger).unwrap();
        assert_eq!(json["type"], "daily");
        assert_eq!(json["period"], 86_400_000_000i64);
        assert!(json.get("dayOfWeek").is_none(), "got {json}");
    }

    /// A weekly schedule with no day given still needs one.
    #[test]
    fn a_weekly_recurrence_defaults_to_monday() {
        let mut trigger = TriggerKind::RecurrentScheduled.instantiate("Primary Phone".to_owned());
        set_recurrence(&mut trigger, Recurrence::Weekly, None);
        assert_eq!(serde_json::to_value(&trigger).unwrap()["dayOfWeek"], 1);
    }
}
