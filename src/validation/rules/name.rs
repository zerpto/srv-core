use validator::ValidationError;

use crate::validation::failure;

/// A name has at least this many characters, ignoring surrounding spaces.
pub const NAME_MIN_LENGTH: usize = 2;

/// A person's or company's name: at least [`NAME_MIN_LENGTH`] characters once
/// trimmed. Code `invalid_name`.
///
/// # Errors
///
/// `invalid_name` when blank or too short.
pub fn name(value: &str) -> Result<(), ValidationError> {
    let length = value.trim().chars().count();
    if length == 0 {
        return Err(failure("invalid_name", "Name is required."));
    }
    if length < NAME_MIN_LENGTH {
        return Err(failure(
            "invalid_name",
            "Name must be at least 2 characters long.",
        ));
    }
    Ok(())
}
