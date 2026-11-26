use serde_json::json;
use zerpto_core::initializers::query_log::{Query, QueryReport};

fn query(sql: &str, ms: f64) -> Query {
    Query {
        sql: sql.to_string(),
        ms,
        failed: false,
    }
}

#[test]
fn the_report_goes_into_meta() {
    let mut body = json!({"data": {"id": 1}, "meta": {"request_id": "r"}});
    let report = QueryReport::new(vec![query("SELECT 1", 1.0), query("SELECT 2", 2.0)]);
    assert!(report.attach(&mut body));
    assert_eq!(body["meta"]["request_id"], "r");
    assert_eq!(body["meta"]["debug"]["query_count"], 2);
    assert_eq!(body["meta"]["debug"]["query_ms"], 3.0);
    assert_eq!(body["meta"]["debug"]["queries"][1]["sql"], "SELECT 2");
}

#[test]
fn bodies_without_a_meta_object_are_left_alone() {
    let report = QueryReport::new(vec![query("SELECT 1", 1.0)]);
    for mut body in [
        json!({"error": {"message": "x"}}),
        json!({"data": 1, "meta": null}),
        json!([1, 2]),
    ] {
        let before = body.clone();
        assert!(!report.attach(&mut body));
        assert_eq!(body, before);
    }
}
