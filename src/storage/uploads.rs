//! Reading the files of a multipart upload.

use axum::{body::Bytes, extract::Multipart, http::StatusCode};

use crate::{http::problem::Problem, storage::sniff::FileError};

/// The files of a multipart body sent under any of the field `names`. Other
/// fields are skipped; a service that needs them reads the multipart body
/// itself.
pub struct MultipartFiles<'a> {
    multipart: &'a mut Multipart,
    names: &'a [&'a str],
}

impl<'a> MultipartFiles<'a> {
    pub const fn new(multipart: &'a mut Multipart, names: &'a [&'a str]) -> Self {
        Self { multipart, names }
    }

    /// The files as `(filename, bytes)`.
    ///
    /// # Errors
    ///
    /// `400 invalid_upload` for a body that is not multipart, `422
    /// file_too_large` when it grew past the limit.
    pub async fn read(self) -> Result<Vec<(String, Bytes)>, Problem> {
        let mut files = Vec::new();
        while let Some(field) = self
            .multipart
            .next_field()
            .await
            .map_err(|err| Self::bad_upload(&err))?
        {
            let name = field.name().unwrap_or_default().to_string();
            if self.names.contains(&name.as_str()) {
                let filename = field.file_name().unwrap_or_default().to_string();
                let bytes = field.bytes().await.map_err(|err| Self::bad_upload(&err))?;
                files.push((filename, bytes));
            }
        }
        Ok(files)
    }

    /// A body that is not multipart, or that grew past the limit.
    fn bad_upload(err: &axum::extract::multipart::MultipartError) -> Problem {
        if err.status() == StatusCode::PAYLOAD_TOO_LARGE {
            FileError::TooLarge.problem()
        } else {
            Problem::bad_request(
                "invalid_upload",
                "The upload is not valid multipart form data.",
            )
            .field("file")
        }
    }
}
