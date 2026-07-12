// @generated — do not edit by hand.
/// All resource types managed by the service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Debug, PartialEq)]
pub enum Resource {
    Agent(super::agents::v0alpha1::Agent),
    AgentSkill(super::agent_skills::v0alpha1::AgentSkill),
    Catalog(super::catalogs::v1::Catalog),
    Column(super::tables::v1::Column),
    Credential(super::credentials::v1::Credential),
    ExternalLocation(super::external_locations::v1::ExternalLocation),
    Function(super::functions::v1::Function),
    PolicyInfo(super::policies::v1::PolicyInfo),
    Provider(super::providers::v1::Provider),
    Recipient(super::recipients::v1::Recipient),
    Schema(super::schemas::v1::Schema),
    Share(super::shares::v1::Share),
    StagingTable(super::staging_tables::v1::StagingTable),
    Table(super::tables::v1::Table),
    TagPolicy(super::tags::v1::TagPolicy),
    Volume(super::volumes::v1::Volume),
}
/// Discriminant label for each resource type.
#[derive(
    ::strum::AsRefStr,
    ::strum::Display,
    ::strum::EnumIter,
    ::strum::EnumString,
    ::serde::Serialize,
    ::serde::Deserialize,
    Hash,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "sqlx", derive(::sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "object_label", rename_all = "snake_case")
)]
pub enum ObjectLabel {
    Agent,
    AgentSkill,
    Catalog,
    Column,
    Credential,
    ExternalLocation,
    Function,
    PolicyInfo,
    Provider,
    Recipient,
    Schema,
    Share,
    StagingTable,
    Table,
    TagPolicy,
    Volume,
}
impl Resource {
    /// Return the discriminant label for this resource.
    pub fn resource_label(&self) -> &ObjectLabel {
        match self {
            Resource::Agent(_) => &ObjectLabel::Agent,
            Resource::AgentSkill(_) => &ObjectLabel::AgentSkill,
            Resource::Catalog(_) => &ObjectLabel::Catalog,
            Resource::Column(_) => &ObjectLabel::Column,
            Resource::Credential(_) => &ObjectLabel::Credential,
            Resource::ExternalLocation(_) => &ObjectLabel::ExternalLocation,
            Resource::Function(_) => &ObjectLabel::Function,
            Resource::PolicyInfo(_) => &ObjectLabel::PolicyInfo,
            Resource::Provider(_) => &ObjectLabel::Provider,
            Resource::Recipient(_) => &ObjectLabel::Recipient,
            Resource::Schema(_) => &ObjectLabel::Schema,
            Resource::Share(_) => &ObjectLabel::Share,
            Resource::StagingTable(_) => &ObjectLabel::StagingTable,
            Resource::Table(_) => &ObjectLabel::Table,
            Resource::TagPolicy(_) => &ObjectLabel::TagPolicy,
            Resource::Volume(_) => &ObjectLabel::Volume,
        }
    }
}
impl From<super::agents::v0alpha1::Agent> for Resource {
    fn from(v: super::agents::v0alpha1::Agent) -> Self {
        Resource::Agent(v)
    }
}
impl TryFrom<Resource> for super::agents::v0alpha1::Agent {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Agent(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Agent)
            ))),
        }
    }
}
impl From<super::agent_skills::v0alpha1::AgentSkill> for Resource {
    fn from(v: super::agent_skills::v0alpha1::AgentSkill) -> Self {
        Resource::AgentSkill(v)
    }
}
impl TryFrom<Resource> for super::agent_skills::v0alpha1::AgentSkill {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::AgentSkill(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(AgentSkill)
            ))),
        }
    }
}
impl From<super::catalogs::v1::Catalog> for Resource {
    fn from(v: super::catalogs::v1::Catalog) -> Self {
        Resource::Catalog(v)
    }
}
impl TryFrom<Resource> for super::catalogs::v1::Catalog {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Catalog(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Catalog)
            ))),
        }
    }
}
impl From<super::tables::v1::Column> for Resource {
    fn from(v: super::tables::v1::Column) -> Self {
        Resource::Column(v)
    }
}
impl TryFrom<Resource> for super::tables::v1::Column {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Column(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Column)
            ))),
        }
    }
}
impl From<super::credentials::v1::Credential> for Resource {
    fn from(v: super::credentials::v1::Credential) -> Self {
        Resource::Credential(v)
    }
}
impl TryFrom<Resource> for super::credentials::v1::Credential {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Credential(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Credential)
            ))),
        }
    }
}
impl From<super::external_locations::v1::ExternalLocation> for Resource {
    fn from(v: super::external_locations::v1::ExternalLocation) -> Self {
        Resource::ExternalLocation(v)
    }
}
impl TryFrom<Resource> for super::external_locations::v1::ExternalLocation {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::ExternalLocation(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(ExternalLocation)
            ))),
        }
    }
}
impl From<super::functions::v1::Function> for Resource {
    fn from(v: super::functions::v1::Function) -> Self {
        Resource::Function(v)
    }
}
impl TryFrom<Resource> for super::functions::v1::Function {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Function(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Function)
            ))),
        }
    }
}
impl From<super::policies::v1::PolicyInfo> for Resource {
    fn from(v: super::policies::v1::PolicyInfo) -> Self {
        Resource::PolicyInfo(v)
    }
}
impl TryFrom<Resource> for super::policies::v1::PolicyInfo {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::PolicyInfo(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(PolicyInfo)
            ))),
        }
    }
}
impl From<super::providers::v1::Provider> for Resource {
    fn from(v: super::providers::v1::Provider) -> Self {
        Resource::Provider(v)
    }
}
impl TryFrom<Resource> for super::providers::v1::Provider {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Provider(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Provider)
            ))),
        }
    }
}
impl From<super::recipients::v1::Recipient> for Resource {
    fn from(v: super::recipients::v1::Recipient) -> Self {
        Resource::Recipient(v)
    }
}
impl TryFrom<Resource> for super::recipients::v1::Recipient {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Recipient(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Recipient)
            ))),
        }
    }
}
impl From<super::schemas::v1::Schema> for Resource {
    fn from(v: super::schemas::v1::Schema) -> Self {
        Resource::Schema(v)
    }
}
impl TryFrom<Resource> for super::schemas::v1::Schema {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Schema(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Schema)
            ))),
        }
    }
}
impl From<super::shares::v1::Share> for Resource {
    fn from(v: super::shares::v1::Share) -> Self {
        Resource::Share(v)
    }
}
impl TryFrom<Resource> for super::shares::v1::Share {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Share(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Share)
            ))),
        }
    }
}
impl From<super::staging_tables::v1::StagingTable> for Resource {
    fn from(v: super::staging_tables::v1::StagingTable) -> Self {
        Resource::StagingTable(v)
    }
}
impl TryFrom<Resource> for super::staging_tables::v1::StagingTable {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::StagingTable(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(StagingTable)
            ))),
        }
    }
}
impl From<super::tables::v1::Table> for Resource {
    fn from(v: super::tables::v1::Table) -> Self {
        Resource::Table(v)
    }
}
impl TryFrom<Resource> for super::tables::v1::Table {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Table(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Table)
            ))),
        }
    }
}
impl From<super::tags::v1::TagPolicy> for Resource {
    fn from(v: super::tags::v1::TagPolicy) -> Self {
        Resource::TagPolicy(v)
    }
}
impl TryFrom<Resource> for super::tags::v1::TagPolicy {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::TagPolicy(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(TagPolicy)
            ))),
        }
    }
}
impl From<super::volumes::v1::Volume> for Resource {
    fn from(v: super::volumes::v1::Volume) -> Self {
        Resource::Volume(v)
    }
}
impl TryFrom<Resource> for super::volumes::v1::Volume {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Volume(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Volume)
            ))),
        }
    }
}
use crate::Error;
use crate::models::object::Object;
use crate::models::resources::{ResourceExt, ResourceIdent, ResourceName, ResourceRef};
impl TryFrom<Object> for super::agents::v0alpha1::Agent {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::agents::v0alpha1::Agent = ::serde_json::from_value(props)?;
        res.agent_id = object.id.hyphenated().to_string();
        Ok(res)
    }
}
impl TryFrom<super::agents::v0alpha1::Agent> for Object {
    type Error = Error;
    fn try_from(obj: super::agents::v0alpha1::Agent) -> Result<Self, Self::Error> {
        let id = ::uuid::Uuid::parse_str(&obj.agent_id).unwrap_or_else(|_| ::uuid::Uuid::nil());
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::Agent,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::agents::v0alpha1::Agent {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.catalog_name, &self.schema_name, &self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        ::uuid::Uuid::parse_str(&self.agent_id)
            .ok()
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::Agent).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::agent_skills::v0alpha1::AgentSkill {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::agent_skills::v0alpha1::AgentSkill = ::serde_json::from_value(props)?;
        res.agent_skill_id = object.id.hyphenated().to_string();
        Ok(res)
    }
}
impl TryFrom<super::agent_skills::v0alpha1::AgentSkill> for Object {
    type Error = Error;
    fn try_from(obj: super::agent_skills::v0alpha1::AgentSkill) -> Result<Self, Self::Error> {
        let id =
            ::uuid::Uuid::parse_str(&obj.agent_skill_id).unwrap_or_else(|_| ::uuid::Uuid::nil());
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::AgentSkill,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::agent_skills::v0alpha1::AgentSkill {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.catalog_name, &self.schema_name, &self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        ::uuid::Uuid::parse_str(&self.agent_skill_id)
            .ok()
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::AgentSkill).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::catalogs::v1::Catalog {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::catalogs::v1::Catalog = ::serde_json::from_value(props)?;
        res.id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::catalogs::v1::Catalog> for Object {
    type Error = Error;
    fn try_from(obj: super::catalogs::v1::Catalog) -> Result<Self, Self::Error> {
        let id = obj
            .id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::Catalog,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::catalogs::v1::Catalog {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::Catalog).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::tables::v1::Column {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::tables::v1::Column = ::serde_json::from_value(props)?;
        res.column_id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::tables::v1::Column> for Object {
    type Error = Error;
    fn try_from(obj: super::tables::v1::Column) -> Result<Self, Self::Error> {
        let id = obj
            .column_id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::Column,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::tables::v1::Column {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.column_id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::Column).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::credentials::v1::Credential {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::credentials::v1::Credential = ::serde_json::from_value(props)?;
        res.id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::credentials::v1::Credential> for Object {
    type Error = Error;
    fn try_from(obj: super::credentials::v1::Credential) -> Result<Self, Self::Error> {
        let id = obj
            .id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::Credential,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::credentials::v1::Credential {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::Credential).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::external_locations::v1::ExternalLocation {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::external_locations::v1::ExternalLocation =
            ::serde_json::from_value(props)?;
        res.external_location_id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::external_locations::v1::ExternalLocation> for Object {
    type Error = Error;
    fn try_from(obj: super::external_locations::v1::ExternalLocation) -> Result<Self, Self::Error> {
        let id = obj
            .external_location_id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::ExternalLocation,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::external_locations::v1::ExternalLocation {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.external_location_id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::ExternalLocation).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::functions::v1::Function {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::functions::v1::Function = ::serde_json::from_value(props)?;
        res.function_id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::functions::v1::Function> for Object {
    type Error = Error;
    fn try_from(obj: super::functions::v1::Function) -> Result<Self, Self::Error> {
        let id = obj
            .function_id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::Function,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::functions::v1::Function {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.catalog_name, &self.schema_name, &self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.function_id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::Function).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::policies::v1::PolicyInfo {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::policies::v1::PolicyInfo = ::serde_json::from_value(props)?;
        res.id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::policies::v1::PolicyInfo> for Object {
    type Error = Error;
    fn try_from(obj: super::policies::v1::PolicyInfo) -> Result<Self, Self::Error> {
        let id = obj
            .id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::PolicyInfo,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::policies::v1::PolicyInfo {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::PolicyInfo).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::providers::v1::Provider {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::providers::v1::Provider = ::serde_json::from_value(props)?;
        res.id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::providers::v1::Provider> for Object {
    type Error = Error;
    fn try_from(obj: super::providers::v1::Provider) -> Result<Self, Self::Error> {
        let id = obj
            .id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::Provider,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::providers::v1::Provider {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::Provider).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::recipients::v1::Recipient {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::recipients::v1::Recipient = ::serde_json::from_value(props)?;
        res.id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::recipients::v1::Recipient> for Object {
    type Error = Error;
    fn try_from(obj: super::recipients::v1::Recipient) -> Result<Self, Self::Error> {
        let id = obj
            .id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::Recipient,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::recipients::v1::Recipient {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::Recipient).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::schemas::v1::Schema {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::schemas::v1::Schema = ::serde_json::from_value(props)?;
        res.schema_id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::schemas::v1::Schema> for Object {
    type Error = Error;
    fn try_from(obj: super::schemas::v1::Schema) -> Result<Self, Self::Error> {
        let id = obj
            .schema_id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::Schema,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::schemas::v1::Schema {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.catalog_name, &self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.schema_id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::Schema).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::shares::v1::Share {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::shares::v1::Share = ::serde_json::from_value(props)?;
        res.id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::shares::v1::Share> for Object {
    type Error = Error;
    fn try_from(obj: super::shares::v1::Share) -> Result<Self, Self::Error> {
        let id = obj
            .id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::Share,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::shares::v1::Share {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::Share).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::staging_tables::v1::StagingTable {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::staging_tables::v1::StagingTable = ::serde_json::from_value(props)?;
        res.id = object.id.hyphenated().to_string();
        Ok(res)
    }
}
impl TryFrom<super::staging_tables::v1::StagingTable> for Object {
    type Error = Error;
    fn try_from(obj: super::staging_tables::v1::StagingTable) -> Result<Self, Self::Error> {
        let id = ::uuid::Uuid::parse_str(&obj.id).unwrap_or_else(|_| ::uuid::Uuid::nil());
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::StagingTable,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::staging_tables::v1::StagingTable {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        ::uuid::Uuid::parse_str(&self.id)
            .ok()
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::StagingTable).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::tables::v1::Table {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::tables::v1::Table = ::serde_json::from_value(props)?;
        res.table_id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::tables::v1::Table> for Object {
    type Error = Error;
    fn try_from(obj: super::tables::v1::Table) -> Result<Self, Self::Error> {
        let id = obj
            .table_id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::Table,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::tables::v1::Table {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.catalog_name, &self.schema_name, &self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.table_id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::Table).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::tags::v1::TagPolicy {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::tags::v1::TagPolicy = ::serde_json::from_value(props)?;
        res.id = Some(object.id.hyphenated().to_string());
        Ok(res)
    }
}
impl TryFrom<super::tags::v1::TagPolicy> for Object {
    type Error = Error;
    fn try_from(obj: super::tags::v1::TagPolicy) -> Result<Self, Self::Error> {
        let id = obj
            .id
            .as_ref()
            .map(|id| ::uuid::Uuid::parse_str(id))
            .transpose()?
            .unwrap_or_else(::uuid::Uuid::nil);
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::TagPolicy,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::tags::v1::TagPolicy {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.tag_key])
    }
    fn resource_ref(&self) -> ResourceRef {
        self.id
            .as_ref()
            .and_then(|id| ::uuid::Uuid::parse_str(id).ok())
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::TagPolicy).to_ident(self.resource_ref())
    }
}
impl TryFrom<Object> for super::volumes::v1::Volume {
    type Error = Error;
    fn try_from(object: Object) -> Result<Self, Self::Error> {
        let props = object
            .properties
            .ok_or_else(|| Error::generic("expected properties"))?;
        let mut res: super::volumes::v1::Volume = ::serde_json::from_value(props)?;
        res.volume_id = object.id.hyphenated().to_string();
        Ok(res)
    }
}
impl TryFrom<super::volumes::v1::Volume> for Object {
    type Error = Error;
    fn try_from(obj: super::volumes::v1::Volume) -> Result<Self, Self::Error> {
        let id = ::uuid::Uuid::parse_str(&obj.volume_id).unwrap_or_else(|_| ::uuid::Uuid::nil());
        Ok(Object {
            id,
            name: obj.resource_name(),
            label: ObjectLabel::Volume,
            properties: Some(::serde_json::to_value(obj)?),
            version: 0,
            updated_at: None,
            created_at: chrono::Utc::now(),
        })
    }
}
impl ResourceExt for super::volumes::v1::Volume {
    fn resource_name(&self) -> ResourceName {
        ResourceName::new([&self.catalog_name, &self.schema_name, &self.name])
    }
    fn resource_ref(&self) -> ResourceRef {
        ::uuid::Uuid::parse_str(&self.volume_id)
            .ok()
            .map(ResourceRef::Uuid)
            .unwrap_or_else(|| ResourceRef::Name(self.resource_name()))
    }
    fn resource_ident(&self) -> ResourceIdent {
        (ObjectLabel::Volume).to_ident(self.resource_ref())
    }
}
impl super::agents::v0alpha1::Agent {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}.{}", self.catalog_name, self.schema_name, self.name)
    }
}
impl super::agent_skills::v0alpha1::AgentSkill {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}.{}", self.catalog_name, self.schema_name, self.name)
    }
}
impl super::catalogs::v1::Catalog {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::tables::v1::Column {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::credentials::v1::Credential {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::external_locations::v1::ExternalLocation {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::functions::v1::Function {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}.{}", self.catalog_name, self.schema_name, self.name)
    }
}
impl super::policies::v1::PolicyInfo {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::providers::v1::Provider {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::recipients::v1::Recipient {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::schemas::v1::Schema {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.catalog_name, self.name)
    }
}
impl super::shares::v1::Share {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::staging_tables::v1::StagingTable {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::tables::v1::Table {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}.{}", self.catalog_name, self.schema_name, self.name)
    }
}
impl super::tags::v1::TagPolicy {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.tag_key.clone()
    }
}
impl super::volumes::v1::Volume {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}.{}", self.catalog_name, self.schema_name, self.name)
    }
}
impl ::olai_store::Label for ObjectLabel {
    fn as_str(&self) -> &str {
        self.as_ref()
    }
}
/// Static resource type descriptors derived from proto annotations.
///
/// Each entry describes a resource type's fields (with roles: data, identifier,
/// sensitive, managed), hierarchical name components, and parent relationship.
///
/// Use `ResourceRegistry::from_static` to build a runtime registry from this data.
pub static RESOURCE_DESCRIPTORS: &[::olai_store::ResourceTypeDescriptor<ObjectLabel>] = &[
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::Agent,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "catalog_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "schema_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "agent_id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "invocation_protocol",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "endpoint",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "description",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "capabilities",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "input_schema",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "metastore_id",
                role: ::olai_store::FieldRole::Data,
            },
        ],
        path_names: &["catalog_name", "schema_name", "name"],
        parent_label: Some(ObjectLabel::Catalog),
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::AgentSkill,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "catalog_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "schema_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "storage_location",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "agent_skill_id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "agent_skill_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "description",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "license",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "allowed_tools",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "metadata",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "metastore_id",
                role: ::olai_store::FieldRole::Data,
            },
        ],
        path_names: &["catalog_name", "schema_name", "name"],
        parent_label: Some(ObjectLabel::Catalog),
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::Catalog,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "properties",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "storage_root",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "provider_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "share_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "catalog_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "storage_location",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "browse_only",
                role: ::olai_store::FieldRole::Data,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::Column,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "type_text",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "type_json",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "position",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "type_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "type_precision",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "type_scale",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "type_interval_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "nullable",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "partition_index",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "column_id",
                role: ::olai_store::FieldRole::Identifier,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::Credential,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "purpose",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "read_only",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "used_for_managed_storage",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "azure_service_principal",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "azure_managed_identity",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "azure_storage_key",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "aws_iam_role",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "databricks_gcp_service_account",
                role: ::olai_store::FieldRole::Data,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::ExternalLocation,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "url",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "credential_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "read_only",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "credential_id",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "browse_only",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "external_location_id",
                role: ::olai_store::FieldRole::Identifier,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::Function,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "catalog_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "schema_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "data_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "full_data_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "input_params",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "return_params",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "routine_body_language",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "routine_definition",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "routine_dependencies",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "parameter_style",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "is_deterministic",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "sql_data_access",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "is_null_call",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "security_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "specific_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "routine_body",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "properties",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "function_id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
        ],
        path_names: &["catalog_name", "schema_name", "name"],
        parent_label: Some(ObjectLabel::Catalog),
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::PolicyInfo,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "on_securable_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "on_securable_fullname",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "policy_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "to_principals",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "except_principals",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "when_condition",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "match_columns",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "function",
                role: ::olai_store::FieldRole::Data,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::Provider,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "authentication_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "recipient_profile_str",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "properties",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::Recipient,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "authentication_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "properties",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "tokens",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::Schema,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "catalog_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "properties",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "schema_id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "storage_root",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "storage_location",
                role: ::olai_store::FieldRole::Data,
            },
        ],
        path_names: &["catalog_name", "name"],
        parent_label: Some(ObjectLabel::Catalog),
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::Share,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "objects",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "storage_location",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "storage_root",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::StagingTable,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "schema_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "catalog_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "staging_location",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "stage_committed",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::Table,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "catalog_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "schema_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "table_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "data_source_format",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "columns",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "storage_location",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "view_definition",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "view_dependencies",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "properties",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "storage_credential_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "deleted_at",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "table_id",
                role: ::olai_store::FieldRole::Identifier,
            },
        ],
        path_names: &["catalog_name", "schema_name", "name"],
        parent_label: Some(ObjectLabel::Catalog),
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::TagPolicy,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "tag_key",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "description",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "values",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
        ],
        path_names: &["tag_key"],
        parent_label: None,
    },
    ::olai_store::ResourceTypeDescriptor {
        label: ObjectLabel::Volume,
        fields: &[
            ::olai_store::ResourceFieldDescriptor {
                name: "name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "catalog_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "schema_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "storage_location",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "volume_id",
                role: ::olai_store::FieldRole::Identifier,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "volume_type",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::olai_store::FieldRole::Managed,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "browse_only",
                role: ::olai_store::FieldRole::Data,
            },
            ::olai_store::ResourceFieldDescriptor {
                name: "metastore_id",
                role: ::olai_store::FieldRole::Data,
            },
        ],
        path_names: &["catalog_name", "schema_name", "name"],
        parent_label: Some(ObjectLabel::Catalog),
    },
];
