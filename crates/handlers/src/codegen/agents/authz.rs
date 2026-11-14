// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::agents::v0alpha1::*;

impl Authorizable for ListAgentsRequest {
    fn operation(&self) -> Operation {
        Operation::ListAgents
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        name.extend(self.schema_name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Schema),
            name,
        }
    }
}

impl Authorizable for CreateAgentRequest {
    fn operation(&self) -> Operation {
        Operation::CreateAgent
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        name.extend(self.schema_name.split('.').map(str::to_string));
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Agent),
            name,
        }
    }
}

impl Authorizable for GetAgentRequest {
    fn operation(&self) -> Operation {
        Operation::GetAgent
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Agent),
            name,
        }
    }
}

impl Authorizable for UpdateAgentRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateAgent
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Agent),
            name,
        }
    }
}

impl Authorizable for DeleteAgentRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteAgent
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Agent),
            name,
        }
    }
}
