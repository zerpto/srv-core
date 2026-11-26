use crate::http::problem::Problem;

/// Why a file was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum FileError {
    /// Not one of the types the upload takes.
    UnsupportedFileType,
    TooLarge,
}

impl FileError {
    /// The `422` for a refused file.
    #[must_use]
    pub fn problem(&self) -> Problem {
        match self {
            Self::UnsupportedFileType => Problem::unprocessable(
                "unsupported_file_type",
                "The file type is not accepted here.",
            )
            .field("file"),
            Self::TooLarge => {
                Problem::unprocessable("file_too_large", "The file is larger than allowed.")
                    .field("file")
            }
        }
    }
}
