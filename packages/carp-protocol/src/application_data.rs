// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The `applicationData` block: CARP Mobile Sensing's extensions.
//!
//! CARP's core protocol format has no room for "who is responsible for this
//! study" or "where does the data go", because those are properties of the
//! *app* running the protocol rather than of the measurement plan. CAMS
//! therefore parks them in a free-form `applicationData` object that the core
//! runtime passes through untouched.
//!
//! Protocols aimed at the core runtime alone leave the block out entirely -
//! the browser-based ICAT study does - so the whole thing is optional.

use serde::{Deserialize, Serialize};

use crate::node::UnknownNode;

/// The CAMS extension block of a protocol.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationData {
    /// Protocol API level the study app should read this document as, e.g.
    /// `"2.0"`. Absent on protocols written before the level was introduced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_api_level: Option<String>,
    /// Flutter application id the protocol is written for, e.g.
    /// `"neuropathy_tracker"`. Lets one CAWS instance serve several apps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub study_description: Option<StudyDescription>,
    /// Where collected data is written. Absent means the app's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_end_point: Option<DataEndPoint>,
}

/// How a study presents itself to a participant.
///
/// Most fields hold a localisation key such as `study.description.title`
/// rather than prose: the study app resolves them against the language files
/// that sit beside `protocol.json`. Plain text is equally valid and several
/// protocols use it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyDescription {
    #[serde(rename = "__type")]
    #[serde(default = "study_description_type")]
    pub type_name: String,
    pub title: String,
    pub description: String,
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub study_description_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy_policy_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub responsible: Option<StudyResponsible>,
}

fn study_description_type() -> String {
    "StudyDescription".to_owned()
}

impl StudyDescription {
    /// A description whose text fields are localisation keys under `prefix`,
    /// matching the convention the reference protocols follow.
    pub fn localised(prefix: &str) -> Self {
        Self {
            type_name: study_description_type(),
            title: format!("{prefix}.title"),
            description: format!("{prefix}.description"),
            purpose: format!("{prefix}.purpose"),
            study_description_url: Some(format!("{prefix}.url")),
            privacy_policy_url: Some(format!("{prefix}.privacy")),
            responsible: Some(StudyResponsible::localised("study.responsible")),
        }
    }
}

/// The person accountable for a study, shown in the app's about screen.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyResponsible {
    #[serde(rename = "__type")]
    #[serde(default = "study_responsible_type")]
    pub type_name: String,
    pub id: String,
    pub name: String,
    pub title: String,
    pub email: String,
    pub address: String,
    pub affiliation: String,
}

fn study_responsible_type() -> String {
    "StudyResponsible".to_owned()
}

impl StudyResponsible {
    /// A responsible party whose fields are localisation keys under `prefix`.
    pub fn localised(prefix: &str) -> Self {
        Self {
            type_name: study_responsible_type(),
            id: format!("{prefix}.id"),
            name: format!("{prefix}.name"),
            title: format!("{prefix}.title"),
            email: format!("{prefix}.email"),
            address: format!("{prefix}.address"),
            affiliation: format!("{prefix}.affiliation"),
        }
    }
}

/// Where collected data is written.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataEndPoint {
    Known(KnownDataEndPoint),
    /// An endpoint type this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// The endpoint types this crate models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "__type")]
pub enum KnownDataEndPoint {
    /// Upload to the CARP web service.
    #[serde(rename = "CarpDataEndPoint", rename_all = "camelCase")]
    Carp {
        /// Always `"CAWS"`; kept as data so a future value round-trips.
        #[serde(default = "caws")]
        r#type: String,
        #[serde(default = "carp_data_format")]
        data_format: String,
        /// `"stream"`, `"datapoint"` or `"file"`.
        upload_method: String,
        name: String,
        /// Capital `F`, which `rename_all = "camelCase"` would not produce.
        #[serde(rename = "onlyUploadOnWiFi", default)]
        only_upload_on_wifi: bool,
        /// **Minutes** between upload attempts - not a microsecond duration
        /// like the rest of the schema. CAMS reads it as
        /// `Duration(minutes: uploadInterval)`.
        #[serde(default = "default_upload_interval")]
        upload_interval: i64,
        #[serde(default)]
        delete_when_uploaded: bool,
        #[serde(default = "yes")]
        compress: bool,
    },
    /// Keep the data in the phone's local database only.
    #[serde(rename = "SQLiteDataEndPoint", rename_all = "camelCase")]
    SqLite {
        #[serde(default = "sqlite")]
        r#type: String,
        #[serde(default = "carp_data_format")]
        data_format: String,
    },
}

fn caws() -> String {
    "CAWS".to_owned()
}

fn sqlite() -> String {
    "SQLITE".to_owned()
}

fn carp_data_format() -> String {
    "dk.cachet.carp".to_owned()
}

fn default_upload_interval() -> i64 {
    10
}

fn yes() -> bool {
    true
}

impl DataEndPoint {
    /// The default endpoint: stream to CAWS, keeping a local copy.
    pub fn carp_stream() -> Self {
        Self::Known(KnownDataEndPoint::Carp {
            r#type: caws(),
            data_format: carp_data_format(),
            upload_method: "stream".to_owned(),
            name: "CARP Web Service".to_owned(),
            only_upload_on_wifi: false,
            upload_interval: default_upload_interval(),
            delete_when_uploaded: false,
            compress: true,
        })
    }

    /// A one-line label for the editor.
    pub fn label(&self) -> String {
        match self {
            Self::Known(KnownDataEndPoint::Carp {
                upload_method,
                name,
                ..
            }) => format!("{name} ({upload_method})"),
            Self::Known(KnownDataEndPoint::SqLite { .. }) => "local SQLite database".to_owned(),
            Self::Unknown(node) => node.short_type().to_owned(),
        }
    }
}

#[cfg(test)]
mod tests;
