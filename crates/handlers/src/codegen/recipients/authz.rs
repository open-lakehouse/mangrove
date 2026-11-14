// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::recipients::v1::*;

impl Authorizable for ListRecipientsRequest {
    fn operation(&self) -> Operation {
        Operation::ListRecipients
    }

    fn securable(&self) -> Securable {
        Securable {
            label: None,
            name: Vec::new(),
        }
    }
}

impl Authorizable for CreateRecipientRequest {
    fn operation(&self) -> Operation {
        Operation::CreateRecipient
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Recipient),
            name,
        }
    }
}

impl Authorizable for GetRecipientRequest {
    fn operation(&self) -> Operation {
        Operation::GetRecipient
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Recipient),
            name,
        }
    }
}

impl Authorizable for UpdateRecipientRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateRecipient
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Recipient),
            name,
        }
    }
}

impl Authorizable for DeleteRecipientRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteRecipient
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Recipient),
            name,
        }
    }
}

impl Authorizable for RotateRecipientTokenRequest {
    fn operation(&self) -> Operation {
        Operation::RotateRecipientToken
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Recipient),
            name,
        }
    }
}
