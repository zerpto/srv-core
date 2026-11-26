//! Parsing shared by the list endpoints' query strings.

use chrono::{DateTime, NaiveDate, Utc};

use crate::http::problem::Problem;

/// One optional query parameter: its `value`, its `name` and the error `code`
/// to answer with when the value is malformed.
#[derive(Debug, Clone, Copy)]
pub struct QueryParam<'a> {
    value: Option<&'a str>,
    name: &'static str,
    code: &'static str,
}

impl<'a> QueryParam<'a> {
    #[must_use]
    pub const fn new(value: Option<&'a str>, name: &'static str, code: &'static str) -> Self {
        Self { value, name, code }
    }

    /// The value as an RFC 3339 timestamp.
    ///
    /// # Errors
    ///
    /// `400` with the code when the value is not a timestamp.
    pub fn timestamp(&self) -> Result<Option<DateTime<Utc>>, Problem> {
        self.value
            .map(|v| {
                DateTime::parse_from_rfc3339(v)
                    .map(|t| t.with_timezone(&Utc))
                    .map_err(|_| {
                        Problem::bad_request(
                            self.code,
                            format!("`{}` must be an RFC 3339 timestamp.", self.name),
                        )
                        .field(self.name)
                    })
            })
            .transpose()
    }

    /// The value as a `YYYY-MM-DD` date.
    ///
    /// # Errors
    ///
    /// `400` with the code when the value is not a date.
    pub fn date(&self) -> Result<Option<NaiveDate>, Problem> {
        self.value
            .map(|v| {
                NaiveDate::parse_from_str(v, "%Y-%m-%d").map_err(|_| {
                    Problem::bad_request(self.code, format!("`{}` must be a date.", self.name))
                        .field(self.name)
                })
            })
            .transpose()
    }
}
