use std::str::FromStr;

use sea_orm::prelude::Decimal;
use zerpto_core::helpers::{format_amount, format_price, format_quantity, parse_amount};

fn dec(text: &str) -> Decimal {
    Decimal::from_str(text).unwrap()
}

#[test]
fn quantities_drop_needless_decimals() {
    assert_eq!(format_quantity(&dec("1.00")), "1");
    assert_eq!(format_quantity(&dec("120.000")), "120");
    assert_eq!(format_quantity(&dec("2.50")), "2.5");
    assert_eq!(format_quantity(&dec("0.125")), "0.125");
}

#[test]
fn prices_keep_two_to_six_decimals() {
    assert_eq!(format_price(&dec("10")), "10.00");
    assert_eq!(format_price(&dec("10.5")), "10.50");
    assert_eq!(format_price(&dec("0.123456")), "0.123456");
}

#[test]
fn amounts_have_two_decimals() {
    assert_eq!(format_amount(&dec("10")), "10.00");
}

#[test]
fn parses_only_non_negative_amounts_with_two_decimals() {
    assert_eq!(parse_amount(" 12.34 "), Some(dec("12.34")));
    assert_eq!(parse_amount("12.345"), None);
    assert_eq!(parse_amount("-1"), None);
    assert_eq!(parse_amount("1e3"), None);
    assert_eq!(parse_amount(""), None);
}
