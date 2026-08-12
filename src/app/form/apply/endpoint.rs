// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Applying the data-endpoint form.

use carp_protocol::StudyProtocol;
use carp_protocol::application_data::{DataEndPoint, KnownDataEndPoint};

use crate::app::form::Form;

use super::{Applied, default_application_data};

/// Write the data-endpoint form back.
///
/// Switching kind replaces the endpoint rather than mutating it, since the two
/// have almost no fields in common. Switching back to CAWS therefore starts
/// from the standard streaming defaults rather than from whatever a
/// half-converted object would hold.
pub fn apply(protocol: &mut StudyProtocol, form: &Form) -> Applied {
    let data = protocol
        .application_data
        .get_or_insert_with(default_application_data);

    let endpoint = match form.text("kind").as_str() {
        "SQLITE" => DataEndPoint::Known(KnownDataEndPoint::SqLite {
            r#type: "SQLITE".to_owned(),
            data_format: "dk.cachet.carp".to_owned(),
        }),
        _ => {
            let method = form.text("upload_method");
            if method.trim().is_empty() {
                return Applied::Refused("choose how the data is uploaded".to_owned());
            }

            // Start from the current CAWS endpoint when there is one, so a
            // field the form does not offer is not silently reset.
            let base = match data.data_end_point.clone() {
                Some(DataEndPoint::Known(carp @ KnownDataEndPoint::Carp { .. })) => {
                    DataEndPoint::Known(carp)
                }
                _ => DataEndPoint::carp_stream(),
            };
            let DataEndPoint::Known(KnownDataEndPoint::Carp {
                r#type,
                data_format,
                ..
            }) = base
            else {
                return Applied::Refused("could not read the current endpoint".to_owned());
            };

            DataEndPoint::Known(KnownDataEndPoint::Carp {
                r#type,
                data_format,
                upload_method: method,
                name: form.text("name"),
                only_upload_on_wifi: form.flag("only_upload_on_wifi"),
                upload_interval: form.integer("upload_interval").unwrap_or(10),
                delete_when_uploaded: form.flag("delete_when_uploaded"),
                compress: form.flag("compress"),
            })
        }
    };

    data.data_end_point = Some(endpoint);
    Applied::Changed
}
