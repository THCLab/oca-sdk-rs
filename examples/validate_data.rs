use std::fs;
use std::path::Path;

use oca_sdk_rs::oca;

fn main() {
    let data_path = Path::new("tests/assets/data/nested_valid.json");
    let data_str = fs::read_to_string(data_path).expect("failed to read data");
    let data: serde_json::Value = serde_json::from_str(&data_str).expect("parse data");

    let bundle_path = Path::new("tests/assets/semantics/structural_bundle2.json");
    let bundle_str = fs::read_to_string(bundle_path).expect("failed to read bundle");

    let registry =
        oca::overlay_file::OverlayLocalRegistry::from_dir("tests/assets/overlay-file/").expect("load overlays");
    let mut bundle = oca::bundle::load(&mut bundle_str.as_bytes(), &registry).expect("load bundle");

    match oca::bundle::validate_semantics(&bundle).expect("validate semantics") {
        oca::bundle::SemanticValidationStatus::Valid => {}
        oca::bundle::SemanticValidationStatus::Invalid(errs) => {
            eprintln!("Semantics validation failed:");
            for err in errs {
                eprintln!("- {}", err);
            }
            std::process::exit(1);
        }
    }

    let status = oca::validator::validate_data(&mut bundle, &data).expect("validate data");
    match status {
        oca::validator::DataValidationStatus::Valid => println!("Data validation: valid"),
        oca::validator::DataValidationStatus::Invalid(errs) => {
            eprintln!("Data validation failed:");
            for err in errs {
                eprintln!("- {}", err);
            }
            std::process::exit(1);
        }
    }
}
