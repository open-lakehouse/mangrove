// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::schemas::v1::*;

impl Authorizable for ListSchemasRequest {
    fn operation(&self) -> Operation {
        Operation::ListSchemas
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Catalog),
            name,
        }
    }
}

impl Authorizable for CreateSchemaRequest {
    fn operation(&self) -> Operation {
        Operation::CreateSchema
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Schema),
            name,
        }
    }
}

impl Authorizable for GetSchemaRequest {
    fn operation(&self) -> Operation {
        Operation::GetSchema
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.full_name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Schema),
            name,
        }
    }
}

impl Authorizable for UpdateSchemaRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateSchema
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.full_name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Schema),
            name,
        }
    }
}

impl Authorizable for DeleteSchemaRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteSchema
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.full_name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Schema),
            name,
        }
    }
}
