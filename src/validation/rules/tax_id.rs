use validator::ValidationError;

use crate::validation::failure;

/// An NPWP has 15 digits, or 16 for the newer format built on the NIK. The
/// dots and dash it is usually written with are not counted.
const NPWP_DIGITS: [usize; 2] = [15, 16];

/// A tax id (NPWP): 15 or 16 digits, written with only digits, dots, dashes
/// and spaces. Code `invalid_tax_id`.
///
/// # Errors
///
/// `invalid_tax_id` otherwise.
pub fn tax_id(value: &str) -> Result<(), ValidationError> {
    let value = value.trim();
    let well_formed = value
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | ' '));
    let digits = value.chars().filter(char::is_ascii_digit).count();
    if well_formed && NPWP_DIGITS.contains(&digits) {
        Ok(())
    } else {
        Err(failure(
            "invalid_tax_id",
            "A tax id (NPWP) has 15 or 16 digits.",
        ))
    }
}
