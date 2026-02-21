# OCA SDK

Comprehensive SDK for OCA Bundle management and integration in Rust.

This crate helps you:
- parse OCAfiles,
- build and validate OCA bundles,
- validate captured data against bundles,
- load/serialize bundles and overlays.

## Usage (library)

Add to your `Cargo.toml`:

```
oca-sdk-rs = "2.0.0-rc.5"
```

### Creating a Bundle from OCAFile

```rust
use oca_sdk_rs::oca::overlay_file::OverlayLocalRegistry;
use oca_sdk_rs::oca::file::parse_from_string;
use oca_sdk_rs::oca::bundle::from_ast;

let overlay_registry = OverlayLocalRegistry::from_dir("path/to/overlay-files")?;
let ocafile_str = std::fs::read_to_string("path/to/ocafile.ocafile")?;
let oca_ast = parse_from_string(ocafile_str, &overlay_registry)?;
let oca_bundle = from_ast(None, &oca_ast)?.oca_bundle;
```

### Validating Bundle Semantics

```rust
use oca_sdk_rs::oca::bundle::{validate_semantics, SemanticValidationStatus};

let status = validate_semantics(&oca_bundle)?;
assert!(matches!(status, SemanticValidationStatus::Valid));
```

### Validating Captured Data

```rust
use oca_sdk_rs::oca::validator::validate_data;

let data = serde_json::from_str(r#"{"field": "value"}"#)?;
let validation_status = validate_data(&mut oca_bundle, &data)?;
```

### Accessing Bundle Attributes

```rust
use oca_sdk_rs::oca::bundle::{OCABundleModel, WithInfo};

let info = oca_bundle.model.info();
for attr in info.attributes() {
    println!("{:?}", attr);
}
```

### Converting Bundle to JSON

```rust
use oca_sdk_rs::oca::bundle::{OCABundleModel, ToJSON};

let json = oca_bundle.model.get_json_bundle();
println!("{}", json);
```

## Tests

Integration tests must live directly under `tests/` (for example `tests/captured_data_validation.rs`).
Files placed in subdirectories like `tests/assets/` are treated as fixtures/modules and will not be
picked up by Cargo automatically.

## Examples

Run any example with:

```
cargo run --example <name>
```

Available examples:
- `build_from_ocafile` — parse an ocafile and build a bundle
- `validate_semantics` — validate semantics for a bundle JSON
- `validate_data` — validate captured JSON data
- `load_bundle_json` — load an existing bundle JSON
- `generate_ocafile` — convert a bundle JSON back to ocafile

Notes:
- Examples use fixtures under `tests/assets/` (paths are relative to the crate root).
- If you run from a different working directory, use absolute paths or `cargo run --example <name> -- <args>`.

See `docs/quickstart.md` for a step-by-step walkthrough with expected inputs/outputs.

## License

EUPL 1.2

We have distilled the most crucial license specifics to make your adoption seamless: [see here for details](https://github.com/THCLab/licensing).

