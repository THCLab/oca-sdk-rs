# OCA SDK
Comprehensive SDK for OCA Bundle management and integration

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
See `docs/quickstart.md` for a step-by-step walkthrough.
