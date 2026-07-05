pub use super::_gen::ObjectLabel;
pub use olai_store::{EMPTY_RESOURCE_NAME, ResourceName, ResourceRef};

// Re-export the resource_name! macro from name.rs (which wraps the derive macro)
pub use name::resource_name;

mod name;

/// Resource that a policy can authorize.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceIdent {
    Agent(ResourceRef),
    AgentSkill(ResourceRef),
    Share(ResourceRef),
    Credential(ResourceRef),
    ExternalLocation(ResourceRef),
    Catalog(ResourceRef),
    Schema(ResourceRef),
    Table(ResourceRef),
    Recipient(ResourceRef),
    Provider(ResourceRef),
    Column(ResourceRef),
    Volume(ResourceRef),
    Function(ResourceRef),
    TagPolicy(ResourceRef),
    StagingTable(ResourceRef),
    PolicyInfo(ResourceRef),
}

impl ResourceIdent {
    pub fn label(&self) -> &ObjectLabel {
        self.as_ref()
    }

    pub fn reference(&self) -> &ResourceRef {
        self.as_ref()
    }

    pub fn agent(name: impl Into<ResourceRef>) -> Self {
        Self::Agent(name.into())
    }

    pub fn agent_skill(name: impl Into<ResourceRef>) -> Self {
        Self::AgentSkill(name.into())
    }

    pub fn share(name: impl Into<ResourceRef>) -> Self {
        Self::Share(name.into())
    }

    pub fn credential(name: impl Into<ResourceRef>) -> Self {
        Self::Credential(name.into())
    }

    pub fn catalog(name: impl Into<ResourceRef>) -> Self {
        Self::Catalog(name.into())
    }

    pub fn schema(name: impl Into<ResourceRef>) -> Self {
        Self::Schema(name.into())
    }

    pub fn table(name: impl Into<ResourceRef>) -> Self {
        Self::Table(name.into())
    }

    pub fn column(name: impl Into<ResourceRef>) -> Self {
        Self::Column(name.into())
    }

    pub fn external_location(name: impl Into<ResourceRef>) -> Self {
        Self::ExternalLocation(name.into())
    }

    pub fn recipient(name: impl Into<ResourceRef>) -> Self {
        Self::Recipient(name.into())
    }

    pub fn provider(name: impl Into<ResourceRef>) -> Self {
        Self::Provider(name.into())
    }

    pub fn volume(name: impl Into<ResourceRef>) -> Self {
        Self::Volume(name.into())
    }

    pub fn function(name: impl Into<ResourceRef>) -> Self {
        Self::Function(name.into())
    }

    pub fn tag_policy(name: impl Into<ResourceRef>) -> Self {
        Self::TagPolicy(name.into())
    }

    pub fn staging_table(name: impl Into<ResourceRef>) -> Self {
        Self::StagingTable(name.into())
    }

    pub fn policy_info(name: impl Into<ResourceRef>) -> Self {
        Self::PolicyInfo(name.into())
    }
}

impl std::fmt::Display for ResourceIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceIdent::Agent(r) => write!(f, "agent:{}", r),
            ResourceIdent::AgentSkill(r) => write!(f, "agent_skill:{}", r),
            ResourceIdent::Share(r) => write!(f, "share:{}", r),
            ResourceIdent::Credential(r) => write!(f, "credential:{}", r),
            ResourceIdent::ExternalLocation(r) => write!(f, "external_location:{}", r),
            ResourceIdent::Catalog(r) => write!(f, "catalog:{}", r),
            ResourceIdent::Schema(r) => write!(f, "schema:{}", r),
            ResourceIdent::Table(r) => write!(f, "table:{}", r),
            ResourceIdent::Recipient(r) => write!(f, "recipient:{}", r),
            ResourceIdent::Provider(r) => write!(f, "provider:{}", r),
            ResourceIdent::Column(r) => write!(f, "column:{}", r),
            ResourceIdent::Volume(r) => write!(f, "volume:{}", r),
            ResourceIdent::Function(r) => write!(f, "function:{}", r),
            ResourceIdent::TagPolicy(r) => write!(f, "tag_policy:{}", r),
            ResourceIdent::StagingTable(r) => write!(f, "staging_table:{}", r),
            ResourceIdent::PolicyInfo(r) => write!(f, "policy_info:{}", r),
        }
    }
}

impl AsRef<ResourceRef> for ResourceIdent {
    fn as_ref(&self) -> &ResourceRef {
        match self {
            ResourceIdent::Agent(r) => r,
            ResourceIdent::AgentSkill(r) => r,
            ResourceIdent::Share(r) => r,
            ResourceIdent::Credential(r) => r,
            ResourceIdent::ExternalLocation(r) => r,
            ResourceIdent::Catalog(r) => r,
            ResourceIdent::Schema(r) => r,
            ResourceIdent::Table(r) => r,
            ResourceIdent::Recipient(r) => r,
            ResourceIdent::Provider(r) => r,
            ResourceIdent::Column(r) => r,
            ResourceIdent::Volume(r) => r,
            ResourceIdent::Function(r) => r,
            ResourceIdent::TagPolicy(r) => r,
            ResourceIdent::StagingTable(r) => r,
            ResourceIdent::PolicyInfo(r) => r,
        }
    }
}

impl AsRef<ObjectLabel> for ResourceIdent {
    fn as_ref(&self) -> &ObjectLabel {
        match self {
            ResourceIdent::Agent(_) => &ObjectLabel::Agent,
            ResourceIdent::AgentSkill(_) => &ObjectLabel::AgentSkill,
            ResourceIdent::Share(_) => &ObjectLabel::Share,
            ResourceIdent::Credential(_) => &ObjectLabel::Credential,
            ResourceIdent::ExternalLocation(_) => &ObjectLabel::ExternalLocation,
            ResourceIdent::Catalog(_) => &ObjectLabel::Catalog,
            ResourceIdent::Schema(_) => &ObjectLabel::Schema,
            ResourceIdent::Table(_) => &ObjectLabel::Table,
            ResourceIdent::Recipient(_) => &ObjectLabel::Recipient,
            ResourceIdent::Provider(_) => &ObjectLabel::Provider,
            ResourceIdent::Column(_) => &ObjectLabel::Column,
            ResourceIdent::Volume(_) => &ObjectLabel::Volume,
            ResourceIdent::Function(_) => &ObjectLabel::Function,
            ResourceIdent::TagPolicy(_) => &ObjectLabel::TagPolicy,
            ResourceIdent::StagingTable(_) => &ObjectLabel::StagingTable,
            ResourceIdent::PolicyInfo(_) => &ObjectLabel::PolicyInfo,
        }
    }
}

impl From<ResourceIdent> for ResourceRef {
    fn from(ident: ResourceIdent) -> Self {
        match ident {
            ResourceIdent::Agent(r) => r,
            ResourceIdent::AgentSkill(r) => r,
            ResourceIdent::Share(r) => r,
            ResourceIdent::Credential(r) => r,
            ResourceIdent::ExternalLocation(r) => r,
            ResourceIdent::Catalog(r) => r,
            ResourceIdent::Schema(r) => r,
            ResourceIdent::Table(r) => r,
            ResourceIdent::Recipient(r) => r,
            ResourceIdent::Provider(r) => r,
            ResourceIdent::Column(r) => r,
            ResourceIdent::Volume(r) => r,
            ResourceIdent::Function(r) => r,
            ResourceIdent::TagPolicy(r) => r,
            ResourceIdent::StagingTable(r) => r,
            ResourceIdent::PolicyInfo(r) => r,
        }
    }
}

impl From<&ResourceIdent> for ResourceRef {
    fn from(ident: &ResourceIdent) -> Self {
        (ident as &dyn AsRef<ResourceRef>).as_ref().clone()
    }
}

impl From<&ResourceIdent> for ObjectLabel {
    fn from(ident: &ResourceIdent) -> Self {
        *(ident as &dyn AsRef<ObjectLabel>).as_ref()
    }
}

impl From<ResourceIdent> for ObjectLabel {
    fn from(ident: ResourceIdent) -> Self {
        (&ident).into()
    }
}

pub trait ResourceExt {
    /// Get the name of the resource
    fn resource_name(&self) -> ResourceName;

    /// Get the reference for the resource
    ///
    /// Depending on the resource type, this may be a UUID or a name.
    /// If possible, implementations should prefer to use the UUID
    /// as it is globally unique. However not all resource-like objects
    /// have a UUID field, or the UUID field may be optional.
    fn resource_ref(&self) -> ResourceRef;

    /// Get the ident for the resource
    fn resource_ident(&self) -> ResourceIdent;
}

impl<T: ResourceExt> From<&T> for ResourceIdent {
    fn from(resource: &T) -> Self {
        resource.resource_ident()
    }
}

impl ObjectLabel {
    pub fn to_ident(&self, id: impl Into<ResourceRef>) -> ResourceIdent {
        match self {
            ObjectLabel::Agent => ResourceIdent::agent(id),
            ObjectLabel::AgentSkill => ResourceIdent::agent_skill(id),
            ObjectLabel::Share => ResourceIdent::share(id),
            ObjectLabel::Credential => ResourceIdent::credential(id),
            ObjectLabel::Catalog => ResourceIdent::catalog(id),
            ObjectLabel::Schema => ResourceIdent::schema(id),
            ObjectLabel::Table => ResourceIdent::table(id),
            ObjectLabel::ExternalLocation => ResourceIdent::external_location(id),
            ObjectLabel::Recipient => ResourceIdent::recipient(id),
            ObjectLabel::Provider => ResourceIdent::provider(id),
            ObjectLabel::Column => ResourceIdent::column(id),
            ObjectLabel::Volume => ResourceIdent::volume(id),
            ObjectLabel::Function => ResourceIdent::function(id),
            ObjectLabel::TagPolicy => ResourceIdent::tag_policy(id),
            ObjectLabel::StagingTable => ResourceIdent::staging_table(id),
            ObjectLabel::PolicyInfo => ResourceIdent::policy_info(id),
        }
    }
}
