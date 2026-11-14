// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::external_locations::v1::*;

impl Authorizable for ListExternalLocationsRequest {
    fn operation(&self) -> Operation {
        Operation::ListExternalLocations
    }

    fn securable(&self) -> Securable {
        Securable {
            label: None,
            name: Vec::new(),
        }
    }
}

impl Authorizable for CreateExternalLocationRequest {
    fn operation(&self) -> Operation {
        Operation::CreateExternalLocation
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::ExternalLocation),
            name,
        }
    }
}

impl Authorizable for GetExternalLocationRequest {
    fn operation(&self) -> Operation {
        Operation::GetExternalLocation
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::ExternalLocation),
            name,
        }
    }
}

impl Authorizable for UpdateExternalLocationRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateExternalLocation
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::ExternalLocation),
            name,
        }
    }
}

impl Authorizable for DeleteExternalLocationRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteExternalLocation
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::ExternalLocation),
            name,
        }
    }
}
