//! Standalone helpers: functions that need no shared state, so they are not
//! methods of a struct. Nothing here depends on a Cargo feature.
//!
//! - `money`: how amounts, prices and quantities are written and read.
//! - `text`: cleaning text and file names from a client.
//! - `response`: the success envelope and file downloads.
//! - `channel`: a lazy gRPC channel to a peer service.

mod channel;
mod money;
mod response;
mod text;

pub use channel::channel;
pub use money::{format_amount, format_price, format_quantity, parse_amount, IDR};
pub use response::{attachment_response, inline_response, json};
pub use text::{clean_filename, non_blank};
