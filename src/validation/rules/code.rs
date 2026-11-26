use validator::ValidationError;

use crate::validation::failure;

/// A code has at least this many characters, ignoring surrounding spaces.
pub const CODE_MIN_LENGTH: usize = 1;
/// A code has at most this many characters, ignoring surrounding spaces.
pub const CODE_MAX_LENGTH: usize = 32;

/// A short user-chosen code: 1 to 32 characters once trimmed. Code
/// `invalid_code`.
///
/// # Errors
///
/// `invalid_code` when blank or longer than 32 characters.
pub fn code(value: &str) -> Result<(), ValidationError> {
    let length = value.trim().chars().count();
    if (CODE_MIN_LENGTH..=CODE_MAX_LENGTH).contains(&length) {
        Ok(())
    } else {
        Err(failure(
            "invalid_code",
            "Code must be 1 to 32 characters long.",
        ))
    }
}
