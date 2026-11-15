// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::tags::v1::*;

impl Authorizable for ListTagPoliciesRequest {
    fn operation(&self) -> Operation {
        Operation::ListTagPolicies
    }

    fn securable(&self) -> Securable {
        Securable {
            label: None,
            name: Vec::new(),
        }
    }
}

impl Authorizable for CreateTagPolicyRequest {
    fn operation(&self) -> Operation {
        Operation::CreateTagPolicy
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.tag_key.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::TagPolicy),
            name,
        }
    }
}

impl Authorizable for GetTagPolicyRequest {
    fn operation(&self) -> Operation {
        Operation::GetTagPolicy
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.tag_key.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::TagPolicy),
            name,
        }
    }
}

impl Authorizable for UpdateTagPolicyRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateTagPolicy
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.tag_key.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::TagPolicy),
            name,
        }
    }
}

impl Authorizable for DeleteTagPolicyRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteTagPolicy
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.tag_key.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::TagPolicy),
            name,
        }
    }
}
