use std::borrow::Cow;

use validator::ValidationError;

/// A validation rule failure with an explicit stable `code` and a message.
#[must_use]
pub fn failure(code: &'static str, message: &'static str) -> ValidationError {
    ValidationError::new(code).with_message(Cow::Borrowed(message))
}
