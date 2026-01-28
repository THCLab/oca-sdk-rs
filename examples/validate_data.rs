use std::fs;
use std::path::Path;

use oca_sdk_rs::{
    data_validator::{validate_data, DataValidationStatus},
    load,
    overlay_registry::OverlayLocalRegistry,
    validate_semantics, SemanticValidationStatus,
};

fn main() {
    let data_path = Path::new("tests/assets/data/nested_valid.json");
    let data_str = fs::read_to_string(data_path).expect("failed to read data");
    let data: serde_json::Value = serde_json::from_str(&data_str).expect("parse data");

    let bundle_path = Path::new("tests/assets/semantics/structural_bundle2.json");
    let bundle_str = fs::read_to_string(bundle_path).expect("failed to read bundle");

    let registry =
        OverlayLocalRegistry::from_dir("tests/assets/overlay-file/").expect("load overlays");
    let mut bundle = load(&mut bundle_str.as_bytes(), &registry).expect("load bundle");

    match validate_semantics(&bundle).expect("validate semantics") {
        SemanticValidationStatus::Valid => {}
        SemanticValidationStatus::Invalid(errs) => {
            eprintln!("Semantics validation failed:");
            for err in errs {
                eprintln!("- {}", err);
            }
            std::process::exit(1);
        }
    }

    let status = validate_data(&mut bundle, &data).expect("validate data");
    match status {
        DataValidationStatus::Valid => println!("Data validation: valid"),
        DataValidationStatus::Invalid(errs) => {
            eprintln!("Data validation failed:");
            for err in errs {
                eprintln!("- {}", err);
            }
            std::process::exit(1);
        }
    }
}
