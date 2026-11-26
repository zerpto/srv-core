//! Building HTTP responses: the `{data, meta}` success envelope and file
//! downloads.

use axum::{
    body::Bytes,
    http::{header, StatusCode},
};
use loco_rs::prelude::*;
use ts_rs::TS;

use crate::http::envelope::Envelope;

/// A success response: `{data, meta}` with the given status.
pub fn json<D: TS + serde::Serialize, M: TS + serde::Serialize>(
    status: StatusCode,
    data: D,
    meta: M,
) -> Response {
    Envelope { data, meta }.respond(status)
}

/// The file as the response body, shown in the browser where it can be.
#[must_use]
pub fn inline_response(filename: &str, content_type: &str, bytes: Bytes) -> Response {
    file_response("inline", filename, content_type, bytes)
}

/// The file as the response body, saved by the browser rather than shown.
#[must_use]
pub fn attachment_response(filename: &str, content_type: &str, bytes: Vec<u8>) -> Response {
    file_response("attachment", filename, content_type, bytes)
}

fn file_response(
    disposition: &str,
    filename: &str,
    content_type: &str,
    body: impl IntoResponse,
) -> Response {
    let filename = filename.replace(['"', '\\'], "_");
    (
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("{disposition}; filename=\"{filename}\""),
            ),
        ],
        body,
    )
        .into_response()
}
