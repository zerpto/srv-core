use validator::ValidationError;

use crate::helpers::parse_amount;
use crate::validation::failure;

/// A money amount written as a string: digits with at most two decimals,
/// never negative (see [`parse_amount`]). Code `invalid_amount`.
///
/// # Errors
///
/// `invalid_amount` when [`parse_amount`] rejects it.
pub fn amount(value: &str) -> Result<(), ValidationError> {
    parse_amount(value).map(|_| ()).ok_or_else(|| {
        failure(
            "invalid_amount",
            "Amount must be a number of zero or more, with at most two decimals.",
        )
    })
}
