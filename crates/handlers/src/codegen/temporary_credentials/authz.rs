// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::temporary_credentials::v1::*;

impl Authorizable for GenerateTemporaryModelVersionCredentialsRequest {
    fn operation(&self) -> Operation {
        Operation::GenerateTemporaryModelVersionCredentials
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        name.extend(self.schema_name.split('.').map(str::to_string));
        name.extend(self.model_name.split('.').map(str::to_string));
        name.push(self.version.to_string());
        Securable {
            label: Some(ObjectLabel::ModelVersion),
            name,
        }
    }
}
