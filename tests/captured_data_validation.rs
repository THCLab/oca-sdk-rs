use oca_sdk_rs::validator::validate;
use std::fs;
use std::path::Path;

use oca_bundle_semantics::controller::load_oca;

#[test]
fn validate_captured_data() -> Result<(), Box<dyn std::error::Error>> {
    let oca_bundle_path = Path::new("tests/assets/oca_bundle.json");
    assert!(oca_bundle_path.exists(), "Asset file not found!");
    let oca_bundle_str = fs::read_to_string(oca_bundle_path)?;

    let captured_data_path = Path::new("tests/assets/captured_data.json");
    assert!(captured_data_path.exists(), "Asset file not found!");
    let captured_data_str = fs::read_to_string(captured_data_path)?;

    let oca = load_oca(&mut oca_bundle_str.as_bytes()).unwrap();
    let result = validate(&oca, &captured_data_str).unwrap();
    assert_eq!(result.len(), 3);

    Ok(())
}
