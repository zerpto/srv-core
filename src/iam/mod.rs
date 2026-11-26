//! The IAM service as seen from another service (feature `iam`):
//! its authorization contract and an [`IamAuthorizer`] that plugs into
//! `http::authorizer::Authorizer`. Its address comes from `settings.iam`.

pub mod iam_authorizer;
pub mod iam_settings;
pub mod pb;

pub use iam_authorizer::IamAuthorizer;
pub use iam_settings::IamSettings;
