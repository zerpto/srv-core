#![cfg(feature = "storage")]

use zerpto_core::storage::sniff::{FileError, Sniffed};

#[test]
fn recognizes_a_file_by_its_bytes_not_its_name() {
    assert_eq!(Sniffed::document(b"%PDF-1.7 ...").unwrap().extension, "pdf");
    assert_eq!(
        Sniffed::document(&[0xFF, 0xD8, 0xFF, 0xE0])
            .unwrap()
            .extension,
        "jpg"
    );
    assert!(Sniffed::document(b"MZ not a document").is_none());
}

#[test]
fn rejects_unknown_types_and_oversized_files() {
    assert_eq!(
        Sniffed::check_attachment(b"plain text", 10).unwrap_err(),
        FileError::UnsupportedFileType
    );
    assert_eq!(
        Sniffed::check_attachment(b"%PDF-1.7 0123456789", 10).unwrap_err(),
        FileError::TooLarge
    );
    assert!(Sniffed::check_attachment(b"%PDF-1.7", 1024).is_ok());
}
