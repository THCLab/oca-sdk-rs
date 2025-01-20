use oca_sdk_rs::{
    build_from_ocafile,
    data_validator::{validate_data, DataValidationStatus},
    load, validate_semantics, SemanticValidationStatus, ToJSON, WithInfo,
};
use std::fs;
use std::path::Path;

#[test]
fn building_from_ocafile() -> Result<(), Box<dyn std::error::Error>> {
    let ocafile_path = Path::new("tests/assets/semantics/entrance_credential.ocafile");
    assert!(ocafile_path.exists(), "Asset file not found!");
    let ocafile_str = fs::read_to_string(ocafile_path)?;

    let oca_bundle = build_from_ocafile(ocafile_str).unwrap();
    assert_eq!(
        oca_bundle.said.clone().unwrap().to_string(),
        "EKHBds6myKVIsQuT7Zr23M8Xk_gwq-2SaDRUprvqOXxa"
    );

    oca_bundle.info().attributes().for_each(|attr| {
        println!("{:?}", attr);
    });
    println!("{}", oca_bundle.get_json_bundle());

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

    let structural_bundle = load(&mut structural_bundle_str.as_bytes()).unwrap();

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
