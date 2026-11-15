// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::functions::v1::*;

impl Authorizable for ListFunctionsRequest {
    fn operation(&self) -> Operation {
        Operation::ListFunctions
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        name.extend(self.schema_name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Schema),
            name,
        }
    }
}

impl Authorizable for CreateFunctionRequest {
    fn operation(&self) -> Operation {
        Operation::CreateFunction
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(
            self.function_info
                .catalog_name
                .split('.')
                .map(str::to_string),
        );
        name.extend(
            self.function_info
                .schema_name
                .split('.')
                .map(str::to_string),
        );
        name.extend(self.function_info.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Function),
            name,
        }
    }
}

impl Authorizable for GetFunctionRequest {
    fn operation(&self) -> Operation {
        Operation::GetFunction
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Function),
            name,
        }
    }
}

impl Authorizable for UpdateFunctionRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateFunction
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Function),
            name,
        }
    }
}

impl Authorizable for DeleteFunctionRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteFunction
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Function),
            name,
        }
    }
}
