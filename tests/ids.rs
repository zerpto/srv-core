use uuid::Uuid;
use zerpto_core::http::ids::ApiIds;

const USER: &str = "usr";
const TENANT: &str = "ten";

#[test]
fn round_trips_an_api_id() {
    let id = Uuid::now_v7();
    let tenants = ApiIds::new(TENANT);
    let api = tenants.encode(&id);
    assert_eq!(api, format!("ten-{id}"));
    assert_eq!(tenants.parse(&api), Some(id));
}

#[test]
fn rejects_a_different_prefix_or_a_malformed_uuid() {
    let tenants = ApiIds::new(TENANT);
    let api = tenants.encode(&Uuid::now_v7());
    assert_eq!(ApiIds::new(USER).parse(&api), None);
    assert_eq!(tenants.parse("ten-not-a-uuid"), None);
    assert_eq!(tenants.parse("ten"), None);
}

#[test]
fn a_bad_id_in_a_handler_is_not_found() {
    assert!(ApiIds::new(TENANT).require("ten-not-a-uuid").is_err());
}
