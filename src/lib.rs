pub mod validator;
pub use oca_bundle_semantics::{controller::load_oca, state::validator::Validator as OCAValidator};

use oca_bundle_semantics::state::oca::{OCABox, OCABundle};

pub trait WithInfo {
     fn info(&self) -> OCABox;
}

impl WithInfo for OCABundle {
     fn info(&self) -> OCABox {
        OCABox::from(self.clone())
    }
}
