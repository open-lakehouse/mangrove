use crate::Error;
use crate::models::{
    ObjectLabel, Resource, ResourceExt, ResourceIdent, ResourceName, ResourceRef, Table,
};

use super::tables::v1::TableSummary;

/// Project-specific alias for `resource_store::Object` with our generated `ObjectLabel`.
pub type Object = olai_store::Object<ObjectLabel>;

impl ResourceExt for Object {
    fn resource_name(&self) -> ResourceName {
        self.name.clone()
    }

    fn resource_ref(&self) -> ResourceRef {
        ResourceRef::Uuid(self.id)
    }

    fn resource_ident(&self) -> ResourceIdent {
        self.label.to_ident(self.id)
    }
}

impl ResourceExt for Resource {
    fn resource_name(&self) -> ResourceName {
        match self {
            Resource::Agent(obj) => obj.resource_name(),
            Resource::AgentSkill(obj) => obj.resource_name(),
            Resource::Share(obj) => obj.resource_name(),
            Resource::Credential(obj) => obj.resource_name(),
            Resource::Catalog(obj) => obj.resource_name(),
            Resource::Schema(obj) => obj.resource_name(),
            Resource::Table(obj) => obj.resource_name(),
            Resource::ExternalLocation(obj) => obj.resource_name(),
            Resource::Recipient(obj) => obj.resource_name(),
            Resource::Provider(obj) => obj.resource_name(),
            Resource::Column(obj) => obj.resource_name(),
            Resource::Volume(obj) => obj.resource_name(),
            Resource::Function(obj) => obj.resource_name(),
            Resource::TagPolicy(obj) => obj.resource_name(),
            Resource::StagingTable(obj) => obj.resource_name(),
            Resource::PolicyInfo(obj) => obj.resource_name(),
        }
    }

    fn resource_ref(&self) -> ResourceRef {
        match self {
            Resource::Agent(obj) => obj.resource_ref(),
            Resource::AgentSkill(obj) => obj.resource_ref(),
            Resource::Share(obj) => obj.resource_ref(),
            Resource::Credential(obj) => obj.resource_ref(),
            Resource::Catalog(obj) => obj.resource_ref(),
            Resource::Schema(obj) => obj.resource_ref(),
            Resource::Table(obj) => obj.resource_ref(),
            Resource::ExternalLocation(obj) => obj.resource_ref(),
            Resource::Recipient(obj) => obj.resource_ref(),
            Resource::Provider(obj) => obj.resource_ref(),
            Resource::Column(obj) => obj.resource_ref(),
            Resource::Volume(obj) => obj.resource_ref(),
            Resource::Function(obj) => obj.resource_ref(),
            Resource::TagPolicy(obj) => obj.resource_ref(),
            Resource::StagingTable(obj) => obj.resource_ref(),
            Resource::PolicyInfo(obj) => obj.resource_ref(),
        }
    }

    fn resource_ident(&self) -> ResourceIdent {
        match self {
            Resource::Agent(obj) => obj.resource_ident(),
            Resource::AgentSkill(obj) => obj.resource_ident(),
            Resource::Share(obj) => obj.resource_ident(),
            Resource::Credential(obj) => obj.resource_ident(),
            Resource::Catalog(obj) => obj.resource_ident(),
            Resource::Schema(obj) => obj.resource_ident(),
            Resource::Table(obj) => obj.resource_ident(),
            Resource::ExternalLocation(obj) => obj.resource_ident(),
            Resource::Recipient(obj) => obj.resource_ident(),
            Resource::Provider(obj) => obj.resource_ident(),
            Resource::Column(obj) => obj.resource_ident(),
            Resource::Volume(obj) => obj.resource_ident(),
            Resource::Function(obj) => obj.resource_ident(),
            Resource::TagPolicy(obj) => obj.resource_ident(),
            Resource::StagingTable(obj) => obj.resource_ident(),
            Resource::PolicyInfo(obj) => obj.resource_ident(),
        }
    }
}

impl TryFrom<Resource> for Object {
    type Error = Error;

    fn try_from(resource: Resource) -> Result<Self, Self::Error> {
        match resource {
            Resource::Agent(obj) => obj.try_into(),
            Resource::AgentSkill(obj) => obj.try_into(),
            Resource::Share(obj) => obj.try_into(),
            Resource::Credential(obj) => obj.try_into(),
            Resource::Catalog(obj) => obj.try_into(),
            Resource::Schema(obj) => obj.try_into(),
            Resource::Table(obj) => obj.try_into(),
            Resource::ExternalLocation(obj) => obj.try_into(),
            Resource::Recipient(obj) => obj.try_into(),
            Resource::Provider(obj) => obj.try_into(),
            Resource::Column(obj) => obj.try_into(),
            Resource::Volume(obj) => obj.try_into(),
            Resource::Function(obj) => obj.try_into(),
            Resource::TagPolicy(obj) => obj.try_into(),
            Resource::StagingTable(obj) => obj.try_into(),
            Resource::PolicyInfo(obj) => obj.try_into(),
        }
    }
}

impl TryFrom<Object> for Resource {
    type Error = Error;

    fn try_from(obj: Object) -> Result<Self, Self::Error> {
        match obj.label {
            ObjectLabel::Agent => Ok(Resource::Agent(obj.try_into()?)),
            ObjectLabel::AgentSkill => Ok(Resource::AgentSkill(obj.try_into()?)),
            ObjectLabel::Share => Ok(Resource::Share(obj.try_into()?)),
            ObjectLabel::Credential => Ok(Resource::Credential(obj.try_into()?)),
            ObjectLabel::Catalog => Ok(Resource::Catalog(obj.try_into()?)),
            ObjectLabel::Schema => Ok(Resource::Schema(obj.try_into()?)),
            ObjectLabel::Table => Ok(Resource::Table(obj.try_into()?)),
            ObjectLabel::ExternalLocation => Ok(Resource::ExternalLocation(obj.try_into()?)),
            ObjectLabel::Recipient => Ok(Resource::Recipient(obj.try_into()?)),
            ObjectLabel::Provider => Ok(Resource::Provider(obj.try_into()?)),
            ObjectLabel::Column => Ok(Resource::Column(obj.try_into()?)),
            ObjectLabel::Volume => Ok(Resource::Volume(obj.try_into()?)),
            ObjectLabel::Function => Ok(Resource::Function(obj.try_into()?)),
            ObjectLabel::TagPolicy => Ok(Resource::TagPolicy(obj.try_into()?)),
            ObjectLabel::StagingTable => Ok(Resource::StagingTable(obj.try_into()?)),
            ObjectLabel::PolicyInfo => Ok(Resource::PolicyInfo(obj.try_into()?)),
        }
    }
}

impl From<Table> for TableSummary {
    fn from(table: Table) -> Self {
        TableSummary {
            table_type: table.table_type,
            full_name: table.full_name,
            ..Default::default()
        }
    }
}
