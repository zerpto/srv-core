//! Uploaded files.
//!
//! Covers telling a file's type by its bytes ([`sniff`]), storing and
//! reading the bytes ([`files::Files`]), signed download links
//! ([`file_links::FileLinks`]), reading multipart uploads
//! ([`uploads::MultipartFiles`]) and building the backend
//! ([`driver::StorageDriver`]). Naming files safely is `helpers::clean_filename`.

pub mod driver;
pub mod envelope_allowance;
pub mod file_links;
pub mod files;
pub mod sniff;
pub mod uploads;
