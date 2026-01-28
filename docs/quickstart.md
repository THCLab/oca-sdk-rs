# oca-sdk-rs Quickstart

This guide walks through the most common SDK flows using the bundled examples and test assets.

## Prerequisites

From the `oca-sdk-rs` crate root:

```
cargo build
```

The examples use fixture files located under `tests/assets/`.

## 1) Build a bundle from an OCAfile

```
cargo run --example build_from_ocafile
```

What it does:
- Loads `tests/assets/semantics/entrance_credential.ocafile`
- Parses it into an AST
- Builds an `OCABundle`
- Prints the computed SAID

## 2) Validate bundle semantics

```
cargo run --example validate_semantics
```

What it does:
- Loads `tests/assets/semantics/structural_bundle2.json`
- Loads overlay definitions from `tests/assets/overlay-file/`
- Runs semantic validation and prints whether it is valid

## 3) Validate captured data

```
cargo run --example validate_data
```

What it does:
- Loads sample captured data from `tests/assets/data/nested.json`
- Loads a structural bundle from `tests/assets/semantics/structural_bundle2.json`
- Runs semantic validation first, then validates the data

## 4) Load a bundle JSON

```
cargo run --example load_bundle_json
```

What it does:
- Loads a bundle JSON
- Deserializes it into a model and prints its digest

## 5) Convert a bundle back to OCAfile

```
cargo run --example generate_ocafile
```

What it does:
- Loads a bundle JSON
- Converts its AST back to OCAfile format
- Prints the output

## Notes

- Integration tests live directly under `tests/`. Files under `tests/assets/` are fixtures.
- All example paths are relative to the crate root.
## Data fixtures

The data fixtures are split into valid/invalid samples:

- `tests/assets/data/nested_valid.json` — passes validation
- `tests/assets/data/nested_invalid.json` — fails validation (used by tests)

