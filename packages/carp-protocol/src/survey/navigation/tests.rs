// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Tests for [`super`].

use super::*;

#[test]
fn a_jump_rule_round_trips() {
    let original = serde_json::json!({
        "__type": "RPStepJumpRule",
        "answerMap": {
            "1": "onboarding.smoking.number.step",
            "0": "onboarding.alcohol.step"
        }
    });

    let rule: RpStepNavigationRule = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(rule.label(), "2 branches");
    assert_eq!(serde_json::to_value(&rule).unwrap(), original);
}

/// Renaming a step has to follow into the rules pointing at it, or the
/// survey dead-ends at a step that no longer exists.
#[test]
fn renaming_a_destination_rewires_the_branch() {
    let mut rule = RpStepNavigationRule::jump(BTreeMap::from([
        ("0".to_owned(), "step.a".to_owned()),
        ("1".to_owned(), "step.b".to_owned()),
    ]));

    rule.rename_destination("step.b", "step.c");
    let mut destinations = rule.destinations();
    destinations.sort_unstable();
    assert_eq!(destinations, ["step.a", "step.c"]);
}
