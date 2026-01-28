use std::fs;
use std::path::Path;

use oca_sdk_rs::{
    load, overlay_registry::OverlayLocalRegistry, validate_semantics, SemanticValidationStatus,
};

fn main() {
    let bundle_path = Path::new("tests/assets/semantics/structural_bundle2.json");
    let bundle_str = fs::read_to_string(bundle_path).expect("failed to read bundle");
    let registry =
        OverlayLocalRegistry::from_dir("tests/assets/overlay-file/").expect("load overlays");

    let bundle = load(&mut bundle_str.as_bytes(), &registry).expect("load bundle");
    let status = validate_semantics(&bundle).expect("validate semantics");

    match status {
        SemanticValidationStatus::Valid => println!("Semantics validation: valid"),
        SemanticValidationStatus::Invalid(errs) => {
            println!("Semantics validation: invalid ({} errors)", errs.len());
        }
    }
}
