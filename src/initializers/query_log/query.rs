/// One SQL statement a request ran.
#[derive(Debug, Clone)]
pub struct Query {
    pub sql: String,
    pub ms: f64,
    pub failed: bool,
}
