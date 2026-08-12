// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Building the form for the data endpoint.
//!
//! Where a study's data goes is one of the few settings with a consequence
//! outside the app: `deleteWhenUploaded` decides whether a participant can
//! still see their own data, and `onlyUploadOnWiFi` decides whether the study
//! costs them mobile data. Both are one keystroke away from being wrong, so
//! both are spelled out.

use carp_protocol::StudyProtocol;
use carp_protocol::application_data::{DataEndPoint, KnownDataEndPoint};

use crate::app::form::{Choice, Field, FieldValue, Form, Subject, Vocabulary};

/// The `dataEndPoint` of the protocol's study-app settings.
pub fn data_end_point(protocol: &StudyProtocol) -> Form {
    let endpoint = protocol
        .application_data
        .as_ref()
        .and_then(|data| data.data_end_point.clone())
        .unwrap_or_else(DataEndPoint::carp_stream);

    let mut fields = vec![kind_field(&endpoint)];

    if let DataEndPoint::Known(KnownDataEndPoint::Carp {
        upload_method,
        name,
        only_upload_on_wifi,
        upload_interval,
        delete_when_uploaded,
        compress,
        ..
    }) = &endpoint
    {
        fields.extend([
            Field::new(
                "upload_method",
                "Upload as",
                FieldValue::Catalog {
                    vocabulary: Vocabulary::UploadMethods,
                    value: upload_method.clone(),
                },
            )
            .with_help("stream sends data continuously; file uploads archives"),
            Field::new("name", "Endpoint name", FieldValue::Text(name.clone()))
                .with_help("Shown in the study app's data screen"),
            Field::new(
                "upload_interval",
                "Every (minutes)",
                FieldValue::Integer {
                    // Minutes, not a microsecond duration: the one field of
                    // the schema that is a plain count of minutes.
                    value: *upload_interval,
                    min: 1,
                    max: 1440,
                },
            )
            .with_help("How often the phone tries to upload"),
            Field::new(
                "only_upload_on_wifi",
                "Wi-Fi only",
                FieldValue::Toggle(*only_upload_on_wifi),
            )
            .with_help("Spares the participant's mobile data, at the cost of delay"),
            Field::new(
                "delete_when_uploaded",
                "Delete after upload",
                FieldValue::Toggle(*delete_when_uploaded),
            )
            .with_help("Off keeps a copy on the phone, so the participant can see it"),
            Field::new("compress", "Compress", FieldValue::Toggle(*compress)),
        ]);
    }

    Form::new(Subject::DataEndPoint, fields)
}

/// Which kind of endpoint the data goes to.
fn kind_field(endpoint: &DataEndPoint) -> Field {
    let options = vec![
        Choice::described("CAWS", "CARP web service", "Uploads to the CARP server"),
        Choice::described(
            "SQLITE",
            "On the phone only",
            "Keeps everything local; nothing is uploaded",
        ),
    ];
    let current = match endpoint {
        DataEndPoint::Known(KnownDataEndPoint::SqLite { .. }) => "SQLITE",
        _ => "CAWS",
    };
    let selected = options
        .iter()
        .position(|option| option.value == current)
        .unwrap_or(0);

    Field::new("kind", "Data goes to", FieldValue::Choice { options, selected })
        .with_help("Switching to local storage means no data reaches the researchers")
}
