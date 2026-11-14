// @generated — do not edit by hand.
//! The authorization vocabulary of the API.
//!
//! Every RPC is an [`Operation`], named after it, and every request an [`Authorizable`]
//! that names its operation and the [`Securable`] it acts on. A policy decides whether a
//! caller may perform an operation on a securable; the Cedar actions of the operations
//! are in the sibling `authz.cedarschema`.

use unitycatalog_common::models::ObjectLabel;

/// An operation of the API: one per RPC, named after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Operation {
    /// Lists agent skills.
    ListAgentSkills,
    CreateAgentSkill,
    GetAgentSkill,
    UpdateAgentSkill,
    DeleteAgentSkill,
    /// Lists agents.
    ListAgents,
    CreateAgent,
    GetAgent,
    UpdateAgent,
    DeleteAgent,
    /// List catalogs
    ///
    /// Gets an array of catalogs in the metastore. If the caller is the metastore admin,
    /// all catalogs will be retrieved. Otherwise, only catalogs owned by the caller
    /// (or for which the caller has the USE_CATALOG privilege) will be retrieved.
    /// There is no guarantee of a specific ordering of the elements in the array.
    ListCatalogs,
    /// Create a new catalog
    ///
    /// Creates a new catalog instance in the parent metastore if the caller
    /// is a metastore admin or has the CREATE_CATALOG privilege.
    CreateCatalog,
    /// Get a catalog
    ///
    /// Gets the specified catalog in a metastore. The caller must be a metastore admin,
    /// the owner of the catalog, or a user that has the USE_CATALOG privilege set for their account.
    GetCatalog,
    /// Update a catalog
    ///
    /// Updates the catalog that matches the supplied name. The caller must be either
    /// the owner of the catalog, or a metastore admin (when changing the owner field of the catalog).
    UpdateCatalog,
    /// Delete a catalog
    ///
    /// Deletes the catalog that matches the supplied name. The caller must
    /// be a metastore admin or the owner of the catalog.
    DeleteCatalog,
    ListCredentials,
    CreateCredential,
    GetCredential,
    UpdateCredential,
    DeleteCredential,
    /// List entity tag assignments
    ///
    /// Gets the tag assignments for the specified entity.
    ListEntityTagAssignments,
    /// Create an entity tag assignment
    ///
    /// Assigns a tag to a Unity Catalog entity.
    CreateEntityTagAssignment,
    /// Get an entity tag assignment
    ///
    /// Gets the tag assignment for the specified entity and tag key.
    GetEntityTagAssignment,
    /// Update an entity tag assignment
    ///
    /// Updates the tag assignment for the specified entity and tag key.
    UpdateEntityTagAssignment,
    /// Delete an entity tag assignment
    ///
    /// Deletes the tag assignment for the specified entity and tag key.
    DeleteEntityTagAssignment,
    /// List external locations
    ListExternalLocations,
    /// Create a new external location
    CreateExternalLocation,
    /// Get an external location
    GetExternalLocation,
    /// Update an external location
    UpdateExternalLocation,
    /// Delete an external location
    DeleteExternalLocation,
    /// List functions
    ///
    /// List functions within the specified parent catalog and schema. If the caller is the metastore
    /// admin, all functions are returned in the response. Otherwise, the caller must have USE_CATALOG
    /// on the parent catalog and USE_SCHEMA on the parent schema, and the function must either be
    /// owned by the caller or have SELECT on the function.
    ListFunctions,
    /// Create a function
    ///
    /// Creates a new function. The caller must be a metastore admin or have the CREATE_FUNCTION
    /// privilege on the parent catalog and schema.
    CreateFunction,
    /// Get a function
    ///
    /// Gets a function from within a parent catalog and schema. For the fetch to succeed,
    /// the caller must be a metastore admin, the owner of the function, or have SELECT on
    /// the function.
    GetFunction,
    /// Update a function
    ///
    /// Updates the function that matches the supplied name. Only the owner of the function
    /// can be updated.
    UpdateFunction,
    /// Delete a function
    ///
    /// Deletes the function that matches the supplied name. For the deletion to succeed,
    /// the caller must be the owner of the function.
    DeleteFunction,
    /// Gets the privileges granted on a securable. Does not include inherited ones.
    GetGrants,
    /// Grants and revokes privileges on a securable.
    UpdateGrants,
    /// List model versions
    ///
    /// List the model versions of the specified registered model. If the caller is
    /// the metastore admin, all model versions are returned. Otherwise, the caller
    /// must have the appropriate privileges on the parent model.
    ListModelVersions,
    /// Create a model version
    ///
    /// Creates a new model version in PENDING_REGISTRATION status. The server
    /// assigns the version number and a storage location for the artifacts. The
    /// caller must be a metastore admin or the owner of the parent registered model.
    CreateModelVersion,
    /// Get a model version
    ///
    /// Gets a model version by its parent model name and version number.
    GetModelVersion,
    /// Update a model version
    ///
    /// Updates the model version that matches the supplied name and version.
    UpdateModelVersion,
    /// Delete a model version
    ///
    /// Deletes the model version that matches the supplied name and version. For the
    /// deletion to succeed, the caller must be the owner of the parent registered
    /// model.
    DeleteModelVersion,
    /// Finalize a model version
    ///
    /// Transitions a model version to READY once all artifacts have been written to
    /// its storage location.
    FinalizeModelVersion,
    /// List policies
    ///
    /// Gets an array of policies defined on the specified securable. There is no guarantee
    /// of a specific ordering of the elements in the array.
    ListPolicies,
    /// Create a new policy
    ///
    /// Creates a new row-filter or column-mask policy on the specified securable.
    CreatePolicy,
    /// Get a policy
    ///
    /// Gets the policy that matches the supplied name, defined on the specified securable.
    GetPolicy,
    /// Update a policy
    ///
    /// Updates the policy that matches the supplied name, defined on the specified securable.
    UpdatePolicy,
    /// Delete a policy
    ///
    /// Deletes the policy that matches the supplied name, defined on the specified securable.
    DeletePolicy,
    /// List providers.
    ListProviders,
    /// Create a new provider.
    CreateProvider,
    /// Get a provider by name.
    GetProvider,
    /// Update a provider.
    UpdateProvider,
    /// Delete a provider.
    DeleteProvider,
    /// List recipients.
    ListRecipients,
    /// Create a new recipient.
    CreateRecipient,
    /// Get a recipient by name.
    GetRecipient,
    /// Update a recipient.
    UpdateRecipient,
    /// Delete a recipient.
    DeleteRecipient,
    /// Rotate a recipient's token: issue a new token (and activation link) and
    /// retire the existing one.
    RotateRecipientToken,
    /// List registered models
    ///
    /// List registered models within the specified parent catalog and schema. If
    /// the caller is the metastore admin, all registered models are returned in the
    /// response. Otherwise, the caller must have USE_CATALOG on the parent catalog
    /// and USE_SCHEMA on the parent schema, and the model must either be owned by
    /// the caller or the caller must have a privilege on the model.
    ListRegisteredModels,
    /// Create a registered model
    ///
    /// Creates a new registered model. The caller must be a metastore admin or have
    /// the CREATE_MODEL privilege on the parent catalog and schema.
    CreateRegisteredModel,
    /// Get a registered model
    ///
    /// Gets a registered model from within a parent catalog and schema. For the
    /// fetch to succeed, the caller must be a metastore admin, the owner of the
    /// registered model, or have a privilege on the registered model.
    GetRegisteredModel,
    /// Update a registered model
    ///
    /// Updates the registered model that matches the supplied name.
    UpdateRegisteredModel,
    /// Delete a registered model
    ///
    /// Deletes the registered model that matches the supplied name. For the deletion
    /// to succeed, the caller must be the owner of the registered model.
    DeleteRegisteredModel,
    /// Gets an array of schemas for a catalog in the metastore. If the caller is the metastore
    /// admin or the owner of the parent catalog, all schemas for the catalog will be retrieved.
    /// Otherwise, only schemas owned by the caller (or for which the caller has the USE_SCHEMA privilege)
    /// will be retrieved. There is no guarantee of a specific ordering of the elements in the array.
    ListSchemas,
    /// Creates a new schema for catalog in the Metatastore. The caller must be a metastore admin,
    /// or have the CREATE_SCHEMA privilege in the parent catalog.
    CreateSchema,
    /// Gets the specified schema within the metastore.
    /// The caller must be a metastore admin, the owner of the schema,
    /// or a user that has the USE_SCHEMA privilege on the schema.
    GetSchema,
    /// Updates a schema for a catalog. The caller must be the owner of the schema or a metastore admin.
    /// If the caller is a metastore admin, only the owner field can be changed in the update.
    /// If the name field must be updated, the caller must be a metastore admin or have the CREATE_SCHEMA
    /// privilege on the parent catalog.
    UpdateSchema,
    /// Deletes the specified schema from the parent catalog. The caller must be the owner
    /// of the schema or an owner of the parent catalog.
    DeleteSchema,
    /// List shares.
    ListShares,
    /// Create a new share.
    CreateShare,
    /// Get a share by name.
    GetShare,
    /// Update a share.
    UpdateShare,
    /// Deletes a share.
    DeleteShare,
    /// Gets the permissions for a data share from the metastore.
    GetPermissions,
    /// Updates the permissions for a data share in the metastore.
    UpdatePermissions,
    /// Creates a new staging table, allocating an immutable table id and a storage
    /// location under the parent schema/catalog managed storage root. The caller
    /// must have the CREATE privilege on the parent schema.
    CreateStagingTable,
    /// Gets an array of summaries for tables for a schema and catalog within the metastore. The table summaries returned are either:
    /// - summaries for tables (within the current metastore and parent catalog and schema), when the user is a metastore admin, or:
    /// - summaries for tables and schemas (within the current metastore and parent catalog) for which the user has ownership or the
    /// SELECT privilege on the table and ownership or USE_SCHEMA privilege on the schema, provided that the user also has ownership
    /// or the USE_CATALOG privilege on the parent catalog.
    ///
    /// There is no guarantee of a specific ordering of the elements in the array.
    ListTableSummaries,
    /// Gets an array of all tables for the current metastore under the parent catalog and schema.
    ///
    /// The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table.
    /// For the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the
    /// parent catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a
    /// specific ordering of the elements in the array.
    ListTables,
    /// Create a table
    CreateTable,
    /// Get a table
    GetTable,
    /// Get boolean reflecting if table exists
    GetTableExists,
    /// Delete a table
    DeleteTable,
    /// List tag policies
    ///
    /// Gets an array of tag policies. There is no guarantee of a specific ordering
    /// of the elements in the array.
    ListTagPolicies,
    /// Create a new tag policy
    ///
    /// Creates a new governed tag definition.
    CreateTagPolicy,
    /// Get a tag policy
    ///
    /// Gets the governed tag definition for the specified tag key.
    GetTagPolicy,
    /// Update a tag policy
    ///
    /// Updates the governed tag definition that matches the supplied tag key.
    UpdateTagPolicy,
    /// Delete a tag policy
    ///
    /// Deletes the governed tag definition that matches the supplied tag key.
    DeleteTagPolicy,
    /// Generate a new set of credentials for a table.
    ///
    /// The metastore must have the `external_access_enabled` flag set to true
    /// (default false). The caller must have the `EXTERNAL_USE_SCHEMA`
    /// privilege on the parent schema (granted by a catalog owner).
    GenerateTemporaryTableCredentials,
    /// Generate a new set of credentials for a path.
    ///
    /// Only external storage paths (external locations and external tables) are
    /// supported; managed tables are not. The metastore must have the
    /// `external_access_enabled` flag set to true (default false). The caller must
    /// have the `EXTERNAL_USE_LOCATION` privilege on the external location
    /// (granted by its owner) and, for existing external tables, the
    /// `EXTERNAL_USE_SCHEMA` privilege on the parent schema.
    GenerateTemporaryPathCredentials,
    /// Generate a new set of credentials for a volume.
    ///
    /// The metastore must have the `external_access_enabled` flag set to true
    /// (default false). The caller must have the `EXTERNAL_USE_SCHEMA`
    /// privilege on the parent schema (granted by a catalog owner).
    GenerateTemporaryVolumeCredentials,
    /// Generate a new set of credentials for a model version.
    ///
    /// The metastore must have the `external_access_enabled` flag set to true
    /// (default false). The caller must have the `EXTERNAL_USE_SCHEMA`
    /// privilege on the parent schema (granted by a catalog owner).
    GenerateTemporaryModelVersionCredentials,
    /// Lists volumes.
    ListVolumes,
    CreateVolume,
    GetVolume,
    UpdateVolume,
    DeleteVolume,
}

impl Operation {
    /// Every operation, in declaration order.
    pub const ALL: &'static [Operation] = &[
        Operation::ListAgentSkills,
        Operation::CreateAgentSkill,
        Operation::GetAgentSkill,
        Operation::UpdateAgentSkill,
        Operation::DeleteAgentSkill,
        Operation::ListAgents,
        Operation::CreateAgent,
        Operation::GetAgent,
        Operation::UpdateAgent,
        Operation::DeleteAgent,
        Operation::ListCatalogs,
        Operation::CreateCatalog,
        Operation::GetCatalog,
        Operation::UpdateCatalog,
        Operation::DeleteCatalog,
        Operation::ListCredentials,
        Operation::CreateCredential,
        Operation::GetCredential,
        Operation::UpdateCredential,
        Operation::DeleteCredential,
        Operation::ListEntityTagAssignments,
        Operation::CreateEntityTagAssignment,
        Operation::GetEntityTagAssignment,
        Operation::UpdateEntityTagAssignment,
        Operation::DeleteEntityTagAssignment,
        Operation::ListExternalLocations,
        Operation::CreateExternalLocation,
        Operation::GetExternalLocation,
        Operation::UpdateExternalLocation,
        Operation::DeleteExternalLocation,
        Operation::ListFunctions,
        Operation::CreateFunction,
        Operation::GetFunction,
        Operation::UpdateFunction,
        Operation::DeleteFunction,
        Operation::GetGrants,
        Operation::UpdateGrants,
        Operation::ListModelVersions,
        Operation::CreateModelVersion,
        Operation::GetModelVersion,
        Operation::UpdateModelVersion,
        Operation::DeleteModelVersion,
        Operation::FinalizeModelVersion,
        Operation::ListPolicies,
        Operation::CreatePolicy,
        Operation::GetPolicy,
        Operation::UpdatePolicy,
        Operation::DeletePolicy,
        Operation::ListProviders,
        Operation::CreateProvider,
        Operation::GetProvider,
        Operation::UpdateProvider,
        Operation::DeleteProvider,
        Operation::ListRecipients,
        Operation::CreateRecipient,
        Operation::GetRecipient,
        Operation::UpdateRecipient,
        Operation::DeleteRecipient,
        Operation::RotateRecipientToken,
        Operation::ListRegisteredModels,
        Operation::CreateRegisteredModel,
        Operation::GetRegisteredModel,
        Operation::UpdateRegisteredModel,
        Operation::DeleteRegisteredModel,
        Operation::ListSchemas,
        Operation::CreateSchema,
        Operation::GetSchema,
        Operation::UpdateSchema,
        Operation::DeleteSchema,
        Operation::ListShares,
        Operation::CreateShare,
        Operation::GetShare,
        Operation::UpdateShare,
        Operation::DeleteShare,
        Operation::GetPermissions,
        Operation::UpdatePermissions,
        Operation::CreateStagingTable,
        Operation::ListTableSummaries,
        Operation::ListTables,
        Operation::CreateTable,
        Operation::GetTable,
        Operation::GetTableExists,
        Operation::DeleteTable,
        Operation::ListTagPolicies,
        Operation::CreateTagPolicy,
        Operation::GetTagPolicy,
        Operation::UpdateTagPolicy,
        Operation::DeleteTagPolicy,
        Operation::GenerateTemporaryTableCredentials,
        Operation::GenerateTemporaryPathCredentials,
        Operation::GenerateTemporaryVolumeCredentials,
        Operation::GenerateTemporaryModelVersionCredentials,
        Operation::ListVolumes,
        Operation::CreateVolume,
        Operation::GetVolume,
        Operation::UpdateVolume,
        Operation::DeleteVolume,
    ];
    /// The name of the operation: the RPC name, which is also its Cedar action id.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Operation::ListAgentSkills => "ListAgentSkills",
            Operation::CreateAgentSkill => "CreateAgentSkill",
            Operation::GetAgentSkill => "GetAgentSkill",
            Operation::UpdateAgentSkill => "UpdateAgentSkill",
            Operation::DeleteAgentSkill => "DeleteAgentSkill",
            Operation::ListAgents => "ListAgents",
            Operation::CreateAgent => "CreateAgent",
            Operation::GetAgent => "GetAgent",
            Operation::UpdateAgent => "UpdateAgent",
            Operation::DeleteAgent => "DeleteAgent",
            Operation::ListCatalogs => "ListCatalogs",
            Operation::CreateCatalog => "CreateCatalog",
            Operation::GetCatalog => "GetCatalog",
            Operation::UpdateCatalog => "UpdateCatalog",
            Operation::DeleteCatalog => "DeleteCatalog",
            Operation::ListCredentials => "ListCredentials",
            Operation::CreateCredential => "CreateCredential",
            Operation::GetCredential => "GetCredential",
            Operation::UpdateCredential => "UpdateCredential",
            Operation::DeleteCredential => "DeleteCredential",
            Operation::ListEntityTagAssignments => "ListEntityTagAssignments",
            Operation::CreateEntityTagAssignment => "CreateEntityTagAssignment",
            Operation::GetEntityTagAssignment => "GetEntityTagAssignment",
            Operation::UpdateEntityTagAssignment => "UpdateEntityTagAssignment",
            Operation::DeleteEntityTagAssignment => "DeleteEntityTagAssignment",
            Operation::ListExternalLocations => "ListExternalLocations",
            Operation::CreateExternalLocation => "CreateExternalLocation",
            Operation::GetExternalLocation => "GetExternalLocation",
            Operation::UpdateExternalLocation => "UpdateExternalLocation",
            Operation::DeleteExternalLocation => "DeleteExternalLocation",
            Operation::ListFunctions => "ListFunctions",
            Operation::CreateFunction => "CreateFunction",
            Operation::GetFunction => "GetFunction",
            Operation::UpdateFunction => "UpdateFunction",
            Operation::DeleteFunction => "DeleteFunction",
            Operation::GetGrants => "GetGrants",
            Operation::UpdateGrants => "UpdateGrants",
            Operation::ListModelVersions => "ListModelVersions",
            Operation::CreateModelVersion => "CreateModelVersion",
            Operation::GetModelVersion => "GetModelVersion",
            Operation::UpdateModelVersion => "UpdateModelVersion",
            Operation::DeleteModelVersion => "DeleteModelVersion",
            Operation::FinalizeModelVersion => "FinalizeModelVersion",
            Operation::ListPolicies => "ListPolicies",
            Operation::CreatePolicy => "CreatePolicy",
            Operation::GetPolicy => "GetPolicy",
            Operation::UpdatePolicy => "UpdatePolicy",
            Operation::DeletePolicy => "DeletePolicy",
            Operation::ListProviders => "ListProviders",
            Operation::CreateProvider => "CreateProvider",
            Operation::GetProvider => "GetProvider",
            Operation::UpdateProvider => "UpdateProvider",
            Operation::DeleteProvider => "DeleteProvider",
            Operation::ListRecipients => "ListRecipients",
            Operation::CreateRecipient => "CreateRecipient",
            Operation::GetRecipient => "GetRecipient",
            Operation::UpdateRecipient => "UpdateRecipient",
            Operation::DeleteRecipient => "DeleteRecipient",
            Operation::RotateRecipientToken => "RotateRecipientToken",
            Operation::ListRegisteredModels => "ListRegisteredModels",
            Operation::CreateRegisteredModel => "CreateRegisteredModel",
            Operation::GetRegisteredModel => "GetRegisteredModel",
            Operation::UpdateRegisteredModel => "UpdateRegisteredModel",
            Operation::DeleteRegisteredModel => "DeleteRegisteredModel",
            Operation::ListSchemas => "ListSchemas",
            Operation::CreateSchema => "CreateSchema",
            Operation::GetSchema => "GetSchema",
            Operation::UpdateSchema => "UpdateSchema",
            Operation::DeleteSchema => "DeleteSchema",
            Operation::ListShares => "ListShares",
            Operation::CreateShare => "CreateShare",
            Operation::GetShare => "GetShare",
            Operation::UpdateShare => "UpdateShare",
            Operation::DeleteShare => "DeleteShare",
            Operation::GetPermissions => "GetPermissions",
            Operation::UpdatePermissions => "UpdatePermissions",
            Operation::CreateStagingTable => "CreateStagingTable",
            Operation::ListTableSummaries => "ListTableSummaries",
            Operation::ListTables => "ListTables",
            Operation::CreateTable => "CreateTable",
            Operation::GetTable => "GetTable",
            Operation::GetTableExists => "GetTableExists",
            Operation::DeleteTable => "DeleteTable",
            Operation::ListTagPolicies => "ListTagPolicies",
            Operation::CreateTagPolicy => "CreateTagPolicy",
            Operation::GetTagPolicy => "GetTagPolicy",
            Operation::UpdateTagPolicy => "UpdateTagPolicy",
            Operation::DeleteTagPolicy => "DeleteTagPolicy",
            Operation::GenerateTemporaryTableCredentials => "GenerateTemporaryTableCredentials",
            Operation::GenerateTemporaryPathCredentials => "GenerateTemporaryPathCredentials",
            Operation::GenerateTemporaryVolumeCredentials => "GenerateTemporaryVolumeCredentials",
            Operation::GenerateTemporaryModelVersionCredentials => {
                "GenerateTemporaryModelVersionCredentials"
            }
            Operation::ListVolumes => "ListVolumes",
            Operation::CreateVolume => "CreateVolume",
            Operation::GetVolume => "GetVolume",
            Operation::UpdateVolume => "UpdateVolume",
            Operation::DeleteVolume => "DeleteVolume",
        }
    }

    /// The kind of securable the operation is asked on.
    pub const fn securable_kind(&self) -> SecurableKind {
        match self {
            Operation::ListAgentSkills => SecurableKind::Resource(ObjectLabel::Schema),
            Operation::CreateAgentSkill => SecurableKind::Resource(ObjectLabel::AgentSkill),
            Operation::GetAgentSkill => SecurableKind::Resource(ObjectLabel::AgentSkill),
            Operation::UpdateAgentSkill => SecurableKind::Resource(ObjectLabel::AgentSkill),
            Operation::DeleteAgentSkill => SecurableKind::Resource(ObjectLabel::AgentSkill),
            Operation::ListAgents => SecurableKind::Resource(ObjectLabel::Schema),
            Operation::CreateAgent => SecurableKind::Resource(ObjectLabel::Agent),
            Operation::GetAgent => SecurableKind::Resource(ObjectLabel::Agent),
            Operation::UpdateAgent => SecurableKind::Resource(ObjectLabel::Agent),
            Operation::DeleteAgent => SecurableKind::Resource(ObjectLabel::Agent),
            Operation::ListCatalogs => SecurableKind::Root,
            Operation::CreateCatalog => SecurableKind::Resource(ObjectLabel::Catalog),
            Operation::GetCatalog => SecurableKind::Resource(ObjectLabel::Catalog),
            Operation::UpdateCatalog => SecurableKind::Resource(ObjectLabel::Catalog),
            Operation::DeleteCatalog => SecurableKind::Resource(ObjectLabel::Catalog),
            Operation::ListCredentials => SecurableKind::Root,
            Operation::CreateCredential => SecurableKind::Resource(ObjectLabel::Credential),
            Operation::GetCredential => SecurableKind::Resource(ObjectLabel::Credential),
            Operation::UpdateCredential => SecurableKind::Resource(ObjectLabel::Credential),
            Operation::DeleteCredential => SecurableKind::Resource(ObjectLabel::Credential),
            Operation::ListEntityTagAssignments => SecurableKind::Any,
            Operation::CreateEntityTagAssignment => SecurableKind::Any,
            Operation::GetEntityTagAssignment => SecurableKind::Any,
            Operation::UpdateEntityTagAssignment => SecurableKind::Any,
            Operation::DeleteEntityTagAssignment => SecurableKind::Any,
            Operation::ListExternalLocations => SecurableKind::Root,
            Operation::CreateExternalLocation => {
                SecurableKind::Resource(ObjectLabel::ExternalLocation)
            }
            Operation::GetExternalLocation => {
                SecurableKind::Resource(ObjectLabel::ExternalLocation)
            }
            Operation::UpdateExternalLocation => {
                SecurableKind::Resource(ObjectLabel::ExternalLocation)
            }
            Operation::DeleteExternalLocation => {
                SecurableKind::Resource(ObjectLabel::ExternalLocation)
            }
            Operation::ListFunctions => SecurableKind::Resource(ObjectLabel::Schema),
            Operation::CreateFunction => SecurableKind::Resource(ObjectLabel::Function),
            Operation::GetFunction => SecurableKind::Resource(ObjectLabel::Function),
            Operation::UpdateFunction => SecurableKind::Resource(ObjectLabel::Function),
            Operation::DeleteFunction => SecurableKind::Resource(ObjectLabel::Function),
            Operation::GetGrants => SecurableKind::Any,
            Operation::UpdateGrants => SecurableKind::Any,
            Operation::ListModelVersions => SecurableKind::Resource(ObjectLabel::RegisteredModel),
            Operation::CreateModelVersion => SecurableKind::Resource(ObjectLabel::RegisteredModel),
            Operation::GetModelVersion => SecurableKind::Resource(ObjectLabel::ModelVersion),
            Operation::UpdateModelVersion => SecurableKind::Resource(ObjectLabel::ModelVersion),
            Operation::DeleteModelVersion => SecurableKind::Resource(ObjectLabel::ModelVersion),
            Operation::FinalizeModelVersion => SecurableKind::Resource(ObjectLabel::ModelVersion),
            Operation::ListPolicies => SecurableKind::Any,
            Operation::CreatePolicy => SecurableKind::Any,
            Operation::GetPolicy => SecurableKind::Any,
            Operation::UpdatePolicy => SecurableKind::Any,
            Operation::DeletePolicy => SecurableKind::Any,
            Operation::ListProviders => SecurableKind::Root,
            Operation::CreateProvider => SecurableKind::Resource(ObjectLabel::Provider),
            Operation::GetProvider => SecurableKind::Resource(ObjectLabel::Provider),
            Operation::UpdateProvider => SecurableKind::Resource(ObjectLabel::Provider),
            Operation::DeleteProvider => SecurableKind::Resource(ObjectLabel::Provider),
            Operation::ListRecipients => SecurableKind::Root,
            Operation::CreateRecipient => SecurableKind::Resource(ObjectLabel::Recipient),
            Operation::GetRecipient => SecurableKind::Resource(ObjectLabel::Recipient),
            Operation::UpdateRecipient => SecurableKind::Resource(ObjectLabel::Recipient),
            Operation::DeleteRecipient => SecurableKind::Resource(ObjectLabel::Recipient),
            Operation::RotateRecipientToken => SecurableKind::Resource(ObjectLabel::Recipient),
            Operation::ListRegisteredModels => SecurableKind::Resource(ObjectLabel::Schema),
            Operation::CreateRegisteredModel => {
                SecurableKind::Resource(ObjectLabel::RegisteredModel)
            }
            Operation::GetRegisteredModel => SecurableKind::Resource(ObjectLabel::RegisteredModel),
            Operation::UpdateRegisteredModel => {
                SecurableKind::Resource(ObjectLabel::RegisteredModel)
            }
            Operation::DeleteRegisteredModel => {
                SecurableKind::Resource(ObjectLabel::RegisteredModel)
            }
            Operation::ListSchemas => SecurableKind::Resource(ObjectLabel::Catalog),
            Operation::CreateSchema => SecurableKind::Resource(ObjectLabel::Schema),
            Operation::GetSchema => SecurableKind::Resource(ObjectLabel::Schema),
            Operation::UpdateSchema => SecurableKind::Resource(ObjectLabel::Schema),
            Operation::DeleteSchema => SecurableKind::Resource(ObjectLabel::Schema),
            Operation::ListShares => SecurableKind::Root,
            Operation::CreateShare => SecurableKind::Resource(ObjectLabel::Share),
            Operation::GetShare => SecurableKind::Resource(ObjectLabel::Share),
            Operation::UpdateShare => SecurableKind::Resource(ObjectLabel::Share),
            Operation::DeleteShare => SecurableKind::Resource(ObjectLabel::Share),
            Operation::GetPermissions => SecurableKind::Resource(ObjectLabel::Share),
            Operation::UpdatePermissions => SecurableKind::Resource(ObjectLabel::Share),
            Operation::CreateStagingTable => SecurableKind::Resource(ObjectLabel::StagingTable),
            Operation::ListTableSummaries => SecurableKind::Resource(ObjectLabel::Catalog),
            Operation::ListTables => SecurableKind::Resource(ObjectLabel::Schema),
            Operation::CreateTable => SecurableKind::Resource(ObjectLabel::Table),
            Operation::GetTable => SecurableKind::Resource(ObjectLabel::Table),
            Operation::GetTableExists => SecurableKind::Resource(ObjectLabel::Table),
            Operation::DeleteTable => SecurableKind::Resource(ObjectLabel::Table),
            Operation::ListTagPolicies => SecurableKind::Root,
            Operation::CreateTagPolicy => SecurableKind::Resource(ObjectLabel::TagPolicy),
            Operation::GetTagPolicy => SecurableKind::Resource(ObjectLabel::TagPolicy),
            Operation::UpdateTagPolicy => SecurableKind::Resource(ObjectLabel::TagPolicy),
            Operation::DeleteTagPolicy => SecurableKind::Resource(ObjectLabel::TagPolicy),
            Operation::GenerateTemporaryTableCredentials => {
                SecurableKind::Resource(ObjectLabel::Table)
            }
            Operation::GenerateTemporaryPathCredentials => {
                SecurableKind::Resource(ObjectLabel::ExternalLocation)
            }
            Operation::GenerateTemporaryVolumeCredentials => {
                SecurableKind::Resource(ObjectLabel::Volume)
            }
            Operation::GenerateTemporaryModelVersionCredentials => {
                SecurableKind::Resource(ObjectLabel::ModelVersion)
            }
            Operation::ListVolumes => SecurableKind::Resource(ObjectLabel::Schema),
            Operation::CreateVolume => SecurableKind::Resource(ObjectLabel::Volume),
            Operation::GetVolume => SecurableKind::Resource(ObjectLabel::Volume),
            Operation::UpdateVolume => SecurableKind::Resource(ObjectLabel::Volume),
            Operation::DeleteVolume => SecurableKind::Resource(ObjectLabel::Volume),
        }
    }

    /// The operation that reads one resource of `label` (the `Get` RPC of its service),
    /// if there is one. List results can be filtered with it.
    #[allow(unreachable_patterns)]
    pub const fn get_of(label: ObjectLabel) -> Option<Operation> {
        match label {
            ObjectLabel::Agent => Some(Operation::GetAgent),
            ObjectLabel::AgentSkill => Some(Operation::GetAgentSkill),
            ObjectLabel::Catalog => Some(Operation::GetCatalog),
            ObjectLabel::Credential => Some(Operation::GetCredential),
            ObjectLabel::ExternalLocation => Some(Operation::GetExternalLocation),
            ObjectLabel::Function => Some(Operation::GetFunction),
            ObjectLabel::ModelVersion => Some(Operation::GetModelVersion),
            ObjectLabel::PolicyInfo => Some(Operation::GetPolicy),
            ObjectLabel::Provider => Some(Operation::GetProvider),
            ObjectLabel::Recipient => Some(Operation::GetRecipient),
            ObjectLabel::RegisteredModel => Some(Operation::GetRegisteredModel),
            ObjectLabel::Schema => Some(Operation::GetSchema),
            ObjectLabel::Share => Some(Operation::GetShare),
            ObjectLabel::Table => Some(Operation::GetTable),
            ObjectLabel::TagPolicy => Some(Operation::GetTagPolicy),
            ObjectLabel::Volume => Some(Operation::GetVolume),
            _ => None,
        }
    }
}

impl ::std::fmt::Display for Operation {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ::std::str::FromStr for Operation {
    type Err = UnknownOperation;
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "ListAgentSkills" => Ok(Operation::ListAgentSkills),
            "CreateAgentSkill" => Ok(Operation::CreateAgentSkill),
            "GetAgentSkill" => Ok(Operation::GetAgentSkill),
            "UpdateAgentSkill" => Ok(Operation::UpdateAgentSkill),
            "DeleteAgentSkill" => Ok(Operation::DeleteAgentSkill),
            "ListAgents" => Ok(Operation::ListAgents),
            "CreateAgent" => Ok(Operation::CreateAgent),
            "GetAgent" => Ok(Operation::GetAgent),
            "UpdateAgent" => Ok(Operation::UpdateAgent),
            "DeleteAgent" => Ok(Operation::DeleteAgent),
            "ListCatalogs" => Ok(Operation::ListCatalogs),
            "CreateCatalog" => Ok(Operation::CreateCatalog),
            "GetCatalog" => Ok(Operation::GetCatalog),
            "UpdateCatalog" => Ok(Operation::UpdateCatalog),
            "DeleteCatalog" => Ok(Operation::DeleteCatalog),
            "ListCredentials" => Ok(Operation::ListCredentials),
            "CreateCredential" => Ok(Operation::CreateCredential),
            "GetCredential" => Ok(Operation::GetCredential),
            "UpdateCredential" => Ok(Operation::UpdateCredential),
            "DeleteCredential" => Ok(Operation::DeleteCredential),
            "ListEntityTagAssignments" => Ok(Operation::ListEntityTagAssignments),
            "CreateEntityTagAssignment" => Ok(Operation::CreateEntityTagAssignment),
            "GetEntityTagAssignment" => Ok(Operation::GetEntityTagAssignment),
            "UpdateEntityTagAssignment" => Ok(Operation::UpdateEntityTagAssignment),
            "DeleteEntityTagAssignment" => Ok(Operation::DeleteEntityTagAssignment),
            "ListExternalLocations" => Ok(Operation::ListExternalLocations),
            "CreateExternalLocation" => Ok(Operation::CreateExternalLocation),
            "GetExternalLocation" => Ok(Operation::GetExternalLocation),
            "UpdateExternalLocation" => Ok(Operation::UpdateExternalLocation),
            "DeleteExternalLocation" => Ok(Operation::DeleteExternalLocation),
            "ListFunctions" => Ok(Operation::ListFunctions),
            "CreateFunction" => Ok(Operation::CreateFunction),
            "GetFunction" => Ok(Operation::GetFunction),
            "UpdateFunction" => Ok(Operation::UpdateFunction),
            "DeleteFunction" => Ok(Operation::DeleteFunction),
            "GetGrants" => Ok(Operation::GetGrants),
            "UpdateGrants" => Ok(Operation::UpdateGrants),
            "ListModelVersions" => Ok(Operation::ListModelVersions),
            "CreateModelVersion" => Ok(Operation::CreateModelVersion),
            "GetModelVersion" => Ok(Operation::GetModelVersion),
            "UpdateModelVersion" => Ok(Operation::UpdateModelVersion),
            "DeleteModelVersion" => Ok(Operation::DeleteModelVersion),
            "FinalizeModelVersion" => Ok(Operation::FinalizeModelVersion),
            "ListPolicies" => Ok(Operation::ListPolicies),
            "CreatePolicy" => Ok(Operation::CreatePolicy),
            "GetPolicy" => Ok(Operation::GetPolicy),
            "UpdatePolicy" => Ok(Operation::UpdatePolicy),
            "DeletePolicy" => Ok(Operation::DeletePolicy),
            "ListProviders" => Ok(Operation::ListProviders),
            "CreateProvider" => Ok(Operation::CreateProvider),
            "GetProvider" => Ok(Operation::GetProvider),
            "UpdateProvider" => Ok(Operation::UpdateProvider),
            "DeleteProvider" => Ok(Operation::DeleteProvider),
            "ListRecipients" => Ok(Operation::ListRecipients),
            "CreateRecipient" => Ok(Operation::CreateRecipient),
            "GetRecipient" => Ok(Operation::GetRecipient),
            "UpdateRecipient" => Ok(Operation::UpdateRecipient),
            "DeleteRecipient" => Ok(Operation::DeleteRecipient),
            "RotateRecipientToken" => Ok(Operation::RotateRecipientToken),
            "ListRegisteredModels" => Ok(Operation::ListRegisteredModels),
            "CreateRegisteredModel" => Ok(Operation::CreateRegisteredModel),
            "GetRegisteredModel" => Ok(Operation::GetRegisteredModel),
            "UpdateRegisteredModel" => Ok(Operation::UpdateRegisteredModel),
            "DeleteRegisteredModel" => Ok(Operation::DeleteRegisteredModel),
            "ListSchemas" => Ok(Operation::ListSchemas),
            "CreateSchema" => Ok(Operation::CreateSchema),
            "GetSchema" => Ok(Operation::GetSchema),
            "UpdateSchema" => Ok(Operation::UpdateSchema),
            "DeleteSchema" => Ok(Operation::DeleteSchema),
            "ListShares" => Ok(Operation::ListShares),
            "CreateShare" => Ok(Operation::CreateShare),
            "GetShare" => Ok(Operation::GetShare),
            "UpdateShare" => Ok(Operation::UpdateShare),
            "DeleteShare" => Ok(Operation::DeleteShare),
            "GetPermissions" => Ok(Operation::GetPermissions),
            "UpdatePermissions" => Ok(Operation::UpdatePermissions),
            "CreateStagingTable" => Ok(Operation::CreateStagingTable),
            "ListTableSummaries" => Ok(Operation::ListTableSummaries),
            "ListTables" => Ok(Operation::ListTables),
            "CreateTable" => Ok(Operation::CreateTable),
            "GetTable" => Ok(Operation::GetTable),
            "GetTableExists" => Ok(Operation::GetTableExists),
            "DeleteTable" => Ok(Operation::DeleteTable),
            "ListTagPolicies" => Ok(Operation::ListTagPolicies),
            "CreateTagPolicy" => Ok(Operation::CreateTagPolicy),
            "GetTagPolicy" => Ok(Operation::GetTagPolicy),
            "UpdateTagPolicy" => Ok(Operation::UpdateTagPolicy),
            "DeleteTagPolicy" => Ok(Operation::DeleteTagPolicy),
            "GenerateTemporaryTableCredentials" => Ok(Operation::GenerateTemporaryTableCredentials),
            "GenerateTemporaryPathCredentials" => Ok(Operation::GenerateTemporaryPathCredentials),
            "GenerateTemporaryVolumeCredentials" => {
                Ok(Operation::GenerateTemporaryVolumeCredentials)
            }
            "GenerateTemporaryModelVersionCredentials" => {
                Ok(Operation::GenerateTemporaryModelVersionCredentials)
            }
            "ListVolumes" => Ok(Operation::ListVolumes),
            "CreateVolume" => Ok(Operation::CreateVolume),
            "GetVolume" => Ok(Operation::GetVolume),
            "UpdateVolume" => Ok(Operation::UpdateVolume),
            "DeleteVolume" => Ok(Operation::DeleteVolume),
            other => Err(UnknownOperation(other.to_string())),
        }
    }
}

/// The kind of resource an [`Operation`] acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurableKind {
    /// The service root, e.g. for a list of top-level resources.
    Root,
    /// A resource of this kind.
    Resource(ObjectLabel),
    /// Any resource, as the request decides at run time (e.g. a request naming the type
    /// of the resource it acts on).
    Any,
}

/// The error of parsing a name that is not an [`Operation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownOperation(pub String);
impl ::std::fmt::Display for UnknownOperation {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "unknown operation `{}`", self.0)
    }
}

impl ::std::error::Error for UnknownOperation {}
/// The resource a request acts on.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Securable {
    /// The kind of resource, or `None` for the service root.
    pub label: Option<ObjectLabel>,
    /// The parts of the resource's name, outermost first (e.g. a parent's name, then
    /// the resource's own). Empty for the root. A list names the container of the
    /// resources it lists; a create names the new resource.
    ///
    /// A request that leaves a name field empty gives an empty part, which names no
    /// resource: deny it rather than reading the shorter name as a container.
    pub name: Vec<String>,
}

impl Securable {
    /// The resource of kind `label` named `name`.
    pub fn new(label: ObjectLabel, name: Vec<String>) -> Self {
        Self {
            label: Some(label),
            name,
        }
    }

    /// The service root.
    pub fn root() -> Self {
        Self {
            label: None,
            name: Vec::new(),
        }
    }
}

/// A request that names its operation and the securable it acts on.
pub trait Authorizable: Send + Sync {
    /// The operation the request performs.
    fn operation(&self) -> Operation;
    /// The resource the request acts on.
    fn securable(&self) -> Securable;
}
