// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Answer formats: how a survey question is answered.
//!
//! Each format decides the widget the study app renders and the shape of the
//! recorded answer. Note the redundancy CARP carries: every format has a
//! `questionType` string that repeats the class name, and the choice format
//! additionally has an `answerStyle` repeating whether it is single or
//! multiple choice. All three have to agree, which is why
//! [`RpAnswerFormat::single_choice`] and friends exist rather than leaving a
//! caller to set them by hand.

use serde::{Deserialize, Serialize};

use crate::node::UnknownNode;
use crate::survey::choice::{RpChoice, RpImageChoice};

/// How a question is answered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpAnswerFormat {
    Known(Box<KnownAnswerFormat>),
    /// An answer format this version does not model. See [`crate::node`].
    Unknown(UnknownNode),
}

/// The answer formats this crate models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all_fields = "camelCase")]
pub enum KnownAnswerFormat {
    /// Pick one or several from a list of labelled options.
    #[serde(rename = "RPChoiceAnswerFormat")]
    Choice {
        /// `"SingleChoice"` or `"MultipleChoice"`; must match `answer_style`.
        question_type: String,
        choices: Vec<RpChoice>,
        /// Repeats `question_type`. CAMS reads this one when rendering.
        answer_style: String,
    },

    /// Pick one from a row of images.
    #[serde(rename = "RPImageChoiceAnswerFormat")]
    ImageChoice {
        choices: Vec<RpImageChoice>,
        #[serde(default = "image_choice_question_type")]
        question_type: String,
    },

    /// A whole number, optionally bounded and suffixed with a unit.
    #[serde(rename = "RPIntegerAnswerFormat")]
    Integer {
        min_value: i64,
        max_value: i64,
        /// Unit shown after the field, e.g. `minutes`. Often a localisation key.
        #[serde(default)]
        suffix: String,
        #[serde(default = "integer_question_type")]
        question_type: String,
    },

    /// A slider over a continuous range.
    #[serde(rename = "RPSliderAnswerFormat")]
    Slider {
        min_value: f64,
        max_value: f64,
        /// Number of steps. `10` over 0-10 gives whole-number stops.
        divisions: u32,
        /// Label at the low end.
        #[serde(default)]
        prefix: String,
        /// Label at the high end.
        #[serde(default)]
        suffix: String,
        #[serde(default = "slider_question_type")]
        question_type: String,
    },

    /// Free text.
    #[serde(rename = "RPTextAnswerFormat")]
    Text {
        /// Placeholder shown in the empty field.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hint_text: Option<String>,
        /// Whether the keyboard opens as soon as the question appears.
        #[serde(default)]
        auto_focus: bool,
        /// Turn off autocorrect and suggestions, for answers where the
        /// keyboard would get in the way.
        #[serde(default)]
        disable_helpers: bool,
        #[serde(default = "text_question_type")]
        question_type: String,
    },

    /// A date, a time, or both.
    #[serde(rename = "RPDateTimeAnswerFormat")]
    DateTime {
        /// `"Date"`, `"TimeOfDay"` or `"DateAndTime"`; must match the style.
        question_type: String,
        /// Repeats `question_type`, and is the one CAMS renders from.
        date_time_answer_style: String,
    },

    /// Marks a step that groups other questions. See
    /// [`crate::survey::step::KnownStep::Form`].
    #[serde(rename = "RPFormAnswerFormat")]
    Form {
        #[serde(default = "form_question_type")]
        question_type: String,
    },
}

fn image_choice_question_type() -> String {
    "ImageChoice".to_owned()
}

fn integer_question_type() -> String {
    "Integer".to_owned()
}

fn slider_question_type() -> String {
    "Scale".to_owned()
}

fn text_question_type() -> String {
    "Text".to_owned()
}

fn form_question_type() -> String {
    "Form".to_owned()
}

impl RpAnswerFormat {
    /// Pick exactly one of `choices`.
    pub fn single_choice(choices: Vec<RpChoice>) -> Self {
        Self::choice(choices, false)
    }

    /// Pick any number of `choices`.
    pub fn multiple_choice(choices: Vec<RpChoice>) -> Self {
        Self::choice(choices, true)
    }

    /// A choice format with `questionType` and `answerStyle` kept in step.
    fn choice(choices: Vec<RpChoice>, multiple: bool) -> Self {
        let style = if multiple {
            "MultipleChoice"
        } else {
            "SingleChoice"
        };
        Self::Known(Box::new(KnownAnswerFormat::Choice {
            question_type: style.to_owned(),
            choices,
            answer_style: style.to_owned(),
        }))
    }

    /// Pick one of a row of images.
    pub fn image_choice(choices: Vec<RpImageChoice>) -> Self {
        Self::Known(Box::new(KnownAnswerFormat::ImageChoice {
            choices,
            question_type: image_choice_question_type(),
        }))
    }

    /// A whole number between `min` and `max`.
    pub fn integer(min: i64, max: i64, suffix: impl Into<String>) -> Self {
        Self::Known(Box::new(KnownAnswerFormat::Integer {
            min_value: min,
            max_value: max,
            suffix: suffix.into(),
            question_type: integer_question_type(),
        }))
    }

    /// A slider from `min` to `max` in `divisions` steps.
    pub fn slider(min: f64, max: f64, divisions: u32) -> Self {
        Self::Known(Box::new(KnownAnswerFormat::Slider {
            min_value: min,
            max_value: max,
            divisions,
            prefix: String::new(),
            suffix: String::new(),
            question_type: slider_question_type(),
        }))
    }

    /// Free text, with `hint` as the placeholder.
    pub fn text(hint: Option<String>) -> Self {
        Self::Known(Box::new(KnownAnswerFormat::Text {
            hint_text: hint,
            auto_focus: false,
            disable_helpers: false,
            question_type: text_question_type(),
        }))
    }

    /// A date and/or time. `style` is `"Date"`, `"TimeOfDay"` or
    /// `"DateAndTime"`, and is written to both fields that carry it.
    pub fn date_time(style: impl Into<String>) -> Self {
        let style = style.into();
        Self::Known(Box::new(KnownAnswerFormat::DateTime {
            question_type: style.clone(),
            date_time_answer_style: style,
        }))
    }

    /// The marker format a form step carries.
    pub fn form() -> Self {
        Self::Known(Box::new(KnownAnswerFormat::Form {
            question_type: form_question_type(),
        }))
    }

    /// A one-line summary for the editor.
    pub fn label(&self) -> String {
        let format = match self {
            Self::Known(format) => format.as_ref(),
            Self::Unknown(node) => return node.short_type().to_owned(),
        };
        match format {
            KnownAnswerFormat::Choice {
                choices,
                answer_style,
                ..
            } => format!("{answer_style}, {} options", choices.len()),
            KnownAnswerFormat::ImageChoice { choices, .. } => {
                format!("ImageChoice, {} images", choices.len())
            }
            KnownAnswerFormat::Integer {
                min_value,
                max_value,
                suffix,
                ..
            } => {
                if suffix.is_empty() {
                    format!("Integer {min_value}-{max_value}")
                } else {
                    format!("Integer {min_value}-{max_value} {suffix}")
                }
            }
            KnownAnswerFormat::Slider {
                min_value,
                max_value,
                divisions,
                ..
            } => format!("Scale {min_value}-{max_value} in {divisions}"),
            KnownAnswerFormat::Text { .. } => "Text".to_owned(),
            KnownAnswerFormat::DateTime {
                date_time_answer_style,
                ..
            } => date_time_answer_style.clone(),
            KnownAnswerFormat::Form { .. } => "Form".to_owned(),
        }
    }

    /// The options, for the choice formats the editor can edit in place.
    pub fn choices_mut(&mut self) -> Option<&mut Vec<RpChoice>> {
        match self {
            Self::Known(format) => match format.as_mut() {
                KnownAnswerFormat::Choice { choices, .. } => Some(choices),
                _ => None,
            },
            Self::Unknown(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
