pub mod data_validator;
pub use oca_ast_semantics::ast::NestedAttrType;

/// Performs semantic validation of an `OCABundle` and returns a status
/// indicating whether the validation succeeded or failed, along with any associated errors.
///
/// Semantics validation ensures Bundle integrity, that is, that the Bundle identifier under `d`
/// attribute matches the hash of the Bundle content.
///
/// # Arguments
/// * `oca_bundle` - A reference to an `OCABundle` instance to be validated.
///   The `OCABundle` contains the schema and data to be checked for semantic correctness.
///
/// # Returns
/// * `Ok(SemanticValidationStatus::Valid)` - If the `OCABundle` passes all semantic validation checks.
/// * `Ok(SemanticValidationStatus::Invalid(errors))` - If validation errors are found, with a vector of error messages.
/// * `Err(String)` - If a critical error occurs during validation.
///
/// # Errors
/// * Returns `Err` with a string message if the validation process encounters unexpected errors.
///
/// # Examples
/// ```
/// use std::fs;
/// use std::path::Path;
/// use oca_sdk_rs::{load_oca_semantics, validate_semantics, SemanticValidationStatus};
///
/// let structural_bundle_path = Path::new("tests/assets/semantics/structural_bundle.json");
/// let structural_bundle_str = fs::read_to_string(structural_bundle_path).expect("Failed to read the file");
///
/// let structural_bundle = load_oca_semantics(&mut structural_bundle_str.as_bytes()).unwrap();
///
/// let semantics_validation_status = validate_semantics(&structural_bundle).unwrap();
///
/// match semantics_validation_status {
///     SemanticValidationStatus::Valid => println!("The structural bundle is valid!"),
///     SemanticValidationStatus::Invalid(errors) => {
///         println!("Validation errors:");
///         for error in errors {
///             println!("  - {}", error);
///         }
///     }
/// }
/// ```
pub use oca_bundle_semantics::state::validator::validate as validate_semantics;
pub use oca_bundle_semantics::{
    controller::load_oca as load_oca_semantics,
    state::{
        attribute::{Attribute, AttributeType},
        oca::{OCABox as StructuralBox, OCABundle as StructuralBundle},
        validator::{SemanticValidationStatus, Validator as OCAValidator},
    },
};
pub use oca_rs::facade::{
    build::build_from_ocafile,
    bundle::{load_oca, Bundle, BundleElement},
};
use std::collections::HashMap;
pub use transformation_file::state::Transformation;

pub trait WithInfo {
    fn info(&self) -> BundleInfo;
}

impl WithInfo for Bundle {
    fn info(&self) -> BundleInfo {
        BundleInfo::new(self.clone())
    }
}

pub struct BundleInfo {
    pub attributes: HashMap<String, Attribute>,
    pub meta: HashMap<String, HashMap<String, String>>,
    pub links: Vec<Transformation>,
    pub framing: Vec<String>,
}

impl BundleInfo {
    pub fn new(bundle: Bundle) -> Self {
        let mut attributes = HashMap::new();
        let mut meta = HashMap::new();
        if let Some(structural_bundle) = bundle.structural {
            let structural_box = StructuralBox::from(structural_bundle.clone());
            if let Some(m) = structural_box.meta {
                m.iter().for_each(|(k, v)| {
                    meta.insert(k.to_639_3().to_string(), v.to_owned());
                })
            }
            attributes = structural_box.attributes;
        }

        Self {
            attributes,
            meta,
            links: bundle.transformations,
            framing: vec![],
        }
    }

    pub fn attributes(&self) -> std::collections::hash_map::Values<'_, String, Attribute> {
        self.attributes.values()
    }

    pub fn attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.get(name)
    }
}
