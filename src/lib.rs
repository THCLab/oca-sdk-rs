pub mod data_validator;
pub use oca_rs::facade::bundle::{load_oca, Bundle, BundleElement};
pub use oca_bundle_semantics::{
    controller::load_oca as load_oca_semantics, state::validator::validate as validate_semantics,
    state::validator::SemanticValidationStatus,
    state::validator::Validator as OCAValidator,
    state::attribute::{Attribute, AttributeType},
    state::oca::{OCABox as StructuralBox, OCABundle as StructuralBundle}
};
pub use oca_ast_semantics::ast::NestedAttrType;
pub use transformation_file::state::Transformation;

pub trait WithInfo {
    fn info(&self) -> StructuralBox;
}

impl WithInfo for StructuralBundle {
    fn info(&self) -> StructuralBox {
        StructuralBox::from(self.clone())
    }
}
