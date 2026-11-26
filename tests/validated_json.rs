use axum::{
    body::{to_bytes, Body},
    extract::{FromRequest, Request},
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::{json, Value};
use validator::Validate;
use zerpto_core::validation::{rules, ValidatedJson};

#[derive(Debug, Deserialize, Validate)]
struct Create {
    #[validate(custom(function = "rules::code"))]
    code: String,
    #[validate(custom(function = "rules::name"))]
    name: String,
}

fn request(content_type: Option<&str>, body: &str) -> Request {
    let mut builder = Request::builder().method("POST").uri("/");
    if let Some(content_type) = content_type {
        builder = builder.header("content-type", content_type);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

async fn run(req: Request) -> Result<Create, (u16, Value)> {
    match ValidatedJson::<Create>::from_request(req, &()).await {
        Ok(ValidatedJson(value)) => Ok(value),
        Err(problem) => {
            let response = problem.into_response();
            let status = response.status().as_u16();
            let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            Err((status, serde_json::from_slice(&bytes).unwrap()))
        }
    }
}

#[tokio::test]
async fn a_valid_body_reaches_the_handler() {
    let ok = run(request(
        Some("application/json"),
        r#"{"code":"A1","name":"Bob"}"#,
    ))
    .await;
    assert_eq!(ok.unwrap().code, "A1");
}

#[tokio::test]
async fn malformed_json_is_400_invalid_json() {
    let (status, body) = run(request(Some("application/json"), "{not json"))
        .await
        .unwrap_err();
    assert_eq!(status, 400);
    assert!(body["error"].get("code").is_none());
    assert!(body["error"].get("details").is_none());
}

#[tokio::test]
async fn wrong_or_missing_content_type_is_400_invalid_json() {
    for content_type in [None, Some("text/plain")] {
        let (status, body) = run(request(content_type, r#"{"code":"A1","name":"Bob"}"#))
            .await
            .unwrap_err();
        assert_eq!(status, 400);
        assert!(body["error"].get("code").is_none());
    }
}

#[tokio::test]
async fn missing_field_is_422_with_one_fieldless_detail() {
    let (status, body) = run(request(Some("application/json"), r#"{"code":"A1"}"#))
        .await
        .unwrap_err();
    assert_eq!(status, 422);
    assert!(body["error"].get("code").is_none());
    let details = body["error"]["details"].as_array().unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(details[0]["field"], Value::Null);
}

#[tokio::test]
async fn wrong_type_is_422_validation_failed() {
    let (status, body) = run(request(
        Some("application/json"),
        r#"{"code":1,"name":"Bob"}"#,
    ))
    .await
    .unwrap_err();
    assert_eq!(status, 422);
    assert_eq!(body["error"]["details"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn failed_rules_are_422_with_sorted_details() {
    let (status, body) = run(request(
        Some("application/json"),
        r#"{"code":"","name":""}"#,
    ))
    .await
    .unwrap_err();
    assert_eq!(status, 422);
    assert_eq!(
        body["error"]["details"],
        json!([
            {"field": "code", "message": "Code must be 1 to 32 characters long."},
            {"field": "name", "message": "Name is required."}
        ])
    );
    assert_eq!(body["error"]["message"], "The given data was invalid.");
}
