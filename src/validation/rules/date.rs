use chrono::NaiveDate;
use validator::ValidationError;

use crate::validation::failure;

/// A calendar date written `YYYY-MM-DD`. Code `invalid_date`.
///
/// # Errors
///
/// `invalid_date` when it is not a real date in that format.
pub fn date(value: &str) -> Result<(), ValidationError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| {
            failure(
                "invalid_date",
                "Date must be a real date written YYYY-MM-DD.",
            )
        })
}
