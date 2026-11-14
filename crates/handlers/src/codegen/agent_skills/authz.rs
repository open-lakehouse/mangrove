// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::agent_skills::v0alpha1::*;

impl Authorizable for ListAgentSkillsRequest {
    fn operation(&self) -> Operation {
        Operation::ListAgentSkills
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

impl Authorizable for CreateAgentSkillRequest {
    fn operation(&self) -> Operation {
        Operation::CreateAgentSkill
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        name.extend(self.schema_name.split('.').map(str::to_string));
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::AgentSkill),
            name,
        }
    }
}

impl Authorizable for GetAgentSkillRequest {
    fn operation(&self) -> Operation {
        Operation::GetAgentSkill
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::AgentSkill),
            name,
        }
    }
}

impl Authorizable for UpdateAgentSkillRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateAgentSkill
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::AgentSkill),
            name,
        }
    }
}

impl Authorizable for DeleteAgentSkillRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteAgentSkill
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::AgentSkill),
            name,
        }
    }
}
