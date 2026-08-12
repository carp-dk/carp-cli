// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Reading a field's value back out, once a form has been submitted.

use super::FieldValue;
use carp_protocol::duration::Micros;
use carp_protocol::trigger::TimeOfDay;

impl FieldValue {
/// The stored string of a text-like or picked field.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(text) | Self::Catalog { value: text, .. } => Some(text),
            Self::Choice { options, selected } => {
                options.get(*selected).map(|choice| choice.value.as_str())
            }
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Toggle(on) => Some(*on),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer { value, .. } => Some(*value),
            _ => None,
        }
    }

    pub fn as_duration(&self) -> Option<Micros> {
        match self {
            Self::Duration(duration) => Some(*duration),
            _ => None,
        }
    }

    pub fn as_time(&self) -> Option<TimeOfDay> {
        match self {
            Self::Time(time) => Some(*time),
            _ => None,
        }
    }

    pub fn as_set(&self) -> Option<&[String]> {
        match self {
            Self::CatalogSet { values, .. } => Some(values),
            _ => None,
        }
    }
}
