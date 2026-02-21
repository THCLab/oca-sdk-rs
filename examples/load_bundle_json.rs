use std::fs;

use oca_sdk_rs::*;

fn main() {
    let bundle_path = "tests/assets/semantics/structural_bundle2.json";
    let bundle_str = fs::read_to_string(bundle_path).expect("failed to read bundle");

    let registry = oca::overlay_file::OverlayLocalRegistry::from_dir("tests/assets/overlay-file/")
        .expect("load overlays");
    let bundle = oca::bundle::load(&mut bundle_str.as_bytes(), &registry).expect("load bundle");

    println!("Loaded bundle with digest: {:?}", bundle.digest);
}
