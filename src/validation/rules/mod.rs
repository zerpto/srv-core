//! Shared validation rules for `#[validate(custom(function = "..."))]`. Every
//! rule sets an explicit, stable `code` and message on the error it returns,
//! never the crate's default rule-name code.
//!
//! ```ignore
//! #[derive(Deserialize, Validate)]
//! struct CreateThing {
//!     #[validate(custom(function = "zerpto_core::validation::rules::code"))]
//!     code: String,
//! }
//! ```

mod amount;
mod code;
mod date;
mod date_range;
mod name;
mod tax_id;

pub use amount::amount;
pub use code::{code, CODE_MAX_LENGTH, CODE_MIN_LENGTH};
pub use date::date;
pub use date_range::date_range;
pub use name::{name, NAME_MIN_LENGTH};
pub use tax_id::tax_id;
