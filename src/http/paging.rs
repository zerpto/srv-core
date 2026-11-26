use loco_rs::model::query::PaginationQuery;
use serde::Deserialize;

use crate::http::problem::Problem;

const DEFAULT_PER_PAGE: u64 = 25;
const MAX_PER_PAGE: u64 = 100;

/// `?page=` and `?per_page=` of a page-numbered list.
#[derive(Debug, Default, Deserialize)]
pub struct Paging {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl Paging {
    /// # Errors
    ///
    /// `400` when `page` is 0 or `per_page` is outside 1 to 100.
    pub fn resolve(&self) -> Result<PaginationQuery, Problem> {
        let per_page = self.per_page.unwrap_or(DEFAULT_PER_PAGE);
        if !(1..=MAX_PER_PAGE).contains(&per_page) {
            return Err(Problem::bad_request(
                "invalid_per_page",
                format!("`per_page` must be between 1 and {MAX_PER_PAGE}."),
            )
            .field("per_page"));
        }
        let page = self.page.unwrap_or(1);
        if page < 1 {
            return Err(
                Problem::bad_request("invalid_page", "`page` must be 1 or more.").field("page"),
            );
        }
        Ok(PaginationQuery {
            page,
            page_size: per_page,
        })
    }
}

const DEFAULT_LIMIT: u64 = 50;

/// `?limit=` and `?cursor=` of a cursor-paged list.
#[derive(Debug, Default, Deserialize)]
pub struct CursorPaging {
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

impl CursorPaging {
    /// # Errors
    ///
    /// `400` when `limit` is outside 1 to 100.
    pub fn limit(&self) -> Result<u64, Problem> {
        let limit = self.limit.unwrap_or(DEFAULT_LIMIT);
        if (1..=MAX_PER_PAGE).contains(&limit) {
            Ok(limit)
        } else {
            Err(Problem::bad_request(
                "invalid_limit",
                format!("`limit` must be between 1 and {MAX_PER_PAGE}."),
            )
            .field("limit"))
        }
    }
}
