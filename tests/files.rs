use zerpto_core::helpers::clean_filename;

#[test]
fn strips_folders_and_control_characters() {
    assert_eq!(clean_filename("../../etc/passwd", "file", "pdf"), "passwd");
    assert_eq!(
        clean_filename("C:\\docs\\bill.pdf", "file", "pdf"),
        "bill.pdf"
    );
    assert_eq!(clean_filename("a\u{0}b.pdf", "file", "pdf"), "ab.pdf");
}

#[test]
fn blank_names_fall_back() {
    assert_eq!(clean_filename("  ", "receipt", "jpg"), "receipt.jpg");
}
