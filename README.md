# OCA SDK
Comprehensive SDK for OCA Bundle management and integration in Rust.

This crate helps you:
- parse OCAfiles,
- build and validate OCA bundles,
- validate captured data against bundles,
- load/serialize bundles and overlays.

For a step-by-step walkthrough with runnable commands, see `docs/quickstart.md`.

## Usage (library)

Add to your `Cargo.toml`:

```
oca-sdk-rs = "2.0.0-rc.5"
```

Then import what you need, for example:

```
use oca_sdk_rs::{load, validate_semantics, OCABundleModel, SemanticValidationStatus};
```

For end-to-end runnable flows, use the examples (next section) and `docs/quickstart.md`.

## License

EUPL 1.2

We have distilled the most crucial license specifics to make your adoption seamless: [see here for details](https://github.com/THCLab/licensing).

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
