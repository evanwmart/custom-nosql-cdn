#[test]
fn test_checksum_validation() {
    use crate::database::Database;

    let db = Database::new("test.db".to_string());
    let data = b"test data";
    let checksum = db.generate_checksum(data);

    assert!(db.validate_checksum(data, &checksum));
}
