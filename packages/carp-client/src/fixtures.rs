// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Sample payloads, as CARP actually sends them.
//!
//! These are trimmed copies of real responses, kept so that both this crate's
//! tests and those of anything built on it are written against what the server
//! sends rather than against what the model happens to accept. A hand-written
//! literal shaped to fit the struct would pass whatever the struct did.
//!
//! Public, and not gated behind `cfg(test)`, because the callers that need
//! them most are in other crates — a `#[cfg(test)]` item is invisible across a
//! crate boundary. They are string constants; nothing that does not name one
//! carries them.

/// A trimmed `GET /api/studies/{study-id}/participantGroup/status` response.
///
/// Exercises the participant/deployment join: one invited group, a primary
/// device and an optional connected one, and a participant assigned to the
/// primary.
pub const PARTICIPANT_GROUP_STATUS: &str = r#"{
      "groups": [{
        "participantGroupId": "df98d925-3ab4-4b78-8139-fea86d809dc5",
        "deploymentStatus": {
          "__type": "dk.cachet.carp.deployments.application.StudyDeploymentStatus.Invited",
          "createdOn": "2024-10-16T14:22:48.017632727Z",
          "studyDeploymentId": "df98d925-3ab4-4b78-8139-fea86d809dc5",
          "deviceStatusList": [
            {
              "__type": "dk.cachet.carp.deployments.application.DeviceDeploymentStatus.Unregistered",
              "device": {
                "__type": "dk.cachet.carp.common.application.devices.Smartphone",
                "isPrimaryDevice": true,
                "roleName": "Primary Phone"
              },
              "canBeDeployed": true,
              "remainingDevicesToRegisterToObtainDeployment": ["Primary Phone"]
            },
            {
              "__type": "dk.cachet.carp.deployments.application.DeviceDeploymentStatus.Unregistered",
              "device": {
                "__type": "dk.cachet.carp.common.application.devices.LocationService",
                "roleName": "Location Service",
                "isOptional": true
              },
              "canBeDeployed": false
            }
          ],
          "participantStatusList": [
            {
              "participantId": "0c1b0e6c-1111-2222-3333-444455556666",
              "assignedPrimaryDeviceRoleNames": ["Primary Phone"]
            }
          ]
        }
      }]
    }"#;

/// The participant [`PARTICIPANT_GROUP_STATUS`] lists as a member.
pub const PARTICIPANT_GROUP_MEMBER_ID: &str = "0c1b0e6c-1111-2222-3333-444455556666";
