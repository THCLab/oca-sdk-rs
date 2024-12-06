use oca_sdk_rs::{
    validator::{validate, DataValidationStatus},
    load_oca, OCAValidator,
    WithInfo,
};
use std::fs;
use std::path::Path;

#[test]
fn validate_captured_data() -> Result<(), Box<dyn std::error::Error>> {
    let oca_bundle_path = Path::new("tests/assets/oca_bundle.json");
    assert!(oca_bundle_path.exists(), "Asset file not found!");
    let oca_bundle_str = fs::read_to_string(oca_bundle_path)?;

    let captured_data_path = Path::new("tests/assets/captured_data.json");
    assert!(captured_data_path.exists(), "Asset file not found!");
    let captured_data_str = fs::read_to_string(captured_data_path)?;

    let oca = load_oca(&mut oca_bundle_str.as_bytes()).unwrap();

    println!("{:?}", oca.info().meta.unwrap());
    println!(
        "{:?}",
        oca.info()
            .attributes
            .keys()
            .map(|name| { name.to_string() })
            .collect::<Vec<String>>()
    );

    let oca_validator = OCAValidator::new();
    let oca_validation_result = oca_validator.validate(&oca);
    assert!(matches!(oca_validation_result, Ok(())));

    let data_validation_result = validate(&oca, &captured_data_str).unwrap();
    assert!(matches!(
        data_validation_result,
        DataValidationStatus::Invalid(_)
    ));
    if let DataValidationStatus::Invalid(errors) = data_validation_result {
        assert_eq!(errors.len(), 3);
    }

    Ok(())
}
