// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::model_versions::v1::*;

impl Authorizable for ListModelVersionsRequest {
    fn operation(&self) -> Operation {
        Operation::ListModelVersions
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

impl Authorizable for CreateModelVersionRequest {
    fn operation(&self) -> Operation {
        Operation::CreateModelVersion
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        name.extend(self.schema_name.split('.').map(str::to_string));
        name.extend(self.model_name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::RegisteredModel),
            name,
        }
    }
}

impl Authorizable for GetModelVersionRequest {
    fn operation(&self) -> Operation {
        Operation::GetModelVersion
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.full_name.split('.').map(str::to_string));
        name.push(self.version.to_string());
        Securable {
            label: Some(ObjectLabel::ModelVersion),
            name,
        }
    }
}

impl Authorizable for UpdateModelVersionRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateModelVersion
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.full_name.split('.').map(str::to_string));
        name.push(self.version.to_string());
        Securable {
            label: Some(ObjectLabel::ModelVersion),
            name,
        }
    }
}

impl Authorizable for DeleteModelVersionRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteModelVersion
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.full_name.split('.').map(str::to_string));
        name.push(self.version.to_string());
        Securable {
            label: Some(ObjectLabel::ModelVersion),
            name,
        }
    }
}

impl Authorizable for FinalizeModelVersionRequest {
    fn operation(&self) -> Operation {
        Operation::FinalizeModelVersion
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.full_name.split('.').map(str::to_string));
        name.push(self.version.to_string());
        Securable {
            label: Some(ObjectLabel::ModelVersion),
            name,
        }
    }
}
