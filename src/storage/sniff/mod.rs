//! Telling a file's type by its first bytes (the client's own claim cannot be
//! trusted) and holding it to a size limit.

mod file_error;
mod max_video_size;
mod sniffed;

pub use file_error::FileError;
pub use max_video_size::MAX_VIDEO_SIZE;
pub use sniffed::Sniffed;
