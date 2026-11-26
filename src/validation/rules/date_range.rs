use chrono::NaiveDate;
use validator::ValidationError;

use crate::validation::failure;

/// For a struct-level (`schema`) rule: `end` must not be before `start`.
/// Code `invalid_date_range`.
///
/// # Errors
///
/// `invalid_date_range` when `end` is before `start`.
pub fn date_range(start: NaiveDate, end: NaiveDate) -> Result<(), ValidationError> {
    if end < start {
        Err(failure(
            "invalid_date_range",
            "The end date must not be before the start date.",
        ))
    } else {
        Ok(())
    }
}
