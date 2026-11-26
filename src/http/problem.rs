use axum::{extract::Request, http::StatusCode, middleware::Next};
use loco_rs::{controller::middleware::request_id::LocoRequestId, model::ModelError, prelude::*};

use crate::http::envelope::{ErrorBody, ErrorDetail, ErrorEnvelope};

tokio::task_local! {
    /// The current request's id, set by [`with_request_id`] for the duration
    /// of the request so errors can quote it without every handler passing it.
    static REQUEST_ID: String;
}

/// Middleware for the `/v1` routes: makes the request id (from Loco's
/// request-id middleware) available to [`Problem`].
pub async fn with_request_id(request: Request, next: Next) -> Response {
    let id = request_id(&request);
    REQUEST_ID.scope(id, next.run(request)).await
}

fn request_id(request: &Request) -> String {
    request
        .extensions()
        .get::<LocoRequestId>()
        .map(|id| id.get().to_string())
        .unwrap_or_default()
}

/// Router fallback: an unmatched route answers with the `404` error envelope
/// instead of an empty or HTML body. Attached by `initializers::NotFound`.
pub async fn not_found_fallback(request: Request) -> Response {
    let id = request_id(&request);
    REQUEST_ID
        .scope(id, async {
            Problem::not_found("The requested resource was not found.").into_response()
        })
        .await
}

/// A `/v1` error, rendered as the contract's `{error: {message, request_id}}`
/// envelope (plus `details` on a validation failure). Return it as the `Err`
/// side of a handler.
#[derive(Debug)]
pub struct Problem {
    status: StatusCode,
    code: &'static str,
    message: String,
    field: Option<&'static str>,
    details: Option<Vec<ErrorDetail>>,
}

impl Problem {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            field: None,
            details: None,
        }
    }

    #[must_use]
    pub fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    #[must_use]
    pub fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, code, message)
    }

    #[must_use]
    pub fn forbidden(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, code, message)
    }

    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::not_found_with("not_found", message)
    }

    #[must_use]
    pub fn not_found_with(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, code, message)
    }

    #[must_use]
    pub fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, message)
    }

    #[must_use]
    pub fn unprocessable(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, code, message)
    }

    #[must_use]
    pub fn too_many_requests(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::TOO_MANY_REQUESTS, code, message)
    }

    #[must_use]
    pub fn service_unavailable(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, code, message)
    }

    #[must_use]
    pub fn payload_too_large(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::PAYLOAD_TOO_LARGE, code, message)
    }

    /// `422` with one entry per problem in `details`; clients tell it apart by
    /// the status and the `details` array.
    #[must_use]
    pub fn validation(details: Vec<ErrorDetail>) -> Self {
        let mut problem = Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "The given data was invalid.",
        );
        problem.details = Some(details);
        problem
    }

    /// Names the request field at fault. Not sent to the client (the response
    /// has no `field`); it is only logged. Use `details` for per-field errors.
    #[must_use]
    pub const fn field(mut self, field: &'static str) -> Self {
        self.field = Some(field);
        self
    }
}

/// Unexpected failures: log the cause, expose nothing about it.
fn internal(err: &dyn std::fmt::Display) -> Problem {
    tracing::error!(error = %err, "unhandled error on /v1");
    // A no-op until `initializers::sentry` has started a client.
    sentry::with_scope(
        |scope| {
            scope.set_tag(
                "request_id",
                REQUEST_ID.try_with(Clone::clone).unwrap_or_default(),
            );
        },
        || sentry::capture_message(&err.to_string(), sentry::Level::Error),
    );
    Problem::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal_error",
        "Something went wrong.",
    )
}

impl From<Error> for Problem {
    fn from(err: Error) -> Self {
        internal(&err)
    }
}

impl From<ModelError> for Problem {
    fn from(err: ModelError) -> Self {
        internal(&err)
    }
}

impl From<sea_orm::DbErr> for Problem {
    fn from(err: sea_orm::DbErr) -> Self {
        internal(&err)
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        // The response carries no `code` or `field`; keep them in the logs.
        tracing::debug!(
            status = self.status.as_u16(),
            code = self.code,
            field = ?self.field,
            "responding with an error"
        );
        let body = ErrorEnvelope {
            error: ErrorBody {
                message: self.message,
                request_id: REQUEST_ID.try_with(Clone::clone).unwrap_or_default(),
                details: self.details,
            },
        };
        (self.status, Json(body)).into_response()
    }
}

/// Result of a `/v1` handler.
pub type ApiResult<T = Response> = std::result::Result<T, Problem>;
