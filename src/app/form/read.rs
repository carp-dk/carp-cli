// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! Reading a submitted form's values by field key.
//!
//! The keys are the contract between [`super::build`] and [`super::apply`];
//! these are what `apply` reads them through.

use super::{FieldValue, Form};

impl Form {
/// A field's value by key, for reading a submitted form.
    pub fn value(&self, key: &str) -> Option<&FieldValue> {
        self.fields
            .iter()
            .find(|field| field.key == key)
            .map(|field| &field.value)
    }

    /// A text field's value, or the empty string when absent.
    pub fn text(&self, key: &str) -> String {
        self.value(key)
            .and_then(FieldValue::as_str)
            .unwrap_or_default()
            .to_owned()
    }

    pub fn flag(&self, key: &str) -> bool {
        self.value(key).and_then(FieldValue::as_bool).unwrap_or(false)
    }

    pub fn integer(&self, key: &str) -> Option<i64> {
        self.value(key).and_then(FieldValue::as_integer)
    }

    pub fn duration(&self, key: &str) -> Option<carp_protocol::Micros> {
        self.value(key).and_then(FieldValue::as_duration)
    }

    pub fn time(&self, key: &str) -> Option<carp_protocol::trigger::TimeOfDay> {
        self.value(key).and_then(FieldValue::as_time)
    }

    pub fn set(&self, key: &str) -> Vec<String> {
        self.value(key)
            .and_then(FieldValue::as_set)
            .unwrap_or_default()
            .to_vec()
}
}
