use std::sync::Arc;

use delta_kernel::schema::{DataType, PrimitiveType, Schema, StructField};
use delta_kernel::{Snapshot, Version};
use itertools::Itertools;

use unitycatalog_common::ResourceIdent;
use unitycatalog_common::metric_view::{MetricView, dependencies as metric_view_dependencies};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::ResourceName;
use unitycatalog_common::models::tables::v1::*;

use super::staging_tables::find_staging_table_by_location;
use super::{RequestContext, SecuredAction};
pub use crate::codegen::tables::TableHandler;
use crate::policy::{Permission, Policy, Principal, process_resources};
use crate::services::ProvidesLocalStoragePolicy;
use crate::services::location::StorageLocationUrl;
use crate::services::object_store::validate_external_storage_location;
use crate::store::ResourceStore;
use crate::{Error, Result};

const MAX_RESULTS_TABLES: usize = 50;

impl SecuredAction for CreateTableRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::table(ResourceName::new([
            self.catalog_name.as_str(),
            self.schema_name.as_str(),
            self.name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListTableSummariesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::table(ResourceName::new([self.catalog_name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for ListTablesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::table(ResourceName::new([
            self.catalog_name.as_str(),
            self.schema_name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetTableRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::table(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetTableExistsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::table(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for DeleteTableRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::table(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

#[async_trait::async_trait]
pub trait TableManager: Send + Sync + 'static {
    async fn read_snapshot(
        &self,
        location: &StorageLocationUrl,
        format: &DataSourceFormat,
        version: Option<Version>,
    ) -> Result<Arc<Snapshot>>;
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext> + TableManager + ProvidesLocalStoragePolicy>
    TableHandler<RequestContext> for T
{
    #[tracing::instrument(skip(self, context))]
    async fn list_table_summaries(
        &self,
        request: ListTableSummariesRequest,
        context: RequestContext,
    ) -> Result<ListTableSummariesResponse> {
        self.check_required(&request, &context).await?;
        // TODO: handle like operators for schema and table name
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Table,
                Some(&ResourceName::new([&request.catalog_name])),
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        let infos: Vec<Table> = resources.into_iter().map(|r| r.try_into()).try_collect()?;
        Ok(ListTableSummariesResponse {
            tables: infos.into_iter().map(|r| r.into()).collect(),
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: RequestContext,
    ) -> Result<ListTablesResponse> {
        // TODO: assert max_results is within bounds <= 50
        self.check_required(&request, &context).await?;
        // TODO: handle like operators for schema and table name
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Table,
                Some(&ResourceName::new([
                    &request.catalog_name,
                    &request.schema_name,
                ])),
                request
                    .max_results
                    .map(|v| usize::min(v as usize, MAX_RESULTS_TABLES)),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListTablesResponse {
            tables: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_table(
        &self,
        request: CreateTableRequest,
        context: RequestContext,
    ) -> Result<Table> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let info = if request.table_type == TableType::External {
            let Some(location) = request.storage_location.as_ref() else {
                return Err(Error::invalid_argument("missing storage location"));
            };
            let location = StorageLocationUrl::parse(location)?;
            let data_source_format = request.data_source_format.as_known().unwrap_or_default();
            // Validate the storage location before touching cloud storage so we
            // fail fast on a governance violation. The path must live inside a
            // registered external location and must not overlap any existing
            // table or volume (Unity Catalog forbids overlapping governed
            // storage regions).
            validate_external_storage_location(self, &location).await?;
            let snapshot = self
                .read_snapshot(&location, &data_source_format, None)
                .await?;
            Table {
                name: request.name,
                catalog_name: request.catalog_name,
                schema_name: request.schema_name,
                table_type: request.table_type,
                data_source_format: request.data_source_format,
                properties: request.properties,
                storage_location: request.storage_location,
                comment: request.comment,
                columns: schema_to_columns(
                    snapshot.schema().as_ref(),
                    snapshot
                        .table_configuration()
                        .metadata()
                        .partition_columns(),
                )?,
                ..Default::default()
            }
        } else if request.table_type == TableType::Managed {
            // Managed table: finalize a previously created staging table. The
            // client has written the initial Delta commit (`0.json`) at the
            // staging location; here we commit the staging reservation, adopt its
            // id as the table id, and register the table. The server never writes
            // the Delta log itself.
            if request.data_source_format != DataSourceFormat::Delta {
                return Err(Error::invalid_argument(format!(
                    "managed tables must use the DELTA data source format, got {:?}",
                    request.data_source_format
                )));
            }
            let Some(location) = request.storage_location.as_ref() else {
                return Err(Error::invalid_argument(
                    "managed tables require storage_location to be the staging location",
                ));
            };

            // Resolve and validate the staging reservation.
            let staging = find_staging_table_by_location(self, location).await?;
            match context.recipient() {
                Principal::User(name) if staging.created_by.as_deref() == Some(name.as_str()) => {}
                Principal::Anonymous if staging.created_by.is_none() => {}
                _ => {
                    return Err(Error::NotAllowed);
                }
            }
            if staging.stage_committed {
                return Err(Error::invalid_argument(format!(
                    "staging table at '{location}' has already been committed"
                )));
            }

            // Confirm the client wrote a readable Delta table at the location and
            // adopt its schema as the source of truth.
            let location_url = StorageLocationUrl::parse(location)?;
            let snapshot = self
                .read_snapshot(&location_url, &DataSourceFormat::Delta, None)
                .await?;

            // Consume the staging reservation and register the table in one atomic
            // swap: the reservation is deleted and the table is created **at the same
            // uuid** (the id was fixed at staging time and the Delta API protocol
            // depends on it). The object store keys objects by a single uuid, so the
            // StagingTable and the Table cannot coexist at that id — the reservation
            // transitions into the table. Doing both in one transaction means a crash
            // between them cannot leave the reservation gone without the table, and a
            // concurrent reader sees either the staging row or the table row, never
            // neither. A subsequent commit against the same location then fails at
            // `find_staging_table_by_location` (gone), the correct already-committed
            // behavior.
            let staging_ident = ResourceIdent::staging_table(ResourceName::new([
                staging.catalog_name.as_str(),
                staging.schema_name.as_str(),
                staging.name.as_str(),
            ]));

            let table = Table {
                name: request.name,
                catalog_name: request.catalog_name,
                schema_name: request.schema_name,
                table_type: request.table_type,
                data_source_format: request.data_source_format,
                properties: request.properties,
                storage_location: request.storage_location,
                comment: request.comment,
                table_id: Some(staging.id),
                columns: schema_to_columns(
                    snapshot.schema().as_ref(),
                    snapshot
                        .table_configuration()
                        .metadata()
                        .partition_columns(),
                )?,
                ..Default::default()
            };
            // TODO: update the table with the current actor as owner
            return Ok(self
                .replace_atomically(&staging_ident, table.into())
                .await?
                .0
                .try_into()?);
        } else if request.table_type == TableType::MetricView {
            // Metric view: a semantic layer with no storage of its own. The
            // definition (YAML) lives in `view_definition`; there is no Delta
            // snapshot to read and no columns to derive here.
            let Some(view_definition) = request.view_definition.as_ref() else {
                return Err(Error::invalid_argument(
                    "metric views require view_definition (the YAML definition)",
                ));
            };

            // The definition is the single source of truth: parse it and derive
            // the dependency list. A client-supplied `view_dependencies` is only
            // accepted if it matches what we derive (the definition wins).
            let view = MetricView::from_yaml(view_definition)
                .map_err(|e| Error::invalid_argument(format!("invalid metric-view YAML: {e}")))?;
            let view_dependencies = metric_view_dependencies(&view).map_err(|e| {
                Error::invalid_argument(format!("cannot derive metric-view dependencies: {e}"))
            })?;
            if let Some(supplied) = request.view_dependencies.as_option()
                && supplied != &view_dependencies
            {
                return Err(Error::invalid_argument(
                    "supplied view_dependencies diverges from the definition; \
                     omit it (the server derives dependencies from view_definition)",
                ));
            }

            Table {
                name: request.name,
                catalog_name: request.catalog_name,
                schema_name: request.schema_name,
                table_type: request.table_type,
                data_source_format: request.data_source_format,
                properties: request.properties,
                comment: request.comment,
                view_definition: Some(view_definition.clone()),
                view_dependencies: Some(view_dependencies).into(),
                ..Default::default()
            }
        } else {
            return Err(Error::invalid_argument(format!(
                "unsupported table type: {:?}",
                request.table_type
            )));
        };
        // TODO: update the table with the current actor as owner
        // TODO: create updated_* relations
        Ok(self.create(info.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_table(&self, request: GetTableRequest, context: RequestContext) -> Result<Table> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        // TODO: get columns etc ...
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_table_exists(
        &self,
        request: GetTableExistsRequest,
        context: RequestContext,
    ) -> Result<GetTableExistsResponse> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        match self.get(&request.resource()).await {
            Ok(_) => Ok(GetTableExistsResponse {
                table_exists: true,
                ..Default::default()
            }),
            Err(unitycatalog_common::Error::NotFound) => Ok(GetTableExistsResponse {
                table_exists: false,
                ..Default::default()
            }),
            Err(e) => Err(e.into()),
        }
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_table(
        &self,
        request: DeleteTableRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }
}

trait FieldExt {
    fn type_text(&self) -> String;
    fn type_json(&self) -> Result<String>;
    fn type_name(&self) -> ColumnTypeName;
    fn type_precision(&self) -> Option<i32>;
    fn type_scale(&self) -> Option<i32>;
}

impl FieldExt for StructField {
    fn type_text(&self) -> String {
        self.data_type().to_string()
    }

    fn type_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self.data_type())?)
    }

    fn type_name(&self) -> ColumnTypeName {
        match self.data_type() {
            DataType::Primitive(p) => match p {
                PrimitiveType::String => ColumnTypeName::String,
                PrimitiveType::Long => ColumnTypeName::Long,
                PrimitiveType::Integer => ColumnTypeName::Int,
                PrimitiveType::Short => ColumnTypeName::Short,
                PrimitiveType::Byte => ColumnTypeName::Byte,
                PrimitiveType::Double => ColumnTypeName::Double,
                PrimitiveType::Float => ColumnTypeName::Float,
                PrimitiveType::Boolean => ColumnTypeName::Boolean,
                PrimitiveType::Binary => ColumnTypeName::Binary,
                PrimitiveType::Date => ColumnTypeName::Date,
                PrimitiveType::Timestamp => ColumnTypeName::Timestamp,
                PrimitiveType::TimestampNtz => ColumnTypeName::TimestampNtz,
                PrimitiveType::Decimal(_) => ColumnTypeName::Decimal,
                PrimitiveType::Void => ColumnTypeName::Null,
                // kernel v0.25 split the interval primitive into year-month and
                // day-time; UC's column type taxonomy has a single `Interval`.
                PrimitiveType::IntervalYearMonth | PrimitiveType::IntervalDayTime => {
                    ColumnTypeName::Interval
                }
            },
            DataType::Struct(_) => ColumnTypeName::Struct,
            DataType::Array(_) => ColumnTypeName::Array,
            DataType::Map(_) => ColumnTypeName::Map,
            DataType::Variant(_) => ColumnTypeName::Variant,
        }
    }

    fn type_precision(&self) -> Option<i32> {
        match self.data_type() {
            DataType::Primitive(PrimitiveType::Decimal(dec)) => Some(dec.precision() as i32),
            _ => None,
        }
    }

    fn type_scale(&self) -> Option<i32> {
        match self.data_type() {
            DataType::Primitive(PrimitiveType::Decimal(dec)) => Some(dec.scale() as i32),
            _ => None,
        }
    }
}

fn schema_to_columns(schema: &Schema, partition_columns: &[String]) -> Result<Vec<Column>> {
    let partition_index = |name: &str| partition_columns.iter().position(|n| n == name);
    schema
        .fields()
        .enumerate()
        .map(|(idx, f)| {
            Ok(Column {
                name: f.name.clone(),
                nullable: Some(f.nullable),
                type_text: f.type_text(),
                type_json: f.type_json()?,
                type_name: f.type_name().into(),
                type_precision: f.type_precision(),
                type_scale: f.type_scale(),
                type_interval_type: None,
                position: Some(idx as i32),
                partition_index: partition_index(&f.name).map(|v| v as i32),
                ..Default::default()
            })
        })
        .try_collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::models::credentials::v1::{
        AwsIamRoleConfig, CreateCredentialRequest, Purpose,
    };
    use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::api::{CredentialHandler, ExternalLocationHandler};
    use crate::memory::InMemoryResourceStore;
    use crate::policy::ConstantPolicy;
    use crate::services::ServerHandler;

    fn handler() -> ServerHandler<RequestContext> {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        ServerHandler::try_new_tokio(policy, store).unwrap()
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: crate::policy::Principal::anonymous(),
        }
    }

    /// An external table whose storage location is not within any registered
    /// external location is rejected *before* any cloud access is attempted
    /// (the containment check runs ahead of `read_snapshot`).
    #[tokio::test]
    async fn external_table_outside_external_location_is_rejected() {
        let h = handler();
        h.create_credential(
            CreateCredentialRequest {
                name: "cred".to_string(),
                purpose: Purpose::Storage.into(),
                aws_iam_role: Some(AwsIamRoleConfig {
                    role_arn: "arn:aws:iam::123456789012:role/test".to_string(),
                    ..Default::default()
                })
                .into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_external_location(
            CreateExternalLocationRequest {
                name: "ext".to_string(),
                url: "s3://bucket/ext".to_string(),
                credential_name: "cred".to_string(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();

        let res = h
            .create_table(
                CreateTableRequest {
                    name: "t".to_string(),
                    schema_name: "sch".to_string(),
                    catalog_name: "cat".to_string(),
                    table_type: TableType::External.into(),
                    data_source_format: DataSourceFormat::Delta.into(),
                    storage_location: Some("s3://bucket/other/tbl".to_string()),
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    /// A managed table in a non-Delta format is rejected before any staging
    /// lookup or storage access.
    #[tokio::test]
    async fn managed_table_non_delta_is_rejected() {
        let h = handler();
        let res = h
            .create_table(
                CreateTableRequest {
                    name: "t".to_string(),
                    schema_name: "sch".to_string(),
                    catalog_name: "cat".to_string(),
                    table_type: TableType::Managed.into(),
                    data_source_format: DataSourceFormat::Parquet.into(),
                    storage_location: Some("s3://bucket/cat/__unitystorage/tables/x".to_string()),
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    /// A managed table whose storage_location has no matching staging table is
    /// rejected: the reservation must be created first via `/staging-tables`.
    #[tokio::test]
    async fn managed_table_without_staging_reservation_is_rejected() {
        let h = handler();
        let res = h
            .create_table(
                CreateTableRequest {
                    name: "t".to_string(),
                    schema_name: "sch".to_string(),
                    catalog_name: "cat".to_string(),
                    table_type: TableType::Managed.into(),
                    data_source_format: DataSourceFormat::Delta.into(),
                    storage_location: Some(
                        "s3://bucket/cat/__unitystorage/tables/unknown".to_string(),
                    ),
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::NotFound)), "{res:?}");
    }

    /// A managed table request without a storage_location (the staging location)
    /// is rejected.
    #[tokio::test]
    async fn managed_table_without_storage_location_is_rejected() {
        let h = handler();
        let res = h
            .create_table(
                CreateTableRequest {
                    name: "t".to_string(),
                    schema_name: "sch".to_string(),
                    catalog_name: "cat".to_string(),
                    table_type: TableType::Managed.into(),
                    data_source_format: DataSourceFormat::Delta.into(),
                    storage_location: None,
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    const METRIC_VIEW_YAML: &str = "version: \"1.1\"\nsource: cat.sch.orders\n\
                                    measures:\n  - name: revenue\n    expr: SUM(price)\n";

    /// A metric view is created with its YAML definition (no storage location,
    /// no Delta snapshot) and round-trips through `get_table` with the
    /// `view_definition` and `table_type` intact.
    #[tokio::test]
    async fn metric_view_create_get_round_trip() {
        let h = handler();
        let created = h
            .create_table(
                CreateTableRequest {
                    name: "orders_metrics".to_string(),
                    schema_name: "sch".to_string(),
                    catalog_name: "cat".to_string(),
                    table_type: TableType::MetricView.into(),
                    view_definition: Some(METRIC_VIEW_YAML.to_string()),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .expect("create metric view");
        assert_eq!(created.table_type, TableType::MetricView);
        assert_eq!(created.view_definition.as_deref(), Some(METRIC_VIEW_YAML));
        // Dependencies are derived from the definition's `source`.
        assert_eq!(
            dep_names(created.view_dependencies.as_option()),
            vec!["cat.sch.orders"]
        );

        let fetched = h
            .get_table(
                GetTableRequest {
                    full_name: "cat.sch.orders_metrics".to_string(),
                    ..Default::default()
                },
                ctx(),
            )
            .await
            .expect("get metric view");
        assert_eq!(fetched.table_type, TableType::MetricView);
        assert_eq!(fetched.view_definition.as_deref(), Some(METRIC_VIEW_YAML));
        // The derived dependencies round-trip through get.
        assert_eq!(
            dep_names(fetched.view_dependencies.as_option()),
            vec!["cat.sch.orders"]
        );
    }

    /// Extract the `table_full_name`s from a [`DependencyList`] for assertions.
    fn dep_names(deps: Option<&DependencyList>) -> Vec<String> {
        deps.map(|d| {
            d.dependencies
                .iter()
                .filter_map(|dep| match &dep.dependency {
                    Some(dependency::Dependency::Table(t)) => Some(t.table_full_name.clone()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
    }

    fn metric_view_request() -> CreateTableRequest {
        CreateTableRequest {
            name: "orders_metrics".to_string(),
            schema_name: "sch".to_string(),
            catalog_name: "cat".to_string(),
            table_type: TableType::MetricView.into(),
            view_definition: Some(METRIC_VIEW_YAML.to_string()),
            ..Default::default()
        }
    }

    fn table_dep(full_name: &str) -> Dependency {
        Dependency {
            dependency: Some(dependency::Dependency::Table(Box::new(TableDependency {
                table_full_name: full_name.to_string(),
                ..Default::default()
            }))),
            ..Default::default()
        }
    }

    /// A client-supplied `view_dependencies` that matches the derived set is
    /// accepted.
    #[tokio::test]
    async fn metric_view_matching_dependencies_accepted() {
        let h = handler();
        let created = h
            .create_table(
                CreateTableRequest {
                    view_dependencies: Some(DependencyList {
                        dependencies: vec![table_dep("cat.sch.orders")],
                        ..Default::default()
                    })
                    .into(),
                    ..metric_view_request()
                },
                ctx(),
            )
            .await
            .expect("create metric view with matching deps");
        assert_eq!(
            dep_names(created.view_dependencies.as_option()),
            vec!["cat.sch.orders"]
        );
    }

    /// A client-supplied `view_dependencies` that diverges from the definition
    /// is rejected.
    #[tokio::test]
    async fn metric_view_diverging_dependencies_rejected() {
        let h = handler();
        let res = h
            .create_table(
                CreateTableRequest {
                    view_dependencies: Some(DependencyList {
                        dependencies: vec![table_dep("cat.sch.something_else")],
                        ..Default::default()
                    })
                    .into(),
                    ..metric_view_request()
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    /// A metric view whose source cannot be resolved to a three-part name is
    /// rejected (strict derivation).
    #[tokio::test]
    async fn metric_view_unresolvable_source_rejected() {
        let h = handler();
        let yaml = "version: \"1.1\"\nsource: orders\n\
                    measures:\n  - name: revenue\n    expr: SUM(price)\n";
        let res = h
            .create_table(
                CreateTableRequest {
                    view_definition: Some(yaml.to_string()),
                    ..metric_view_request()
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    /// A metric view without a `view_definition` is rejected.
    #[tokio::test]
    async fn metric_view_without_definition_is_rejected() {
        let h = handler();
        let res = h
            .create_table(
                CreateTableRequest {
                    name: "orders_metrics".to_string(),
                    schema_name: "sch".to_string(),
                    catalog_name: "cat".to_string(),
                    table_type: TableType::MetricView.into(),
                    view_definition: None,
                    ..Default::default()
                },
                ctx(),
            )
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }
}
