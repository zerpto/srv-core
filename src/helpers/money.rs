//! How money and quantities are written in the API: always a string, never a
//! float. Every service must format them the same way.

use sea_orm::prelude::Decimal;

/// The only currency invoices are issued in.
pub const IDR: &str = "IDR";

/// Amounts have at most two decimals.
const AMOUNT_SCALE: u32 = 2;

/// A decimal as the API writes it: a string with two decimals, never a float.
#[must_use]
pub fn format_amount(amount: &Decimal) -> String {
    format!("{:.2}", amount.round_dp(2))
}

/// A price as the API writes it: two decimals at least, up to six when the
/// value needs them, trailing zeros beyond two dropped.
#[must_use]
pub fn format_price(price: &Decimal) -> String {
    let text = format!("{:.6}", price.round_dp(6));
    let trimmed = text.trim_end_matches('0');
    let decimals = trimmed.split_once('.').map_or(0, |(_, d)| d.len());
    if decimals >= 2 {
        trimmed.to_string()
    } else {
        format!("{:.2}", price.round_dp(6))
    }
}

/// A quantity for a person to read: a whole number with no decimals (`1`,
/// not `1.00`, which a reader may take for a hundred), and otherwise the
/// decimals it needs, up to six (`2.5`).
#[must_use]
pub fn format_quantity(quantity: &Decimal) -> String {
    quantity.round_dp(6).normalize().to_string()
}

/// Reads a money amount as written: digits with an optional fraction of at
/// most two decimals, never negative. `None` for anything else.
#[must_use]
pub fn parse_amount(text: &str) -> Option<Decimal> {
    let text = text.trim();
    let digits_only = !text.is_empty()
        && text.chars().all(|c| c.is_ascii_digit() || c == '.')
        && text.matches('.').count() <= 1;
    let value: Decimal = digits_only.then(|| text.parse().ok()).flatten()?;
    (value.scale() <= AMOUNT_SCALE || value == value.round_dp(AMOUNT_SCALE)).then_some(value)
}
