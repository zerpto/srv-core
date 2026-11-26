//! Development aid: report the SQL a request ran inside the response `meta`.
//!
//! Only active in the development environment. The database connection is
//! given a metric callback ([`QueryReport::record`]) that appends every
//! statement to a per-request list, and [`QueryReport::collect`] attaches that
//! list to the JSON response as `meta.debug` plus an `X-Query-Count` header.
//!
//! SQL is reported with its `$n` placeholders; bound values are never
//! included because they can hold password hashes, tokens or card data.

mod initializer;
mod query;
mod query_report;

pub use initializer::QueryLog;
pub use query::Query;
pub use query_report::QueryReport;
