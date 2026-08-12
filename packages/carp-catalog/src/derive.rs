// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Reading a vocabulary out of the downloaded protocols.
//!
//! Two passes over each study, because the two things being learned are
//! different in kind:
//!
//! - a **typed** pass over [`carp_protocol::StudyProtocol`], which reaches the
//!   values the model understands: measure types, role names, task types. This
//!   is the pass that matters, and it benefits from the model having already
//!   validated the shape.
//! - a **raw** pass over the same document as JSON, which reaches the values
//!   the model deliberately does not interpret - `questionType` strings buried
//!   in answer formats, `__type` discriminators of classes this build has
//!   never heard of. Doing this on the JSON rather than the model is what lets
//!   the catalogue offer a device class that [`carp_protocol`] cannot yet
//!   construct.
//!
//! Neither pass fails on surprising input. A catalogue is a set of
//! suggestions; being incomplete makes it less useful, while refusing to build
//! makes it useless.

use serde_json::Value;

use crate::catalog::{Catalog, CatalogVersion, Template, VocabularyBuilder};
use crate::snapshot::Snapshot;

/// Derive the vocabulary from `snapshot`.
///
/// Never fails: documents that do not parse are listed in
/// [`Catalog::skipped`] and otherwise ignored.
pub fn catalog(snapshot: &Snapshot) -> Catalog {
    let mut skipped = Vec::new();
    let mut studies = 0usize;

    let mut measure_types = VocabularyBuilder::default();
    let mut device_types = VocabularyBuilder::default();
    let mut health_data_types = VocabularyBuilder::default();
    let mut input_data_types = VocabularyBuilder::default();
    let mut app_task_types = VocabularyBuilder::default();
    let mut participant_roles = VocabularyBuilder::default();
    let mut device_role_names = VocabularyBuilder::default();
    let mut question_types = VocabularyBuilder::default();
    let mut user_task_conditions = VocabularyBuilder::default();
    let mut upload_methods = VocabularyBuilder::default();
    let mut location_accuracies = VocabularyBuilder::default();
    let mut templates = Vec::with_capacity(snapshot.documents.len());

    for document in &snapshot.documents {
        let study = document.study.as_str();
        let protocol: carp_protocol::StudyProtocol = match serde_json::from_str(&document.json) {
            Ok(protocol) => protocol,
            Err(error) => {
                // One malformed study upstream must not make the whole
                // catalogue unusable; it is reported instead.
                skipped.push(format!("{study}: {error}"));
                continue;
            }
        };
        studies += 1;

        // -- typed pass ---------------------------------------------------
        for device in protocol.devices() {
            device_role_names.record(device.role_name(), study);
            if let Some(sampling) = device.sampling() {
                for configuration in sampling.values() {
                    for metric in configuration.health_data_types().unwrap_or_default() {
                        health_data_types.record(metric, study);
                    }
                }
            }
        }
        for role in &protocol.participant_roles {
            participant_roles.record(&role.role, study);
        }
        for expected in &protocol.expected_participant_data {
            input_data_types.record(expected.input_data_type(), study);
        }
        for task in &protocol.tasks {
            for measure in task.measures() {
                measure_types.record(measure.data_type(), study);
                if let Some(sampling) = measure.sampling() {
                    for metric in sampling.health_data_types().unwrap_or_default() {
                        health_data_types.record(metric, study);
                    }
                }
            }
            if let Some(app) = task.app() {
                app_task_types.record(&app.r#type, study);
            }
        }

        // -- raw pass -----------------------------------------------------
        // The typed parse already succeeded, so this one cannot fail; the
        // `if let` is there so a future change to either cannot panic.
        if let Ok(raw) = serde_json::from_str::<Value>(&document.json) {
            walk(&raw, &mut |object| {
                if let Some(type_name) = string(object, "__type") {
                    if type_name.contains(".devices.") {
                        device_types.record(type_name, study);
                    }
                    if type_name.ends_with("UserTaskTrigger")
                        && let Some(condition) = string(object, "triggerCondition")
                    {
                        user_task_conditions.record(condition, study);
                    }
                }
                if let Some(question_type) = string(object, "questionType") {
                    question_types.record(question_type, study);
                }
                if let Some(method) = string(object, "uploadMethod") {
                    upload_methods.record(method, study);
                }
                if let Some(accuracy) = string(object, "accuracy") {
                    location_accuracies.record(accuracy, study);
                }
                // `types` on a health app task duplicates the metrics of its
                // sampling configuration, but a protocol may set one without
                // the other, so both are read.
                if let Some(Value::Array(items)) = object.get("types") {
                    for metric in items.iter().filter_map(Value::as_str) {
                        health_data_types.record(metric, study);
                    }
                }
            });
        }

        templates.push(Template {
            study: study.to_owned(),
            name: protocol.name.clone(),
            summary: protocol.summary(),
            has_application_data: protocol.application_data.is_some(),
        });
    }

    Catalog {
        version: Some(CatalogVersion {
            repository: snapshot.repository.clone(),
            commit: snapshot.commit.clone(),
            fetched_at: snapshot.fetched_at.clone(),
            studies,
        }),
        measure_types: measure_types.build(),
        device_types: device_types.build(),
        health_data_types: health_data_types.build(),
        input_data_types: input_data_types.build(),
        app_task_types: app_task_types.build(),
        participant_roles: participant_roles.build(),
        device_role_names: device_role_names.build(),
        question_types: question_types.build(),
        user_task_conditions: user_task_conditions.build(),
        upload_methods: upload_methods.build(),
        location_accuracies: location_accuracies.build(),
        templates,
        skipped,
    }
}

/// Call `visit` for every JSON object in `value`, including nested ones.
fn walk(value: &Value, visit: &mut impl FnMut(&serde_json::Map<String, Value>)) {
    match value {
        Value::Object(object) => {
            visit(object);
            for nested in object.values() {
                walk(nested, visit);
            }
        }
        Value::Array(items) => {
            for item in items {
                walk(item, visit);
            }
        }
        _ => {}
    }
}

/// A string field of a JSON object, if present and actually a string.
fn string<'a>(object: &'a serde_json::Map<String, Value>, key: &str) -> Option<&'a str> {
    object.get(key).and_then(Value::as_str)
}

#[cfg(test)]
mod tests;
