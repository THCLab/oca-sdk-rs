pub mod data_validator;
pub use oca_ast_semantics::ast::NestedAttrType;
pub use oca_bundle_semantics::{
    controller::load_oca as load_oca_semantics,
    state::{
        attribute::{Attribute, AttributeType},
        oca::{OCABox as StructuralBox, OCABundle as StructuralBundle},
        validator::{
            validate as validate_semantics, SemanticValidationStatus,
            Validator as OCAValidator,
        },
    },
};
pub use oca_rs::facade::bundle::{load_oca, Bundle, BundleElement};
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

    pub fn attributes(
        &self,
    ) -> std::collections::hash_map::Values<'_, String, Attribute> {
        self.attributes.values()
    }

    pub fn attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.get(name)
    }
}
