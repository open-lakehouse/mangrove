// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::providers::v1::*;

impl Authorizable for ListProvidersRequest {
    fn operation(&self) -> Operation {
        Operation::ListProviders
    }

    fn securable(&self) -> Securable {
        Securable {
            label: None,
            name: Vec::new(),
        }
    }
}

impl Authorizable for CreateProviderRequest {
    fn operation(&self) -> Operation {
        Operation::CreateProvider
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Provider),
            name,
        }
    }
}

impl Authorizable for GetProviderRequest {
    fn operation(&self) -> Operation {
        Operation::GetProvider
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Provider),
            name,
        }
    }
}

impl Authorizable for UpdateProviderRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateProvider
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Provider),
            name,
        }
    }
}

impl Authorizable for DeleteProviderRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteProvider
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Provider),
            name,
        }
    }
}
