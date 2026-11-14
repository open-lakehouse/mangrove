// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::shares::v1::*;

impl Authorizable for ListSharesRequest {
    fn operation(&self) -> Operation {
        Operation::ListShares
    }

    fn securable(&self) -> Securable {
        Securable {
            label: None,
            name: Vec::new(),
        }
    }
}

impl Authorizable for CreateShareRequest {
    fn operation(&self) -> Operation {
        Operation::CreateShare
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Share),
            name,
        }
    }
}

impl Authorizable for GetShareRequest {
    fn operation(&self) -> Operation {
        Operation::GetShare
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Share),
            name,
        }
    }
}

impl Authorizable for UpdateShareRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateShare
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Share),
            name,
        }
    }
}

impl Authorizable for DeleteShareRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteShare
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Share),
            name,
        }
    }
}

impl Authorizable for GetPermissionsRequest {
    fn operation(&self) -> Operation {
        Operation::GetPermissions
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Share),
            name,
        }
    }
}

impl Authorizable for UpdatePermissionsRequest {
    fn operation(&self) -> Operation {
        Operation::UpdatePermissions
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Share),
            name,
        }
    }
}
