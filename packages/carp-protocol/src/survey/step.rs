// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Steps: the pages a participant walks through inside a survey.
//!
//! A survey is an ordered list of steps. Most are questions, but a survey also
//! opens with an instruction page, closes with a completion page, and may put
//! several related questions on one page as a form. The cognitive activities
//! (tapping, flanker, reaction time) are steps too: from the survey's point of
//! view they are just a page that produces a result.
//!
//! Every step has an `identifier`, unique within its survey, which is the key
//! its answer is recorded under and what
//! [`super::navigation::RpStepJumpRule`] points at.

pub mod access;

use serde::{Deserialize, Serialize};

use crate::node::UnknownNode;
use crate::survey::answer::RpAnswerFormat;

/// One page of a survey.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpStep {
    Known(Box<KnownStep>),
    /// A step type this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// The step types this crate models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all_fields = "camelCase")]
pub enum KnownStep {
    /// A page of text, shown before the questions start.
    #[serde(rename = "RPInstructionStep")]
    Instruction {
        identifier: String,
        title: String,
        text: String,
        /// Whether the participant may skip the page.
        #[serde(default)]
        optional: bool,
    },

    /// The closing page, shown once every question is answered.
    #[serde(rename = "RPCompletionStep")]
    Completion {
        identifier: String,
        title: String,
        text: String,
        #[serde(default)]
        optional: bool,
    },

    /// A single question.
    #[serde(rename = "RPQuestionStep")]
    Question {
        identifier: String,
        /// The question itself. Often a localisation key.
        title: String,
        /// Whether it may be left unanswered.
        #[serde(default)]
        optional: bool,
        answer_format: RpAnswerFormat,
        /// Move on by itself once answered, without a Next tap.
        #[serde(default)]
        auto_skip: bool,
        /// Seconds before moving on regardless; 0 disables it.
        #[serde(default)]
        timeout: u32,
        #[serde(default)]
        auto_focus: bool,
    },

    /// Several questions on one page.
    #[serde(rename = "RPFormStep")]
    Form {
        identifier: String,
        title: String,
        #[serde(default)]
        optional: bool,
        /// Always an `RPFormAnswerFormat`; CARP writes it even though the
        /// class already says so.
        answer_format: RpAnswerFormat,
        #[serde(default)]
        auto_skip: bool,
        #[serde(default)]
        timeout: u32,
        #[serde(default)]
        auto_focus: bool,
        /// The questions on the page.
        #[serde(default)]
        questions: Vec<RpStep>,
        /// Whether answers are kept when the page auto-skips.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        save_results_on_auto_skip: Option<bool>,
        /// Whether the whole `timeout` must elapse before Next appears.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        force_wait: Option<bool>,
    },

    /// A finger-tapping speed test.
    #[serde(rename = "RPTappingActivity")]
    Tapping {
        identifier: String,
        title: String,
        #[serde(default)]
        optional: bool,
        include_instructions: bool,
        include_results: bool,
        /// Seconds the test runs for.
        length_of_test: u32,
    },

    /// A flanker test of selective attention.
    #[serde(rename = "RPFlankerActivity")]
    Flanker {
        identifier: String,
        title: String,
        #[serde(default)]
        optional: bool,
        include_instructions: bool,
        include_results: bool,
        length_of_test: u32,
        /// How many stimulus cards to show.
        number_of_cards: u32,
    },

    /// A simple reaction-time test.
    #[serde(rename = "RPReactionTimeActivity")]
    ReactionTime {
        identifier: String,
        title: String,
        #[serde(default)]
        optional: bool,
        include_instructions: bool,
        include_results: bool,
        length_of_test: u32,
        /// Seconds between stimuli.
        switch_interval: u32,
    },
}

impl RpStep {
    /// An instruction page.
    pub fn instruction(
        identifier: impl Into<String>,
        title: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self::Known(Box::new(KnownStep::Instruction {
            identifier: identifier.into(),
            title: title.into(),
            text: text.into(),
            optional: false,
        }))
    }

    /// A completion page.
    pub fn completion(
        identifier: impl Into<String>,
        title: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self::Known(Box::new(KnownStep::Completion {
            identifier: identifier.into(),
            title: title.into(),
            text: text.into(),
            optional: false,
        }))
    }

    /// A question with the given answer format.
    pub fn question(
        identifier: impl Into<String>,
        title: impl Into<String>,
        answer_format: RpAnswerFormat,
    ) -> Self {
        Self::Known(Box::new(KnownStep::Question {
            identifier: identifier.into(),
            title: title.into(),
            optional: false,
            answer_format,
            auto_skip: false,
            timeout: 0,
            auto_focus: false,
        }))
    }

    /// The key this step's answer is recorded under.
    pub fn identifier(&self) -> &str {
        match self {
            Self::Known(step) => step.identifier(),
            Self::Unknown(node) => node
                .field("identifier")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        }
    }

    pub fn set_identifier(&mut self, identifier: impl Into<String>) {
        let identifier = identifier.into();
        match self {
            Self::Known(step) => step.set_identifier(identifier),
            Self::Unknown(node) => {
                node.fields.insert(
                    "identifier".to_owned(),
                    serde_json::Value::String(identifier),
                );
            }
        }
    }

    /// The heading shown on the page.
    pub fn title(&self) -> &str {
        match self {
            Self::Known(step) => step.title(),
            Self::Unknown(node) => node
                .field("title")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        }
    }

    pub fn set_title(&mut self, value: impl Into<String>) {
        let value = value.into();
        match self {
            Self::Known(step) => step.set_title(value),
            Self::Unknown(node) => {
                node.fields
                    .insert("title".to_owned(), serde_json::Value::String(value));
            }
        }
    }

    /// The class name, for the editor's list.
    pub fn type_label(&self) -> &str {
        match self {
            Self::Known(step) => step.type_label(),
            Self::Unknown(node) => node.short_type(),
        }
    }

    /// How this step is answered, for the steps that are questions.
    pub fn answer_format(&self) -> Option<&RpAnswerFormat> {
        match self {
            Self::Known(step) => match step.as_ref() {
                KnownStep::Question { answer_format, .. } => Some(answer_format),
                _ => None,
            },
            Self::Unknown(_) => None,
        }
    }

    pub fn answer_format_mut(&mut self) -> Option<&mut RpAnswerFormat> {
        match self {
            Self::Known(step) => match step.as_mut() {
                KnownStep::Question { answer_format, .. } => Some(answer_format),
                _ => None,
            },
            Self::Unknown(_) => None,
        }
    }

    /// The nested questions, for a form step.
    pub fn questions_mut(&mut self) -> Option<&mut Vec<RpStep>> {
        match self {
            Self::Known(step) => match step.as_mut() {
                KnownStep::Form { questions, .. } => Some(questions),
                _ => None,
            },
            Self::Unknown(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
