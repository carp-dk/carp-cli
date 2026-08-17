// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;
use carp_client::api::models::{ParticipantSummary, StudyOverview};
use carp_client::fixtures::{PARTICIPANT_GROUP_MEMBER_ID, PARTICIPANT_GROUP_STATUS};

fn study_with_one_deployed_participant() -> StudyState {
    let mut state = StudyState::new(StudyOverview::default());
    state.set_groups(serde_json::from_str(PARTICIPANT_GROUP_STATUS).unwrap());
    state.participants.set_items(
        vec![ParticipantSummary {
            participant_id: PARTICIPANT_GROUP_MEMBER_ID.to_owned(),
            first_name: Some("Ada".to_owned()),
            last_name: Some("Lovelace".to_owned()),
            ..ParticipantSummary::default()
        }],
        1,
        false,
    );
    state
}

/// The participants tab and the deployments tab must agree about who is
/// deployed where.
#[test]
fn the_join_works_in_both_directions() {
    let state = study_with_one_deployed_participant();

    let group = state
        .group_for(PARTICIPANT_GROUP_MEMBER_ID)
        .expect("the participant is a member of the fixture group");
    assert_eq!(group.short_id(), "df98d925");
    assert_eq!(
        group.assigned_devices(PARTICIPANT_GROUP_MEMBER_ID),
        ["Primary Phone"]
    );
    assert_eq!(state.group_members(group), ["Ada Lovelace"]);
}

/// A participant of no group has no deployment, and must not borrow one.
#[test]
fn a_participant_outside_every_group_has_no_deployment() {
    let state = study_with_one_deployed_participant();
    assert!(
        state
            .group_for("ffffffff-0000-0000-0000-000000000000")
            .is_none()
    );
}

/// Members are named from every page seen, not just the visible one.
#[test]
fn the_directory_survives_paging() {
    let mut state = study_with_one_deployed_participant();
    state.participants.set_items(
        vec![ParticipantSummary {
            participant_id: "page-two".to_owned(),
            first_name: Some("Grace".to_owned()),
            ..ParticipantSummary::default()
        }],
        2,
        false,
    );

    // Page two is what is shown, but page one is still resolvable.
    assert_eq!(state.participants.items.len(), 1);
    let group = state.group_for(PARTICIPANT_GROUP_MEMBER_ID).unwrap();
    assert_eq!(state.group_members(group), ["Ada Lovelace"]);
}
