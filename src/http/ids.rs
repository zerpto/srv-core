use uuid::Uuid;

use super::problem::Problem;

// The core registers no prefixes: a prefix names a kind of record, which is
// the owning service's knowledge. Each service keeps its own consts, and the
// list of all prefixes (checked for collisions across services) lives in the
// `com.zerpto.openapi` repo.

/// The API ids of one kind of record: `<prefix>-<uuid>`, e.g.
/// `inv-0191a3c2-7b1e-7f4a-8c3d-2e5f6a7b8c9d`. Use it for every id that leaves
/// the service in an HTTP body, URL or query string.
///
/// ```ignore
/// let invoices = ApiIds::new("inv");
/// let api = invoices.encode(&record.id);
/// let id = invoices.require(&path_id)?;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ApiIds<'a> {
    prefix: &'a str,
}

impl<'a> ApiIds<'a> {
    /// The ids with this `prefix`; the owning service defines the prefix.
    #[must_use]
    pub const fn new(prefix: &'a str) -> Self {
        Self { prefix }
    }

    /// The API form of a record's `id`.
    #[must_use]
    pub fn encode(&self, id: &Uuid) -> String {
        format!("{}-{id}", self.prefix)
    }

    /// Reads the `id` back out of an API id; `None` for a different prefix or
    /// a malformed uuid.
    #[must_use]
    pub fn parse(&self, api_id: &str) -> Option<Uuid> {
        Uuid::parse_str(api_id.strip_prefix(self.prefix)?.strip_prefix('-')?).ok()
    }

    /// [`ApiIds::parse`] for a handler: an id that is not one of this kind
    /// cannot name a record, so it is a `404 not_found`, not a `400`.
    ///
    /// # Errors
    ///
    /// `404 not_found` for a different prefix or a malformed uuid.
    pub fn require(&self, api_id: &str) -> Result<Uuid, Problem> {
        self.parse(api_id)
            .ok_or_else(|| Problem::not_found("Not found."))
    }
}
