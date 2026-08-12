// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Helpers for [`super`]: what the corpus tests use to decide whether a type
//! is modelled, and to describe where two documents differ.

use serde_json::Value;

/// Collect every `__type` in `value` that no part of this crate models.
///
/// Rather than reaching into the model, this asks each `__type` string
/// whether some `*Kind` recognises it. That keeps the check honest: adding a
/// variant without wiring up its kind still shows here.
pub fn unmodelled_types(value: &Value) -> Vec<String> {
    let mut found = Vec::new();
    collect_types(value, &mut found);
    found.sort_unstable();
    found.dedup();
    found.retain(|type_name| !is_modelled(type_name));
    found
}

fn collect_types(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(type_name)) = map.get("__type") {
                out.push(type_name.clone());
            }
            for nested in map.values() {
                collect_types(nested, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_types(item, out);
            }
        }
        _ => {}
    }
}

/// Whether some part of this crate claims to model `type_name`.
fn is_modelled(type_name: &str) -> bool {
    use carp_protocol::{DeviceKind, TaskKind, TriggerKind};

    if DeviceKind::from_type_name(type_name).is_some()
        || TaskKind::from_type_name(type_name).is_some()
        || TriggerKind::from_type_name(type_name).is_some()
    {
        return true;
    }

    // The remaining polymorphic types have no picker of their own, because the
    // editor reaches them through the value they belong to rather than from a
    // list. They are listed explicitly so that a type appearing upstream that
    // this crate has never seen still fails the test.
    const MODELLED: &[&str] = &[
        // Measures and sampling.
        "dk.cachet.carp.common.application.tasks.Measure.DataStream",
        "dk.cachet.carp.common.application.sampling.HealthSamplingConfiguration",
        "dk.cachet.carp.common.application.sampling.LocationSamplingConfiguration",
        "dk.cachet.carp.common.application.sampling.PeriodicSamplingConfiguration",
        "dk.cachet.carp.common.application.sampling.BluetoothScanPeriodicSamplingConfiguration",
        // Participants.
        "dk.cachet.carp.common.application.users.ParticipantAttribute.DefaultParticipantAttribute",
        "dk.cachet.carp.common.application.users.AssignedTo.All",
        "dk.cachet.carp.common.application.users.AssignedTo.Roles",
        // CAMS application data.
        "StudyDescription",
        "StudyResponsible",
        "CarpDataEndPoint",
        "SQLiteDataEndPoint",
        // Surveys.
        "RPOrderedTask",
        "RPNavigableOrderedTask",
        "RPInstructionStep",
        "RPCompletionStep",
        "RPQuestionStep",
        "RPFormStep",
        "RPTappingActivity",
        "RPFlankerActivity",
        "RPReactionTimeActivity",
        "RPChoiceAnswerFormat",
        "RPImageChoiceAnswerFormat",
        "RPIntegerAnswerFormat",
        "RPSliderAnswerFormat",
        "RPTextAnswerFormat",
        "RPDateTimeAnswerFormat",
        "RPFormAnswerFormat",
        "RPChoice",
        "RPImageChoice",
        "RPStepJumpRule",
    ];
    if MODELLED.contains(&type_name) {
        return true;
    }

    // A `SamplingEventTrigger` carries a data value whose `__type` is a
    // measure type, not a class this crate models. Those are opaque by design.
    type_name.starts_with("dk.cachet.carp.") && !type_name.contains(".application.")
}

/// A readable account of where two documents differ, for the failure message.
pub fn difference(original: &Value, written: &Value) -> String {
    let mut lines = Vec::new();
    walk_difference("", original, written, &mut lines);
    if lines.is_empty() {
        return "(the values compare unequal but no leaf difference was found)".to_owned();
    }
    lines.truncate(20);
    lines.join("\n")
}

fn walk_difference(path: &str, original: &Value, written: &Value, out: &mut Vec<String>) {
    match (original, written) {
        (Value::Object(left), Value::Object(right)) => {
            for (key, value) in left {
                let child = format!("{path}/{key}");
                match right.get(key) {
                    Some(other) => walk_difference(&child, value, other, out),
                    None => out.push(format!("  {child}: dropped ({})", preview(value))),
                }
            }
            for (key, value) in right {
                if !left.contains_key(key) {
                    out.push(format!("  {path}/{key}: added ({})", preview(value)));
                }
            }
        }
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                out.push(format!(
                    "  {path}: length {} became {}",
                    left.len(),
                    right.len()
                ));
                return;
            }
            for (index, (value, other)) in left.iter().zip(right).enumerate() {
                walk_difference(&format!("{path}[{index}]"), value, other, out);
            }
        }
        _ if original != written => {
            out.push(format!(
                "  {path}: {} became {}",
                preview(original),
                preview(written)
            ));
        }
        _ => {}
    }
}

fn preview(value: &Value) -> String {
    let text = value.to_string();
    if text.chars().count() > 60 {
        format!("{}…", text.chars().take(60).collect::<String>())
    } else {
        text
    }
}
