//! Data validation against bundles: capture-base types, nested bundles, the
//! core overlays with rules, and the hcf: structural overlays.
//!
//! The bundles in tests/assets/structural were built with oca-bin 0.8.1 from
//! small ocafiles (one per overlay); see each bundle's capture base.

use oca_sdk_rs::oca::data::{BundleSet, ErrorKind, Options, ValidationError, Validator};
use oca_sdk_rs::oca::overlay_file::OverlayLocalRegistry;
use serde_json::{json, Value};
use std::fs;

const NAMES: [&str; 7] = ["cat", "dog", "pet", "order", "menu", "message", "contact"];

fn set() -> (BundleSet, std::collections::HashMap<&'static str, String>) {
    let dir = "tests/assets/structural";
    let defs = fs::read_to_string(format!("{dir}/definitions.overlayfile")).unwrap();
    let registry = OverlayLocalRegistry::from_string(defs).unwrap();
    let jsons: Vec<String> = NAMES
        .iter()
        .map(|n| fs::read_to_string(format!("{dir}/{n}.json")).unwrap())
        .collect();
    let set = BundleSet::from_json(jsons.iter().map(String::as_str), &registry).unwrap();
    let saids = NAMES
        .iter()
        .zip(&jsons)
        .map(|(n, j)| {
            (
                *n,
                serde_json::from_str::<Value>(j).unwrap()["digest"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            )
        })
        .collect();
    (set, saids)
}

fn check(bundle: &str, data: Value) -> Vec<ValidationError> {
    let (set, saids) = set();
    Validator::new(&set).validate(&data, &saids[bundle])
}

fn kinds(errors: &[ValidationError]) -> Vec<(String, ErrorKind)> {
    errors
        .iter()
        .map(|e| (e.path.clone(), e.kind.clone()))
        .collect()
}

fn has(errors: &[ValidationError], path: &str, kind: ErrorKind) -> bool {
    errors.iter().any(|e| e.path == path && e.kind == kind)
}

#[test]
fn valid_data_has_no_errors() {
    let errors = check(
        "contact",
        json!({
            "name": "Ada", "email": "ada@example.org", "kind": "person", "tags": ["a"],
            "born": "1815-12-10", "pet": { "kind": "cat", "name": "Tom", "indoor": true }
        }),
    );
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn types_conformance_and_unknown_attributes() {
    let errors = check(
        "contact",
        json!({ "email": 3, "tags": ["a"], "colour": "red", "born": "yesterday" }),
    );
    assert!(
        has(&errors, "$.name", ErrorKind::MissingAttribute),
        "{errors:?}"
    );
    assert!(has(&errors, "$.email", ErrorKind::WrongType));
    assert!(has(&errors, "$.colour", ErrorKind::UnknownAttribute));
    assert!(has(&errors, "$.born", ErrorKind::WrongType));

    let (set, saids) = set();
    let lenient = Validator::with_options(
        &set,
        Options {
            allow_unknown_attributes: true,
        },
    );
    let errors = lenient.validate(
        &json!({ "name": "Ada", "colour": "red" }),
        &saids["contact"],
    );
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn entry_codes_formats_and_cardinality() {
    let errors = check(
        "contact",
        json!({ "name": "Ada", "kind": "robot", "email": "not an address", "tags": [] }),
    );
    assert!(
        has(&errors, "$.kind", ErrorKind::NotAnEntryCode),
        "{errors:?}"
    );
    assert!(has(&errors, "$.email", ErrorKind::FormatMismatch));
    assert!(has(&errors, "$.tags", ErrorKind::Cardinality));
    let errors = check(
        "contact",
        json!({ "name": "Ada", "tags": ["a", "b", "c", "d"] }),
    );
    assert!(has(&errors, "$.tags", ErrorKind::Cardinality));
}

#[test]
fn nested_bundles_are_validated() {
    let errors = check(
        "contact",
        json!({ "name": "Ada", "pet": { "kind": "cat", "name": 7 } }),
    );
    assert!(
        has(&errors, "$.pet.name", ErrorKind::WrongType),
        "{errors:?}"
    );
}

#[test]
fn variant_selects_the_shape() {
    assert!(check(
        "pet",
        json!({ "kind": "dog", "name": "Rex", "breed": "collie" })
    )
    .is_empty());
    // breed belongs to dogs, indoor to cats
    let errors = check(
        "pet",
        json!({ "kind": "cat", "name": "Tom", "breed": "collie" }),
    );
    assert_eq!(
        kinds(&errors),
        vec![("$.breed".to_string(), ErrorKind::UnknownAttribute)]
    );
    let errors = check(
        "pet",
        json!({ "kind": "dog", "name": "Rex", "indoor": "yes" }),
    );
    assert!(
        has(&errors, "$.indoor", ErrorKind::UnknownAttribute),
        "{errors:?}"
    );
    let errors = check("pet", json!({ "kind": "fish", "name": "Nemo" }));
    assert!(
        has(&errors, "$.kind", ErrorKind::NotAnEntryCode),
        "{errors:?}"
    );
}

#[test]
fn ranges_lengths_and_unions() {
    assert!(check(
        "order",
        json!({ "quantity": 3, "discount": "0.25", "code": "A1", "extra": null })
    )
    .is_empty());
    let errors = check(
        "order",
        json!({ "quantity": 0, "discount": "0.9", "code": "", "extra": true }),
    );
    assert!(
        has(&errors, "$.quantity", ErrorKind::OutOfRange),
        "{errors:?}"
    );
    assert!(has(&errors, "$.discount", ErrorKind::OutOfRange));
    assert!(has(&errors, "$.code", ErrorKind::BadLength));
    assert!(has(&errors, "$.extra", ErrorKind::WrongType));
    let errors = check(
        "order",
        json!({ "discount": "a lot", "code": "ABCDEFGHIJKLM" }),
    );
    assert!(
        has(&errors, "$.discount", ErrorKind::OutOfRange),
        "{errors:?}"
    );
    assert!(has(&errors, "$.code", ErrorKind::BadLength));
    // null only where a union allows it
    let errors = check("order", json!({ "code": null }));
    assert!(has(&errors, "$.code", ErrorKind::WrongType), "{errors:?}");
}

#[test]
fn maps_check_keys_and_values() {
    assert!(check(
        "order",
        json!({ "labels": { "colour": "red", "size-xl": "yes" } })
    )
    .is_empty());
    let errors = check("order", json!({ "labels": { "Colour": "red", "size": 3 } }));
    assert!(
        has(&errors, "$.labels.Colour", ErrorKind::BadMapKey),
        "{errors:?}"
    );
    assert!(has(&errors, "$.labels.size", ErrorKind::WrongType));
    let errors = check("order", json!({ "labels": ["red"] }));
    assert!(has(&errors, "$.labels", ErrorKind::WrongType), "{errors:?}");
}

#[test]
fn recursive_trees_and_their_depth() {
    let tree = json!({ "label": "File", "children": [{ "label": "Open", "children": [] }, { "label": 3 }] });
    let errors = check("menu", tree);
    assert_eq!(
        kinds(&errors),
        vec![("$.children[1].label".to_string(), ErrorKind::WrongType)]
    );

    let mut deep = json!({ "label": "leaf" });
    for _ in 0..10 {
        deep = json!({ "label": "node", "children": [deep] });
    }
    let errors = check("menu", deep);
    assert!(
        errors.iter().any(|e| e.kind == ErrorKind::TooDeep),
        "{errors:?}"
    );
}

#[test]
fn json_text_is_decoded_and_validated() {
    assert!(check(
        "message",
        json!({ "body": "{\"kind\":\"dog\",\"name\":\"Rex\",\"breed\":\"collie\"}" })
    )
    .is_empty());
    let errors = check(
        "message",
        json!({ "body": "{\"kind\":\"dog\",\"name\":1}" }),
    );
    assert!(
        has(&errors, "$.body<json>.name", ErrorKind::WrongType),
        "{errors:?}"
    );
    let errors = check("message", json!({ "body": "{not json" }));
    assert!(has(&errors, "$.body", ErrorKind::InvalidJson), "{errors:?}");
}

#[test]
fn unknown_bundles_are_reported() {
    let (set, _) = set();
    let errors =
        Validator::new(&set).validate(&json!({}), "EAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(errors[0].kind, ErrorKind::UnresolvedReference);
}

#[test]
fn visitor_sees_every_object_with_its_bundles() {
    let (set, saids) = set();
    let mut seen: Vec<String> = Vec::new();
    let data = json!({ "name": "Ada", "pet": { "kind": "dog", "name": "Rex" } });
    let mut visitor = |path: &str,
                       _b: &oca_sdk_rs::oca::bundle::OCABundleModel,
                       _o: &serde_json::Map<String, Value>| {
        seen.push(path.to_string());
    };
    let errors = Validator::new(&set).validate_with(&data, &saids["contact"], &mut visitor);
    assert!(errors.is_empty(), "{errors:?}");
    // the pet is seen twice: as a pet and as a dog
    assert_eq!(seen, vec!["$.pet", "$.pet", "$"]);
}
