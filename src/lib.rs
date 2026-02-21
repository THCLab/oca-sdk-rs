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

// --- Standard library imports ---
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
mod data_validator;

// -- Re-exports all modules in consistant interface
pub mod oca {
    pub mod utils {
        pub mod said {
            pub use oca_bundle::state::oca_bundle::error;
            pub use oca_bundle::state::oca_bundle::HashFunction;
            pub use oca_bundle::state::oca_bundle::HashFunctionCode;
            pub use oca_bundle::state::oca_bundle::SelfAddressingIdentifier;
        }
    }
    pub mod bundle {
        pub use oca_bundle::build::from_ast;
        pub use oca_bundle::controller::load_oca as load;
        pub use oca_bundle::state::attribute::Attribute;
        pub use oca_bundle::state::validator::{
            SemanticValidationStatus, Validator as OCAValidator,
        };
        pub use oca_bundle::state::{
            oca_bundle::{OCABundle, OCABundleModel},
            validator::validate as validate_semantics,
        };
    }
    pub mod file {
        pub use oca_file::ocafile::*;
    }
    pub mod overlay_file {
        pub use oca_ast::ast::{
            recursive_attributes::NestedAttrTypeFrame, AttributeType, NestedAttrType, RefValue,
        };
        pub use overlay_file::overlay_registry::OverlayLocalRegistry;
    }
    pub mod validator {
        pub use crate::data_validator::*;
    }
}

pub trait ToJSON {
    fn get_json_bundle(&self) -> String;
}

impl ToJSON for oca::bundle::OCABundleModel {
    fn get_json_bundle(&self) -> String {
        let oca_bundle = oca::bundle::OCABundle::from(self.clone());
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

impl WithInfo for oca::bundle::OCABundleModel {
    fn info(&self) -> Arc<OCABundleInfo> {
        let key = self as *const oca::bundle::OCABundleModel as usize;
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
    attributes: Option<HashMap<String, oca::bundle::Attribute>>,
    pub meta: HashMap<String, HashMap<String, String>>,
}

impl OCABundleInfo {
    pub fn new(bundle: &oca::bundle::OCABundleModel) -> Self {
        let meta = HashMap::new();
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

    pub fn attributes(&self) -> impl Iterator<Item = &oca::bundle::Attribute> {
        self.attributes.as_ref().unwrap().values()
    }

    pub fn attribute(&self, name: &str) -> Option<&oca::bundle::Attribute> {
        self.attributes.as_ref().unwrap().get(name)
    }
}
