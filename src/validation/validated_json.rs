use axum::{
    extract::{rejection::JsonRejection, FromRequest, Json, Request},
    http::StatusCode,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::http::{envelope::ErrorDetail, problem::Problem};

/// A JSON request body that is checked before the handler runs: deserialized,
/// then validated with its `#[validate(...)]` rules.
///
/// ```ignore
/// async fn create(ValidatedJson(body): ValidatedJson<CreateThing>) -> ApiResult { ... }
/// ```
///
/// Failures are always the standard error envelope, never axum's plain text:
///
/// - invalid JSON syntax, or a missing / wrong `Content-Type`: `400`
/// - a missing field or a value of the wrong type: `422` with one detail (`field: null`)
/// - a rule that fails: `422`, one detail per problem
///
/// Works with any router state, `AppContext` included.
#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = Problem;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(rejection_to_problem)?;
        value.validate()?;
        Ok(Self(value))
    }
}

fn rejection_to_problem(rejection: JsonRejection) -> Problem {
    match rejection {
        JsonRejection::JsonDataError(_) => Problem::validation(vec![ErrorDetail {
            field: None,
            message: "The request body is missing a field or has a value of the wrong type."
                .to_string(),
        }]),
        JsonRejection::JsonSyntaxError(_) => {
            Problem::bad_request("invalid_json", "The request body is not valid JSON.")
        }
        JsonRejection::MissingJsonContentType(_) => Problem::bad_request(
            "invalid_json",
            "The request needs a Content-Type of application/json.",
        ),
        other if other.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            Problem::payload_too_large("payload_too_large", "The request body is too large.")
        }
        _ => Problem::bad_request("invalid_json", "The request body could not be read."),
    }
}
