use oca_sdk_rs::{
    build_from_ocafile,
    data_validator::{validate_data, DataValidationStatus},
    load_oca, load_oca_semantics, validate_semantics, BundleElement, SemanticValidationStatus,
    WithInfo,
};
use std::fs;
use std::path::Path;

#[test]
fn building_from_ocafile() -> Result<(), Box<dyn std::error::Error>> {
    let ocafile_path = Path::new("tests/assets/semantics/entrance_credential.ocafile");
    assert!(ocafile_path.exists(), "Asset file not found!");
    let ocafile_str = fs::read_to_string(ocafile_path)?;

    let bundle_element = build_from_ocafile(ocafile_str).unwrap();
    assert!(matches!(bundle_element, BundleElement::Structural(_)));
    if let BundleElement::Structural(structural_bundle) = bundle_element {
        assert_eq!(
            structural_bundle.said.unwrap().to_string(),
            "EKHBds6myKVIsQuT7Zr23M8Xk_gwq-2SaDRUprvqOXxa"
        );
    }

    Ok(())
}

#[test]
fn validate_oca_bundle_semantics() -> Result<(), Box<dyn std::error::Error>> {
    let oca_bundle_path = Path::new("tests/assets/semantics/oca_bundle.json");
    assert!(oca_bundle_path.exists(), "Asset file not found!");
    let oca_bundle_str = fs::read_to_string(oca_bundle_path)?;

    let oca_bundle = load_oca(&mut oca_bundle_str.as_bytes()).unwrap();

    let bundle_info = oca_bundle.info();
    println!("{:?}", bundle_info.meta);
    println!("{:?}", bundle_info.attribute("name"));
    println!("{:?}", bundle_info.links);

    let structural_bundle = oca_bundle.structural.unwrap();
    let semantics_validation_status = validate_semantics(&structural_bundle).unwrap();
    assert!(matches!(
        semantics_validation_status,
        SemanticValidationStatus::Valid
    ));

    Ok(())
}

#[test]
fn validate_captured_data() -> Result<(), Box<dyn std::error::Error>> {
    let captured_data_path = Path::new("tests/assets/data/nested.json");
    assert!(captured_data_path.exists(), "Asset file not found!");
    let data_str = fs::read_to_string(captured_data_path)?;
    let data = serde_json::from_str(&data_str)?;

    let structural_bundle_path = Path::new("tests/assets/semantics/structural_bundle.json");
    assert!(structural_bundle_path.exists(), "Asset file not found!");
    let structural_bundle_str = fs::read_to_string(structural_bundle_path)?;

    let structural_bundle = load_oca_semantics(&mut structural_bundle_str.as_bytes()).unwrap();

    let semantics_validation_status = validate_semantics(&structural_bundle).unwrap();
    assert!(matches!(
        semantics_validation_status,
        SemanticValidationStatus::Valid
    ));

    let data_validation_status = validate_data(&structural_bundle, &data).unwrap();
    assert!(matches!(
        data_validation_status,
        DataValidationStatus::Invalid(_)
    ));
    if let DataValidationStatus::Invalid(errors) = data_validation_status {
        assert_eq!(errors.len(), 3);
    }

    Ok(())
}
