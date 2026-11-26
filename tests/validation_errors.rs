use axum::{body::to_bytes, response::IntoResponse};
use serde::Deserialize;
use serde_json::{json, Value};
use validator::{Validate, ValidationError};
use zerpto_core::{
    http::problem::Problem,
    validation::{rules, ValidationDetails},
};

fn wrong_overall(_: &Whole) -> Result<(), ValidationError> {
    Err(ValidationError::new("custom").with_message("Item is wrong overall.".into()))
}

#[derive(Deserialize, Validate)]
struct Address {
    #[validate(custom(function = "rules::name"))]
    city: String,
    #[validate(length(min = 1))]
    street: String,
}

#[derive(Deserialize, Validate)]
struct Item {
    #[validate(custom(function = "rules::code"))]
    code: String,
}

#[derive(Deserialize, Validate)]
#[validate(schema(function = "never_ok", skip_on_field_errors = false))]
struct Order {
    #[validate(custom(function = "rules::name"))]
    name: String,
    #[validate(custom(function = "rules::code"))]
    code: String,
    #[validate(nested)]
    address: Address,
    #[validate(nested)]
    items: Vec<Item>,
}

fn never_ok(_: &Order) -> Result<(), ValidationError> {
    Err(ValidationError::new("whole").with_message("The order is inconsistent.".into()))
}

#[derive(Validate)]
#[validate(schema(function = "wrong_overall"))]
struct Whole {}

fn bad_order() -> Order {
    Order {
        name: String::new(),
        code: String::new(),
        address: Address {
            city: "x".into(),
            street: String::new(),
        },
        items: vec![
            Item { code: "ok".into() },
            Item {
                code: String::new(),
            },
        ],
    }
}

#[test]
fn details_are_sorted_by_field_with_fieldless_last() {
    let errors = bad_order().validate().unwrap_err();
    let got: Vec<(Option<String>, String)> = ValidationDetails::of(&errors)
        .into_iter()
        .map(|d| (d.field, d.message))
        .collect();
    let fields: Vec<Option<&str>> = got.iter().map(|(f, _)| f.as_deref()).collect();
    assert_eq!(
        fields,
        vec![
            Some("address.city"),
            Some("address.street"),
            Some("code"),
            Some("items[1].code"),
            Some("name"),
            None,
        ]
    );
    assert_eq!(got[2].1, "Code must be 1 to 32 characters long.");
    assert_eq!(got[4].1, "Name is required.");
    assert_eq!(got[5].1, "The order is inconsistent.");
}

#[test]
fn builtin_rule_without_message_gets_a_generic_one() {
    let errors = bad_order().validate().unwrap_err();
    let street = ValidationDetails::of(&errors)
        .into_iter()
        .find(|d| d.field.as_deref() == Some("address.street"));
    assert_eq!(street.unwrap().message, "Street is invalid.");
}

#[tokio::test]
async fn response_has_the_exact_shape() {
    let problem: Problem = bad_order().validate().unwrap_err().into();
    let response = problem.into_response();
    assert_eq!(response.status(), 422);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let error = &body["error"];
    assert_eq!(error["message"], "The given data was invalid.");
    assert_eq!(error["request_id"], "");
    assert!(error.get("code").is_none());
    assert!(error.get("field").is_none());
    assert_eq!(error.as_object().unwrap().len(), 3);
    assert_eq!(
        error["details"][0],
        json!({"field": "address.city", "message": "Name must be at least 2 characters long."})
    );
    assert_eq!(
        error["details"][5],
        json!({"field": null, "message": "The order is inconsistent."})
    );
    assert!(error["details"][0].get("code").is_none());
}

#[tokio::test]
async fn other_errors_have_only_message_and_request_id() {
    let response = Problem::not_found("Nothing here.").into_response();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        body,
        json!({"error": {"message": "Nothing here.", "request_id": ""}})
    );
}

#[test]
fn top_level_schema_error_has_a_null_field() {
    let errors = Whole {}.validate().unwrap_err();
    let got = ValidationDetails::of(&errors);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].field, None);
    assert_eq!(got[0].message, "Item is wrong overall.");
}
