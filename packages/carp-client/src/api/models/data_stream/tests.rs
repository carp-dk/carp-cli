// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

#[test]
fn a_data_type_is_written_as_one_dotted_string() {
    let heart_rate: NamespacedId = "dk.cachet.carp.heartrate".parse().unwrap();
    assert_eq!(heart_rate.namespace, "dk.cachet.carp");
    assert_eq!(heart_rate.name, "heartrate");
    assert_eq!(heart_rate.to_string(), "dk.cachet.carp.heartrate");

    // Round trips, which is what makes it usable as a command line argument.
    for value in [
        "dk.cachet.carp.heartrate",
        "dk.cachet.carp.stepcount",
        "dk.carp.webservices.custom",
    ] {
        assert_eq!(value.parse::<NamespacedId>().unwrap().to_string(), value);
    }
}

/// The API takes the two halves separately, so the split has to be the one
/// CARP means: everything up to the last dot is the namespace.
#[test]
fn a_data_type_splits_at_the_last_dot() {
    let id: NamespacedId = "dk.cachet.carp.heartrate".parse().unwrap();
    assert_eq!(
        serde_json::to_value(&id).unwrap(),
        serde_json::json!({ "namespace": "dk.cachet.carp", "name": "heartrate" })
    );
}

#[test]
fn an_unqualified_data_type_is_passed_through_rather_than_refused() {
    let id: NamespacedId = "heartrate".parse().unwrap();
    assert_eq!(id.namespace, "");
    assert_eq!(id.name, "heartrate");
    assert_eq!(id.to_string(), "heartrate");

    assert!("".parse::<NamespacedId>().is_err());
    assert!("   ".parse::<NamespacedId>().is_err());
}

/// A trimmed batch in the shape the hand-written serialiser is expected to
/// produce: a bare list of sequences.
const BATCH_AS_LIST: &str = r#"[
  {
    "dataStream": {
      "studyDeploymentId": "df98d925-3ab4-4b78-8139-fea86d809dc5",
      "deviceRoleName": "Primary Phone",
      "dataType": { "namespace": "dk.cachet.carp", "name": "stepcount" }
    },
    "firstSequenceId": 40,
    "triggerIds": [1],
    "measurements": [
      {
        "sensorStartTime": 1723464000000000,
        "data": { "__type": "dk.cachet.carp.stepcount", "steps": 812 }
      },
      {
        "sensorStartTime": 1723467600000000,
        "sensorEndTime": 1723471200000000,
        "data": { "__type": "dk.cachet.carp.stepcount", "steps": 431 }
      }
    ]
  }
]"#;

#[test]
fn a_batch_arriving_as_a_list_is_read() {
    let batch: DataStreamBatch = serde_json::from_str(BATCH_AS_LIST).unwrap();
    assert_eq!(batch.sequences.len(), 1);
    assert_eq!(batch.measurement_count(), 2);
    assert!(!batch.is_empty());

    let sequence = &batch.sequences[0];
    assert_eq!(
        sequence.data_stream.study_deployment_id.as_str(),
        "df98d925-3ab4-4b78-8139-fea86d809dc5"
    );
    assert_eq!(sequence.data_stream.device_role_name, "Primary Phone");
    assert_eq!(
        sequence.data_stream.data_type.to_string(),
        "dk.cachet.carp.stepcount"
    );
    assert_eq!(sequence.trigger_ids, [1]);
}

/// The documented `{ isEmpty, sequences }` wrapper, and the doubly-wrapped
/// form a Kotlin `Sequence` would produce. Which one a deployment sends is not
/// knowable from the OpenAPI document, so all three are read.
#[test]
fn a_batch_is_read_however_it_is_wrapped() {
    let inner: serde_json::Value = serde_json::from_str(BATCH_AS_LIST).unwrap();

    let wrapped = serde_json::json!({ "isEmpty": false, "sequences": inner.clone() });
    let batch: DataStreamBatch = serde_json::from_value(wrapped).unwrap();
    assert_eq!(batch.measurement_count(), 2);

    let doubly = serde_json::json!({ "sequences": { "sequences": inner } });
    let batch: DataStreamBatch = serde_json::from_value(doubly).unwrap();
    assert_eq!(batch.measurement_count(), 2);
}

#[test]
fn an_empty_batch_is_read_from_every_shape_that_means_nothing() {
    for empty in [
        serde_json::json!([]),
        serde_json::json!(null),
        serde_json::json!({ "isEmpty": true, "sequences": [] }),
        serde_json::json!({ "isEmpty": true }),
    ] {
        let batch: DataStreamBatch = serde_json::from_value(empty.clone()).unwrap();
        assert!(batch.is_empty(), "{empty} should have read as empty");
        assert_eq!(batch.measurement_count(), 0);
        assert!(batch.rows().is_empty());
    }
}

/// A sequence numbers its measurements from `firstSequenceId`; flattening has
/// to keep that numbering, because it is how a row is identified in CARP.
#[test]
fn flattening_numbers_each_measurement_from_the_sequence_start() {
    let batch: DataStreamBatch = serde_json::from_str(BATCH_AS_LIST).unwrap();
    let rows = batch.rows();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].sequence_id, 40);
    assert_eq!(rows[1].sequence_id, 41);

    // The stream is folded into every row, so a row stands on its own.
    for row in &rows {
        assert_eq!(row.deployment_id, "df98d925-3ab4-4b78-8139-fea86d809dc5");
        assert_eq!(row.device_role_name, "Primary Phone");
        assert_eq!(row.data_type, "dk.cachet.carp.stepcount");
        assert_eq!(row.trigger_ids, [1]);
    }
    assert_eq!(rows[0].data["steps"], 812);
    assert_eq!(rows[1].data["steps"], 431);
}

#[test]
fn sensor_times_are_offered_raw_and_readable() {
    let batch: DataStreamBatch = serde_json::from_str(BATCH_AS_LIST).unwrap();
    let rows = batch.rows();

    // Exactly what the server said, and the same instant made readable.
    assert_eq!(rows[0].sensor_start_time, 1_723_464_000_000_000);
    assert_eq!(
        rows[0].start.unwrap().to_rfc3339(),
        "2024-08-12T12:00:00+00:00"
    );

    // An instantaneous measurement has no end; a period does.
    assert_eq!(rows[0].sensor_end_time, None);
    assert_eq!(rows[0].end, None);
    assert_eq!(rows[1].sensor_end_time, Some(1_723_471_200_000_000));
    assert_eq!(
        rows[1].end.unwrap().to_rfc3339(),
        "2024-08-12T14:00:00+00:00"
    );
}

/// A newer CARP may add fields to a sequence or a measurement. Dropping them
/// would silently lose data a study collected, so they are carried through to
/// the JSON output instead.
#[test]
fn unmodelled_fields_survive_to_the_output() {
    let batch: DataStreamBatch = serde_json::from_value(serde_json::json!([{
        "dataStream": {
            "studyDeploymentId": "df98d925-3ab4-4b78-8139-fea86d809dc5",
            "deviceRoleName": "Primary Phone",
            "dataType": { "namespace": "dk.cachet.carp", "name": "heartrate" }
        },
        "firstSequenceId": 0,
        "syncPoint": { "synchronizedOn": "2024-08-12T12:00:00Z" },
        "somethingNewer": 42,
        "measurements": [{
            "sensorStartTime": 1_723_464_000_000_000_i64,
            "data": { "__type": "dk.cachet.carp.heartrate", "bpm": 61 },
            "alsoNewer": true
        }]
    }]))
    .unwrap();

    let sequence = &batch.sequences[0];
    assert_eq!(sequence.extra["somethingNewer"], 42);
    assert!(sequence.extra.contains_key("syncPoint"));
    assert_eq!(sequence.measurements[0].extra["alsoNewer"], true);

    let json = serde_json::to_value(&batch).unwrap();
    assert_eq!(json["sequences"][0]["somethingNewer"], 42);
    assert_eq!(json["sequences"][0]["measurements"][0]["alsoNewer"], true);
}

/// A response that is neither a list nor an object says so, rather than
/// reading as an empty result — "no data" and "unreadable" must not look alike.
#[test]
fn an_unreadable_batch_is_an_error_not_an_empty_one() {
    let error = serde_json::from_value::<DataStreamBatch>(serde_json::json!("nonsense"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("data stream batch"), "{error}");
}

#[test]
fn a_stream_is_addressed_by_deployment_device_and_type() {
    let id = DataStreamId::new(
        "df98d925-3ab4-4b78-8139-fea86d809dc5",
        "Primary Phone",
        "dk.cachet.carp.heartrate".parse().unwrap(),
    );
    assert_eq!(
        serde_json::to_value(&id).unwrap(),
        serde_json::json!({
            "studyDeploymentId": "df98d925-3ab4-4b78-8139-fea86d809dc5",
            "deviceRoleName": "Primary Phone",
            "dataType": { "namespace": "dk.cachet.carp", "name": "heartrate" }
        })
    );
}
