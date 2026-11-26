use chrono::NaiveDate;
use zerpto_core::validation::rules;

fn code_of(result: Result<(), validator::ValidationError>) -> Option<String> {
    result.err().map(|e| e.code.to_string())
}

fn message_of(result: Result<(), validator::ValidationError>) -> Option<String> {
    result.err().and_then(|e| e.message.map(|m| m.to_string()))
}

#[test]
fn name_needs_two_characters_once_trimmed() {
    assert!(rules::name("Al").is_ok());
    assert!(rules::name("  Al  ").is_ok());
    assert_eq!(code_of(rules::name("")), Some("invalid_name".into()));
    assert_eq!(
        message_of(rules::name("   ")),
        Some("Name is required.".into())
    );
    assert_eq!(
        message_of(rules::name(" A ")),
        Some("Name must be at least 2 characters long.".into())
    );
}

#[test]
fn code_is_one_to_thirty_two_characters() {
    assert!(rules::code("A").is_ok());
    assert!(rules::code(&"x".repeat(32)).is_ok());
    assert_eq!(code_of(rules::code("")), Some("invalid_code".into()));
    assert_eq!(code_of(rules::code("   ")), Some("invalid_code".into()));
    assert_eq!(
        message_of(rules::code(&"x".repeat(33))),
        Some("Code must be 1 to 32 characters long.".into())
    );
}

#[test]
fn amount_reuses_the_money_parser() {
    assert!(rules::amount("12.34").is_ok());
    assert!(rules::amount("0").is_ok());
    for bad in ["12.345", "-1", "1e3", "", "abc"] {
        assert_eq!(
            code_of(rules::amount(bad)),
            Some("invalid_amount".into()),
            "{bad}"
        );
    }
}

#[test]
fn tax_id_has_fifteen_or_sixteen_digits() {
    assert!(rules::tax_id("01.234.567.8-901.234").is_ok());
    assert!(rules::tax_id("012345678901234").is_ok());
    assert!(rules::tax_id("0123456789012345").is_ok());
    for bad in ["12345", "01234567890123456", "01234567890123A", ""] {
        assert_eq!(
            code_of(rules::tax_id(bad)),
            Some("invalid_tax_id".into()),
            "{bad}"
        );
    }
}

#[test]
fn date_must_be_a_real_iso_date() {
    assert!(rules::date("2026-11-21").is_ok());
    for bad in ["2026-02-30", "21-11-2026", "", "2026/11/21"] {
        assert_eq!(
            code_of(rules::date(bad)),
            Some("invalid_date".into()),
            "{bad}"
        );
    }
}

#[test]
fn date_range_end_is_not_before_start() {
    let day = |d| NaiveDate::from_ymd_opt(2026, 11, d).unwrap();
    assert!(rules::date_range(day(1), day(1)).is_ok());
    assert!(rules::date_range(day(1), day(2)).is_ok());
    assert_eq!(
        code_of(rules::date_range(day(2), day(1))),
        Some("invalid_date_range".into())
    );
}
