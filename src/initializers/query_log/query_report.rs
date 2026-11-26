use std::sync::Mutex;

use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use sea_orm::metric::Info;
use serde_json::{json, Value};

use super::Query;

/// Largest JSON body the middleware will buffer to add the report.
const MAX_BODY: usize = 64 * 1024 * 1024;

tokio::task_local! {
    static QUERIES: Mutex<Vec<Query>>;
}

/// The statements one request ran, and how they are reported.
#[derive(Debug, Default)]
pub struct QueryReport {
    queries: Vec<Query>,
}

impl QueryReport {
    #[must_use]
    pub const fn new(queries: Vec<Query>) -> Self {
        Self { queries }
    }

    /// Metric callback for the database connection. Does nothing for queries
    /// that run outside a request (boot, workers, spawned tasks).
    pub fn record(info: &Info<'_>) {
        let _ = QUERIES.try_with(|queries| {
            if let Ok(mut queries) = queries.lock() {
                queries.push(Query {
                    sql: info.statement.sql.clone(),
                    ms: info.elapsed.as_secs_f64() * 1000.0,
                    failed: info.failed,
                });
            }
        });
    }

    /// Middleware: run the request with a fresh query list, then report it.
    pub async fn collect(request: Request, next: Next) -> Response {
        let (response, queries) = QUERIES
            .scope(Mutex::new(Vec::new()), async {
                let response = next.run(request).await;
                // Take the list out before the scope (and the list) ends.
                let queries = QUERIES.with(|queries| {
                    queries
                        .lock()
                        .map(|mut list| std::mem::take(&mut *list))
                        .unwrap_or_default()
                });
                (response, queries)
            })
            .await;
        Self::new(queries).respond(response).await
    }

    /// Adds `X-Query-Count` to the response and, for a JSON body with a
    /// `meta` object, the full report under `meta.debug`.
    pub async fn respond(&self, response: Response) -> Response {
        let (mut parts, body) = response.into_parts();
        parts
            .headers
            .insert("x-query-count", HeaderValue::from(self.queries.len()));

        let is_json = parts
            .headers
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("application/json"));
        if !is_json {
            return Response::from_parts(parts, body);
        }

        let Ok(bytes) = to_bytes(body, MAX_BODY).await else {
            parts.status = StatusCode::INTERNAL_SERVER_ERROR;
            return Response::from_parts(parts, Body::empty());
        };
        let mut value = serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null);
        let body = if self.attach(&mut value) {
            parts.headers.remove(header::CONTENT_LENGTH);
            Body::from(value.to_string())
        } else {
            Body::from(bytes)
        };
        Response::from_parts(parts, body)
    }

    /// Puts the report in `value.meta.debug`. Returns whether `value` was
    /// changed: bodies that are not an object with an object `meta` are left
    /// alone.
    pub fn attach(&self, value: &mut Value) -> bool {
        let Some(meta) = value.get_mut("meta").and_then(Value::as_object_mut) else {
            return false;
        };
        let total: f64 = self.queries.iter().map(|query| query.ms).sum();
        meta.insert(
            "debug".to_string(),
            json!({
                "query_count": self.queries.len(),
                "query_ms": (total * 100.0).round() / 100.0,
                "queries": self.queries
                    .iter()
                    .map(|query| json!({
                        "sql": query.sql,
                        "ms": (query.ms * 100.0).round() / 100.0,
                        "failed": query.failed,
                    }))
                    .collect::<Vec<_>>(),
            }),
        );
        true
    }
}
