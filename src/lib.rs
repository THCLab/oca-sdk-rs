//! # OCA SDK
//!
//! Compact yet powerful SDK for working with OCA bundles.
//!
//! # Features
//!
//! - Create OCA Bundle from OCAFile.
//! - Validate OCA Bundle semantics.
//! - Validate data against OCA Bundle.
//! - Traverse through OCA Bundle attributes.
pub mod data_validator;
pub use oca_ast::ast::{
    recursive_attributes::NestedAttrTypeFrame, AttributeType, NestedAttrType,
    RefValue,
};

pub use oca_bundle::state::oca_bundle::OCABundle;
pub use oca_bundle::state::oca_bundle::OCABundleModel;
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
/// * `Err(String)` - If  a critical error occurs during validation.
///
/// # Errors
/// * Returns `Err` with a string message if the validation process encounters unexpected errors.
///
/// # Examples
/// ```
/// use std::fs;
/// use std::path::Path;
/// use oca_sdk_rs::{load, validate_semantics, SemanticValidationStatus};
///
/// let structural_bundle_path = Path::new("tests/assets/semantics/structural_bundle.json");
/// let structural_bundle_str = fs::read_to_string(structural_bundle_path).expect("Failed to read the file");
///
/// let structural_bundle = load(&mut structural_bundle_str.as_bytes()).unwrap();
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
pub use oca_bundle::state::validator::validate as validate_semantics;
pub use oca_bundle::{
    controller::load_oca as load,
    state::{
        attribute::Attribute,
        validator::{SemanticValidationStatus, Validator as OCAValidator},
    },
};
pub use oca_store::facade::{
    build::{build_from_ocafile, parse_oca_bundle_to_ocafile},
    Facade,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};


pub use overlay_file::overlay_registry;
pub use oca_bundle::state::oca_bundle::SelfAddressingIdentifier;
pub use oca_bundle::state::oca_bundle::{HashFunction, HashFunctionCode};
pub use oca_bundle::state::oca_bundle::error as said_error;

pub trait ToJSON {
    fn get_json_bundle(&self) -> String;
}

impl ToJSON for OCABundleModel {
    fn get_json_bundle(&self) -> String {
        let oca_bundle = OCABundle::from(self.clone());
        let result = serde_json::to_string_pretty(&oca_bundle);
        match result {
            Ok(json) => json,
            Err(e) => format!("Error converting to JSON: {}", e),
        }
    }
}

lazy_static::lazy_static! {
    static ref INFO_CACHE: Mutex<HashMap<usize, Weak<OCABundleInfo>>> = Mutex::new(HashMap::new());
}

pub trait WithInfo {
    fn info(&self) -> Arc<OCABundleInfo>;
}

impl WithInfo for OCABundleModel {
    fn info(&self) -> Arc<OCABundleInfo> {
        let key = self as *const OCABundleModel as usize;
        let mut cache = INFO_CACHE.lock().unwrap();
        if let Some(weak_info) = cache.get(&key) {
            if let Some(info) = weak_info.upgrade() {
                return info;
            }
        }

        let new_info = Arc::new(OCABundleInfo::new(self));
        cache.insert(key, Arc::downgrade(&new_info));
        new_info
    }
}

pub struct OCABundleInfo {
    // TODO Find out if this should be option or not
    attributes: Option<HashMap<String, Attribute>>,
    pub meta: HashMap<String, HashMap<String, String>>,
}

impl OCABundleInfo {
    pub fn new(bundle: &OCABundleModel) -> Self {
        let mut meta = HashMap::new();
        // TODO fix it
        // let oca_box = OCABox::from(bundle.clone());
        // if let Some(m) = oca_box.meta {
        //     m.iter().for_each(|(k, v)| {
        //         meta.insert(k.unwrap().to_639_3().to_string(), v.to_owned());
        //     })
        // }

        Self {
            attributes: bundle.attributes.clone(),
            meta,
        }
    }

    pub fn attributes(&self) -> impl Iterator<Item = &Attribute> {
        self.attributes.as_ref().unwrap().values()
    }

    pub fn attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.as_ref().unwrap().get(name)
    }
}
