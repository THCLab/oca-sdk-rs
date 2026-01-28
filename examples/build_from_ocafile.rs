use std::fs;

use oca_bundle::build::from_ast;
use oca_sdk_rs::{ocafile, overlay_registry::OverlayLocalRegistry};

fn main() {
    let ocafile_path = "tests/assets/semantics/entrance_credential.ocafile";
    let ocafile_str = fs::read_to_string(ocafile_path).expect("failed to read ocafile");

    let registry =
        OverlayLocalRegistry::from_dir("tests/assets/overlay-file/").expect("load overlays");
    let oca_ast = ocafile::parse_from_string(ocafile_str, &registry).expect("parse ocafile");
    let oca_bundle = match from_ast(None, &oca_ast) {
        Ok(build) => build.oca_bundle,
        Err(errs) => {
            eprintln!("build errors: {:?}", errs);
            std::process::exit(1);
        }
    };

    println!("SAID: {}", oca_bundle.digest.unwrap());
}
