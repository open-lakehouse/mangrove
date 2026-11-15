// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::credentials::v1::*;

impl Authorizable for ListCredentialsRequest {
    fn operation(&self) -> Operation {
        Operation::ListCredentials
    }

    fn securable(&self) -> Securable {
        Securable {
            label: None,
            name: Vec::new(),
        }
    }
}

impl Authorizable for CreateCredentialRequest {
    fn operation(&self) -> Operation {
        Operation::CreateCredential
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Credential),
            name,
        }
    }
}

impl Authorizable for GetCredentialRequest {
    fn operation(&self) -> Operation {
        Operation::GetCredential
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Credential),
            name,
        }
    }
}

impl Authorizable for UpdateCredentialRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateCredential
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Credential),
            name,
        }
    }
}

impl Authorizable for DeleteCredentialRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteCredential
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Credential),
            name,
        }
    }
}
