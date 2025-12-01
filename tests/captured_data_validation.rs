use oca_bundle::build::from_ast;
use oca_sdk_rs::{
    data_validator::{validate_data, DataValidationStatus},
    load, ocafile,
    overlay_registry::OverlayLocalRegistry,
    validate_semantics, SemanticValidationStatus,
};
use std::fs;
use std::path::Path;

#[test]
fn building_from_ocafile() -> Result<(), Box<dyn std::error::Error>> {
    let ocafile_path = Path::new("tests/assets/semantics/entrance_credential.ocafile");
    assert!(ocafile_path.exists(), "Asset file not found!");
    let ocafile_str = fs::read_to_string(ocafile_path)?;

    let overlay_registry = OverlayLocalRegistry::from_dir("tests/assets/overlay-file/")?;

    let oca_ast = ocafile::parse_from_string(ocafile_str, &overlay_registry)?;

    let oca_bundle = from_ast(None, &oca_ast).unwrap().oca_bundle;
    assert_eq!(
        oca_bundle.digest.clone().unwrap().to_string(),
        "EJoSIDigqNcOhKb2sJsdIdYQbKN3Dkkwab539d6h82si"
    );

    // oca_bundle.model.info().attributes().for_each(|attr| {
    //     println!("{:?}", attr);
    // });
    println!("{}", serde_json::to_string(&oca_bundle).unwrap());

    Ok(())
}

#[test]
fn validate_captured_data() -> Result<(), Box<dyn std::error::Error>> {
    let captured_data_path = Path::new("tests/assets/data/nested.json");
    assert!(captured_data_path.exists(), "Asset file not found!");
    let data_str = fs::read_to_string(captured_data_path)?;
    let data = serde_json::from_str(&data_str)?;

    let structural_bundle_path = Path::new("tests/assets/semantics/structural_bundle2.json");
    assert!(structural_bundle_path.exists(), "Asset file not found!");
    let structural_bundle_str = fs::read_to_string(structural_bundle_path)?;

    println!(">>> {:?}", structural_bundle_str);
    let overlay_registry = OverlayLocalRegistry::from_dir("tests/assets/overlay-file/")?;
    let mut structural_bundle =
        load(&mut structural_bundle_str.as_bytes(), &overlay_registry).unwrap();

    let semantics_validation_status = validate_semantics(&structural_bundle).unwrap();
    assert!(matches!(
        semantics_validation_status,
        SemanticValidationStatus::Valid
    ));

    let data_validation_status = validate_data(&mut structural_bundle, &data).unwrap();
    assert!(matches!(
        data_validation_status,
        DataValidationStatus::Invalid(_)
    ));
    if let DataValidationStatus::Invalid(errors) = data_validation_status {
        assert_eq!(errors.len(), 1);
    }

    Ok(())
}
