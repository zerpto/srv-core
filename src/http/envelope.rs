use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Success envelope shared by every `/v1` response: the record in `data`,
/// plus a `meta` object that is always present.
#[derive(Debug, Serialize, Deserialize, TS)]
pub struct Envelope<D: TS, M: TS> {
    pub data: D,
    pub meta: M,
}

/// Error envelope shared by every `/v1` response: `{error: {...}}`, never
/// wrapped in `data`.
#[derive(Debug, Serialize, Deserialize, TS)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

/// The body of every `/v1` error: a message and the request id, plus
/// `details` on a failed validation. There is no `code` and no `field`;
/// clients go by the HTTP status.
#[derive(Debug, Serialize, Deserialize, TS)]
pub struct ErrorBody {
    /// Human-readable explanation. May change; do not parse.
    pub message: String,
    /// Quote this when contacting support.
    pub request_id: String,
    /// What was wrong, one entry per problem. Present only on a failed
    /// validation (`422`); every other error leaves it out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<Vec<ErrorDetail>>,
}

/// One problem found while validating a request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ErrorDetail {
    /// The request field the problem is about (`address.city`, `items[0].name`);
    /// `null` for a rule that spans several fields or the whole body.
    pub field: Option<String>,
    /// Human-readable explanation. May change; do not parse.
    pub message: String,
}

impl<D: TS + Serialize, M: TS + Serialize> Envelope<D, M> {
    /// The JSON response `{data, meta}` with the given status.
    #[must_use]
    pub fn respond(self, status: StatusCode) -> axum::response::Response {
        (status, Json(self)).into_response()
    }
}

/// Paging numbers and URLs of a page-numbered list (`meta.pagination`).
#[derive(Debug, Serialize, Deserialize, TS)]
pub struct PagePagination {
    #[ts(type = "number")]
    pub page: u64,
    #[ts(type = "number")]
    pub per_page: u64,
    #[ts(type = "number")]
    pub total_items: u64,
    #[ts(type = "number")]
    pub total_pages: u64,
    #[serde(rename = "self")]
    pub self_url: String,
    pub first: Option<String>,
    pub prev: Option<String>,
    pub next: Option<String>,
    pub last: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct PageListMeta {
    pub pagination: PagePagination,
}

impl PagePagination {
    /// Pagination for the page `meta` describes, with URLs that are the
    /// request's own path and query with `page` replaced.
    #[must_use]
    pub fn new(meta: &loco_rs::prelude::PagerMeta, uri: &axum::http::Uri) -> Self {
        let link = |page: u64| {
            let query: Vec<&str> = uri
                .query()
                .unwrap_or_default()
                .split('&')
                .filter(|pair| !pair.is_empty() && !pair.starts_with("page="))
                .collect();
            let mut query = query.join("&");
            if !query.is_empty() {
                query.push('&');
            }
            format!("{}?{query}page={page}", uri.path())
        };
        let last = meta.total_pages.max(1);
        Self {
            page: meta.page,
            per_page: meta.page_size,
            total_items: meta.total_items,
            total_pages: meta.total_pages,
            self_url: link(meta.page),
            first: Some(link(1)),
            prev: (meta.page > 1).then(|| link(meta.page - 1)),
            next: (meta.page < meta.total_pages).then(|| link(meta.page + 1)),
            last: Some(link(last)),
        }
    }
}

/// Paging of a cursor-paged list (`meta.pagination`).
#[derive(Debug, Serialize, Deserialize, TS)]
pub struct CursorPagination {
    /// Pass as `cursor` to fetch the next page; `null` on the last page.
    pub next_cursor: Option<String>,
    pub has_more: bool,
    #[ts(type = "number")]
    pub limit: u64,
    #[serde(rename = "self")]
    pub self_url: String,
    pub next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct CursorListMeta {
    pub pagination: CursorPagination,
}

impl CursorPagination {
    /// Pagination for a page that ended at `next_cursor`, with URLs that are
    /// the request's own path and query with `cursor` replaced.
    #[must_use]
    pub fn new(next_cursor: Option<String>, limit: u64, uri: &axum::http::Uri) -> Self {
        let without_cursor: Vec<&str> = uri
            .query()
            .unwrap_or_default()
            .split('&')
            .filter(|pair| !pair.is_empty() && !pair.starts_with("cursor="))
            .collect();
        let base = without_cursor.join("&");
        let next = next_cursor.as_ref().map(|cursor| {
            let sep = if base.is_empty() { "" } else { "&" };
            format!("{}?{base}{sep}cursor={cursor}", uri.path())
        });
        Self {
            has_more: next_cursor.is_some(),
            next_cursor,
            limit,
            self_url: uri.to_string(),
            next,
        }
    }
}

/// An empty `meta`, for responses whose operation defines none.
#[derive(Debug, Default, Serialize, Deserialize, TS)]
pub struct EmptyMeta {}
