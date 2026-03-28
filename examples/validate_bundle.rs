use std::fs;

use oca_sdk_rs::oca;

fn main() {
    let bundle_path = "tests/assets/semantics/structural_bundle2.json";
    let bundle_str = fs::read_to_string(bundle_path).expect("failed to read bundle");
    let registry = oca::overlay_file::OverlayLocalRegistry::from_dir("tests/assets/overlay-file/")
        .expect("load overlays");

    let result = oca::bundle::validate_bundle(&bundle_str, &registry);

    if result.is_valid {
        println!("Bundle validation: valid");
    } else {
        println!("Bundle validation: invalid ({} issues)", result.issues.len());
        for issue in &result.issues {
            println!("  [{:?}] {}", issue.severity, issue.message);
        }
    }
}
