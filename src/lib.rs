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
pub use oca_ast_semantics::ast::{
    recursive_attributes::NestedAttrTypeFrame, AttributeType, NestedAttrType,
    OverlayType, RefValue,
};

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
pub use oca_bundle_semantics::state::validator::validate as validate_semantics;
pub use oca_bundle_semantics::{
    controller::load_oca as load,
    state::{
        attribute::Attribute,
        oca::{overlay, OCABox, OCABundle},
        validator::{SemanticValidationStatus, Validator as OCAValidator},
    },
};
pub use oca_rs::facade::{
    build::{build_from_ocafile, parse_oca_bundle_to_ocafile},
    Facade,
};
use oca_rs::{EncodeBundle, HashFunctionCode, SerializationFormats};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

pub trait ToJSON {
    fn get_json_bundle(&self) -> String;
}

impl ToJSON for OCABundle {
    fn get_json_bundle(&self) -> String {
        let code = HashFunctionCode::Blake3_256;
        let format = SerializationFormats::JSON;

        String::from_utf8(self.encode(&code, &format).unwrap()).unwrap()
    }
}

lazy_static::lazy_static! {
    static ref INFO_CACHE: Mutex<HashMap<usize, Weak<OCABundleInfo>>> = Mutex::new(HashMap::new());
}

pub trait WithInfo {
    fn info(&self) -> Arc<OCABundleInfo>;
}

impl WithInfo for OCABundle {
    fn info(&self) -> Arc<OCABundleInfo> {
        let key = self as *const OCABundle as usize;
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
    attributes: HashMap<String, Attribute>,
    pub meta: HashMap<String, HashMap<String, String>>,
    pub links: Vec<overlay::Link>,
    pub framings: Vec<overlay::AttributeFraming>,
}

impl OCABundleInfo {
    pub fn new(bundle: &OCABundle) -> Self {
        let mut meta = HashMap::new();
        let oca_box = OCABox::from(bundle.clone());
        if let Some(m) = oca_box.meta {
            m.iter().for_each(|(k, v)| {
                meta.insert(k.to_639_3().to_string(), v.to_owned());
            })
        }

        let mut overlays = bundle.overlays.clone();
        let links: Vec<overlay::Link> = overlays
            .iter_mut()
            .filter(|o| o.as_any().downcast_ref::<overlay::Link>().is_some())
            .map(|o| {
                o.as_any()
                    .downcast_ref::<overlay::Link>()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        let framings: Vec<overlay::AttributeFraming> = overlays
            .iter_mut()
            .filter(|o| {
                o.as_any()
                    .downcast_ref::<overlay::AttributeFraming>()
                    .is_some()
            })
            .map(|o| {
                o.as_any()
                    .downcast_ref::<overlay::AttributeFraming>()
                    .unwrap()
                    .to_owned()
            })
            .collect();

        Self {
            attributes: oca_box.attributes,
            meta,
            links,
            framings,
        }
    }

    pub fn attributes(&self) -> impl Iterator<Item = &Attribute> {
        self.attributes.values()
    }

    pub fn attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.get(name)
    }
}
