use std::borrow::Cow;

use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};

use crate::http::{envelope::ErrorDetail, problem::Problem};

/// Where the validator derive files struct-level (`schema`) errors.
const SCHEMA_KEY: &str = "__all__";

/// Turns the validator's unordered error tree into response details.
#[derive(Debug, Default)]
pub struct ValidationDetails {
    details: Vec<ErrorDetail>,
}

impl ValidationDetails {
    /// The problems in `errors` as response details: nested structs as
    /// `parent.child`, list items as `items[0].name`, sorted by field name,
    /// then message. Problems with no field (a rule across fields, or the
    /// whole body) come last with `field: null`; a struct-level error of a
    /// nested struct carries that struct's path.
    #[must_use]
    pub fn of(errors: &ValidationErrors) -> Vec<ErrorDetail> {
        let mut collected = Self::default();
        collected.collect("", errors);
        collected.details.sort_by(|a, b| {
            (a.field.is_none(), &a.field, &a.message).cmp(&(
                b.field.is_none(),
                &b.field,
                &b.message,
            ))
        });
        collected.details
    }

    fn collect(&mut self, prefix: &str, errors: &ValidationErrors) {
        for (name, kind) in errors.errors() {
            let path = if name == SCHEMA_KEY {
                prefix.to_string()
            } else if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{prefix}.{name}")
            };
            match kind {
                ValidationErrorsKind::Field(list) => {
                    for error in list {
                        self.details.push(ErrorDetail {
                            field: (!path.is_empty()).then(|| path.clone()),
                            message: Self::message(error, &path),
                        });
                    }
                }
                ValidationErrorsKind::Struct(inner) => self.collect(&path, inner),
                ValidationErrorsKind::List(items) => {
                    for (index, inner) in items {
                        self.collect(&format!("{path}[{index}]"), inner);
                    }
                }
            }
        }
    }

    /// The rule's own message; for a built-in rule without one, "<Field> is
    /// invalid." from the last part of the field path.
    fn message(error: &ValidationError, path: &str) -> String {
        if let Some(message) = &error.message {
            return Cow::into_owned(message.clone());
        }
        let leaf = path.rsplit('.').next().unwrap_or_default();
        let leaf = leaf.split('[').next().unwrap_or_default().replace('_', " ");
        let mut chars = leaf.chars();
        chars.next().map_or_else(
            || "The given data was invalid.".to_string(),
            |first| format!("{}{} is invalid.", first.to_uppercase(), chars.as_str()),
        )
    }
}

impl From<ValidationErrors> for Problem {
    fn from(errors: ValidationErrors) -> Self {
        Self::validation(ValidationDetails::of(&errors))
    }
}
