// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::registered_models::v1::*;

impl Authorizable for ListRegisteredModelsRequest {
    fn operation(&self) -> Operation {
        Operation::ListRegisteredModels
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        if let Some(value) = &self.catalog_name {
            name.extend(value.split('.').map(str::to_string));
        }
        if let Some(value) = &self.schema_name {
            name.extend(value.split('.').map(str::to_string));
        }
        Securable {
            label: Some(ObjectLabel::Schema),
            name,
        }
    }
}

impl Authorizable for CreateRegisteredModelRequest {
    fn operation(&self) -> Operation {
        Operation::CreateRegisteredModel
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        name.extend(self.schema_name.split('.').map(str::to_string));
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::RegisteredModel),
            name,
        }
    }
}

impl Authorizable for GetRegisteredModelRequest {
    fn operation(&self) -> Operation {
        Operation::GetRegisteredModel
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.full_name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::RegisteredModel),
            name,
        }
    }
}

impl Authorizable for UpdateRegisteredModelRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateRegisteredModel
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.full_name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::RegisteredModel),
            name,
        }
    }
}

impl Authorizable for DeleteRegisteredModelRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteRegisteredModel
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.full_name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::RegisteredModel),
            name,
        }
    }
}
