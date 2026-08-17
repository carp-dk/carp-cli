// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Writing a submitted [`Form`] back onto a protocol.
//!
//! The counterpart of [`super::build`], and deliberately not its mirror
//! image. Building reads a value directly; applying cannot, because a change
//! to one field may have to reach several places at once:
//!
//! - renaming a device or task goes through [`carp_protocol::builder`], which
//!   moves every reference with it
//! - changing a recurrence moves the `period` the phone schedules on
//! - a rename that would collide with an existing name is refused, and the
//!   caller is told why
//!
//! Everything else is a straight assignment. The return type is a
//! [`Applied`], because "the form was submitted" and "the protocol changed"
//! are different facts and the editor needs both.

pub mod device;
pub mod endpoint;
pub mod participant;
pub mod survey;
pub mod task;
pub mod trigger;

use carp_protocol::StudyProtocol;
use carp_protocol::application_data::{ApplicationData, StudyDescription, StudyResponsible};

use super::{Form, Subject};

/// What applying a form did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    /// The protocol was changed.
    Changed,
    /// The form named something that is no longer there, e.g. a device
    /// deleted in another pane. Nothing was changed.
    Vanished,
    /// The change was refused, with the reason.
    Refused(String),
}

impl Applied {
    /// The message for the status bar, if there is one to show.
    pub fn message(&self) -> Option<String> {
        match self {
            Self::Changed => None,
            Self::Vanished => Some("that is no longer part of the protocol".to_owned()),
            Self::Refused(reason) => Some(reason.clone()),
        }
    }
}

/// Write `form` back onto `protocol`.
pub fn apply(protocol: &mut StudyProtocol, form: &Form) -> Applied {
    match &form.subject {
        Subject::Protocol => apply_protocol(protocol, form),
        Subject::ApplicationData => apply_application_data(protocol, form),
        Subject::DataEndPoint => endpoint::apply(protocol, form),
        Subject::Device(role) => device::apply(protocol, form, role),
        Subject::Task(name) => task::apply(protocol, form, name),
        Subject::Trigger(id) => trigger::apply(protocol, form, *id),
        Subject::ParticipantRole(role) => participant::apply_role(protocol, form, role),
        Subject::ExpectedData(index) => participant::apply_expected(protocol, form, *index),
        Subject::SurveyStep { task, step } => survey::apply_step(protocol, form, task, *step),
        Subject::Measure { task, measure } => survey::apply_measure(protocol, form, task, *measure),
    }
}

/// The protocol's own identity.
fn apply_protocol(protocol: &mut StudyProtocol, form: &Form) -> Applied {
    let name = form.text("name");
    if name.trim().is_empty() {
        return Applied::Refused("a protocol needs a name".to_owned());
    }

    protocol.name = name;
    protocol.owner_id = form.text("owner_id");

    // An empty description is absent rather than an empty string, since that
    // is how the reference protocols write it.
    let description = form.text("description");
    protocol.description = (!description.trim().is_empty()).then_some(description);

    Applied::Changed
}

/// The CAMS `applicationData` block.
///
/// A protocol with no block at all gains one, complete with the CAWS endpoint
/// a study almost always wants - that is how a browser-only protocol acquires
/// study-app settings.
///
/// The nested `studyDescription` and `StudyResponsible` are pruned when every
/// field of them is empty *and* they were not there before. Writing an
/// all-blank description into a protocol that had none would turn an unchanged
/// form submission into a change, and put an empty object in a document CARP
/// wrote without one.
fn apply_application_data(protocol: &mut StudyProtocol, form: &Form) -> Applied {
    let existing = protocol.application_data.clone();
    let had_description = existing
        .as_ref()
        .is_some_and(|data| data.study_description.is_some());
    let had_responsible = existing.as_ref().is_some_and(|data| {
        data.study_description
            .as_ref()
            .is_some_and(|description| description.responsible.is_some())
    });

    let mut data = existing.unwrap_or_else(default_application_data);

    let api_level = form.text("api_level");
    data.protocol_api_level = (!api_level.is_empty()).then_some(api_level);

    let application_name = form.text("application_name");
    data.application_name = (!application_name.trim().is_empty()).then_some(application_name);

    let mut description = data.study_description.unwrap_or_default();
    description.type_name = "StudyDescription".to_owned();
    description.title = form.text("title");
    description.description = form.text("study_description");
    description.purpose = form.text("purpose");

    let mut responsible = description.responsible.unwrap_or_default();
    responsible.type_name = "StudyResponsible".to_owned();
    responsible.name = form.text("responsible_name");
    responsible.email = form.text("responsible_email");
    responsible.affiliation = form.text("responsible_affiliation");

    let responsible_says_something = [
        &responsible.name,
        &responsible.email,
        &responsible.affiliation,
        &responsible.id,
        &responsible.title,
        &responsible.address,
    ]
    .iter()
    .any(|field| !field.trim().is_empty());
    description.responsible =
        (had_responsible || responsible_says_something).then_some(responsible);

    let description_says_something = description.responsible.is_some()
        || [
            &description.title,
            &description.description,
            &description.purpose,
        ]
        .iter()
        .any(|field| !field.trim().is_empty())
        || description.study_description_url.is_some()
        || description.privacy_policy_url.is_some();
    data.study_description = (had_description || description_says_something).then_some(description);

    protocol.application_data = Some(data);
    Applied::Changed
}

/// A blank `applicationData` block, with the endpoint a study almost always
/// wants. Used when a protocol gains study-app settings for the first time.
pub fn default_application_data() -> ApplicationData {
    ApplicationData {
        protocol_api_level: None,
        application_name: None,
        study_description: Some(StudyDescription {
            type_name: "StudyDescription".to_owned(),
            responsible: Some(StudyResponsible {
                type_name: "StudyResponsible".to_owned(),
                ..StudyResponsible::default()
            }),
            ..StudyDescription::default()
        }),
        data_end_point: Some(carp_protocol::DataEndPoint::carp_stream()),
    }
}

#[cfg(test)]
mod reference_tests;
#[cfg(test)]
pub mod tests;
