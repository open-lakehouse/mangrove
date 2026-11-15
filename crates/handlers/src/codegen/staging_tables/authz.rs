// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::staging_tables::v1::*;

impl Authorizable for CreateStagingTableRequest {
    fn operation(&self) -> Operation {
        Operation::CreateStagingTable
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        name.extend(self.schema_name.split('.').map(str::to_string));
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::StagingTable),
            name,
        }
    }
}
