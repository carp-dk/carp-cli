// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The options offered by a choice question.

use serde::{Deserialize, Serialize};

/// One option of a text choice question.
///
/// `value` is what ends up in the collected data, so it - not `text` - is what
/// an analysis sees. Scored instruments depend on it: a WHO-5 item scores 0-5,
/// and changing a value silently rescores every answer already collected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpChoice {
    #[serde(rename = "__type", default = "choice_type")]
    pub type_name: String,
    /// Label shown to the participant. Often a localisation key.
    pub text: String,
    /// The recorded answer. Usually an integer score, but any JSON value the
    /// study app can store is accepted.
    pub value: serde_json::Value,
    /// Secondary line under the label, for options needing explanation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail_text: Option<String>,
    /// Whether picking this option opens a free-text field, as an "other,
    /// please specify" option does.
    #[serde(default)]
    pub is_free_text: bool,
}

fn choice_type() -> String {
    "RPChoice".to_owned()
}

impl RpChoice {
    /// An option labelled `text` recording the integer `value`.
    pub fn new(text: impl Into<String>, value: i64) -> Self {
        Self {
            type_name: choice_type(),
            text: text.into(),
            value: serde_json::Value::from(value),
            detail_text: None,
            is_free_text: false,
        }
    }

    /// The recorded value as an integer, when it is one.
    pub fn integer_value(&self) -> Option<i64> {
        self.value.as_i64()
    }

    /// `text` alongside the value it records, for the editor's list.
    pub fn label(&self) -> String {
        format!("{} = {}", self.text, self.value)
    }
}

/// One option of an image choice question, such as a row of mood faces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpImageChoice {
    #[serde(rename = "__type", default = "image_choice_type")]
    pub type_name: String,
    /// Asset path inside the study app, e.g. `assets/icons/very-sad.png`.
    /// The image has to be bundled with the app; the protocol only names it.
    pub image_url: String,
    /// The recorded answer.
    pub value: serde_json::Value,
    /// Caption under the image. Often a localisation key.
    pub description: String,
}

fn image_choice_type() -> String {
    "RPImageChoice".to_owned()
}

impl RpImageChoice {
    pub fn new(image_url: impl Into<String>, value: i64, description: impl Into<String>) -> Self {
        Self {
            type_name: image_choice_type(),
            image_url: image_url.into(),
            value: serde_json::Value::from(value),
            description: description.into(),
        }
    }

    pub fn label(&self) -> String {
        format!("{} = {}", self.description, self.value)
    }
}

#[cfg(test)]
mod tests;
