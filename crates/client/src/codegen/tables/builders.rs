// @generated — do not edit by hand.
#![allow(unused_mut)]
#![allow(unused_imports)]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
type BoxStr<'a, T> = ::futures::stream::BoxStream<'a, T>;
use super::super::stream_paginated;
use super::client::*;
use crate::Result;
use futures::{StreamExt, TryStreamExt};
use std::future::IntoFuture;
use unitycatalog_common::models::tables::v1::*;
/// Builder for table summaries
pub struct ListTableSummariesBuilder {
    client: TableServiceClient,
    request: ListTableSummariesRequest,
}
impl ListTableSummariesBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TableServiceClient`.
    pub(crate) fn new(client: TableServiceClient, catalog_name: impl Into<String>) -> Self {
        let request = ListTableSummariesRequest {
            catalog_name: catalog_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// A sql LIKE pattern (% and _) for schema names. All schemas will be returned if not set or empty.
    pub fn with_schema_name_pattern(
        mut self,
        schema_name_pattern: impl Into<Option<String>>,
    ) -> Self {
        self.request.schema_name_pattern = schema_name_pattern.into();
        self
    }
    /// A sql LIKE pattern (% and _) for table names. All tables will be returned if not set or empty.
    pub fn with_table_name_pattern(
        mut self,
        table_name_pattern: impl Into<Option<String>>,
    ) -> Self {
        self.request.table_name_pattern = table_name_pattern.into();
        self
    }
    /// The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    /// Opaque pagination token to go to next page based on previous query.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
    /// Whether to include a manifest containing capabilities the table has.
    pub fn with_include_manifest_capabilities(
        mut self,
        include_manifest_capabilities: impl Into<Option<bool>>,
    ) -> Self {
        self.request.include_manifest_capabilities = include_manifest_capabilities.into();
        self
    }
}
impl IntoFuture for ListTableSummariesBuilder {
    type Output = Result<ListTableSummariesResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_table_summaries(&request).await })
    }
}
/// Builder for listing tables
pub struct ListTablesBuilder {
    client: TableServiceClient,
    request: ListTablesRequest,
}
impl ListTablesBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TableServiceClient`.
    pub(crate) fn new(
        client: TableServiceClient,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> Self {
        let request = ListTablesRequest {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    /// Opaque pagination token to go to next page based on previous query.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
    /// Whether delta metadata should be included in the response.
    pub fn with_include_delta_metadata(
        mut self,
        include_delta_metadata: impl Into<Option<bool>>,
    ) -> Self {
        self.request.include_delta_metadata = include_delta_metadata.into();
        self
    }
    /// Whether to omit the columns of the table from the response or not.
    pub fn with_omit_columns(mut self, omit_columns: impl Into<Option<bool>>) -> Self {
        self.request.omit_columns = omit_columns.into();
        self
    }
    /// Whether to omit the properties of the table from the response or not.
    pub fn with_omit_properties(mut self, omit_properties: impl Into<Option<bool>>) -> Self {
        self.request.omit_properties = omit_properties.into();
        self
    }
    /// Whether to omit the username of the table (e.g. owner, updated_by, created_by) from the response or not.
    pub fn with_omit_username(mut self, omit_username: impl Into<Option<bool>>) -> Self {
        self.request.omit_username = omit_username.into();
        self
    }
    /// Whether to include tables in the response for which the principal can only access selective metadata for
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
    /// Whether to include a manifest containing capabilities the table has.
    pub fn with_include_manifest_capabilities(
        mut self,
        include_manifest_capabilities: impl Into<Option<bool>>,
    ) -> Self {
        self.request.include_manifest_capabilities = include_manifest_capabilities.into();
        self
    }
    /// Convert paginated request into stream of results
    pub fn into_stream(self) -> BoxStr<'static, Result<Table>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.list_tables(&builder.request).await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.tables.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.tables.into_iter().map(Ok)))
        .try_flatten();
        stream.boxed()
    }
}
impl IntoFuture for ListTablesBuilder {
    type Output = Result<ListTablesResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_tables(&request).await })
    }
}
/// Builder for creating a table
pub struct CreateTableBuilder {
    client: TableServiceClient,
    request: CreateTableRequest,
}
impl CreateTableBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TableServiceClient`.
    pub(crate) fn new(
        client: TableServiceClient,
        name: impl Into<String>,
        schema_name: impl Into<String>,
        catalog_name: impl Into<String>,
        table_type: TableType,
        data_source_format: DataSourceFormat,
    ) -> Self {
        let request = CreateTableRequest {
            name: name.into(),
            schema_name: schema_name.into(),
            catalog_name: catalog_name.into(),
            table_type: buffa::EnumValue::Known(table_type),
            data_source_format: buffa::EnumValue::Known(data_source_format),
            ..Default::default()
        };
        Self { client, request }
    }
    /// The array of Column definitions of the table's columns.
    pub fn with_columns<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = Column>,
    {
        self.request.columns = columns.into_iter().collect();
        self
    }
    /// Storage root URL for external table.
    pub fn with_storage_location(mut self, storage_location: impl Into<Option<String>>) -> Self {
        self.request.storage_location = storage_location.into();
        self
    }
    /// User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /// A map of key-value properties attached to the securable.
    pub fn with_properties<I, K, V>(mut self, properties: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.request.properties = properties
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }
    /** Definition text for view-like table types (VIEW, MATERIALIZED_VIEW,
    STREAMING_TABLE, METRIC_VIEW). The format depends on the table type:
    SQL for views, YAML for metric views. Required for METRIC_VIEW.*/
    pub fn with_view_definition(mut self, view_definition: impl Into<Option<String>>) -> Self {
        self.request.view_definition = view_definition.into();
        self
    }
    /** Tables and functions the view-like table reads. For metric views the server
    derives this from view_definition and rejects a supplied list that diverges
    from the derived set (the definition is the single source of truth).*/
    pub fn with_view_dependencies(
        mut self,
        view_dependencies: impl Into<Option<DependencyList>>,
    ) -> Self {
        self.request.view_dependencies = {
            let view_dependencies: ::core::option::Option<_> = view_dependencies.into();
            buffa::MessageField::from(view_dependencies)
        };
        self
    }
}
impl IntoFuture for CreateTableBuilder {
    type Output = Result<Table>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_table(&request).await })
    }
}
/// Builder for getting a table
pub struct GetTableBuilder {
    client: TableServiceClient,
    request: GetTableRequest,
}
impl GetTableBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TableServiceClient`.
    pub(crate) fn new(client: TableServiceClient, full_name: impl Into<String>) -> Self {
        let request = GetTableRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Whether delta metadata should be included in the response.
    pub fn with_include_delta_metadata(
        mut self,
        include_delta_metadata: impl Into<Option<bool>>,
    ) -> Self {
        self.request.include_delta_metadata = include_delta_metadata.into();
        self
    }
    /// Whether to include tables in the response for which the principal can only access selective metadata for
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
    /// Whether to include a manifest containing capabilities the table has.
    pub fn with_include_manifest_capabilities(
        mut self,
        include_manifest_capabilities: impl Into<Option<bool>>,
    ) -> Self {
        self.request.include_manifest_capabilities = include_manifest_capabilities.into();
        self
    }
}
impl IntoFuture for GetTableBuilder {
    type Output = Result<Table>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_table(&request).await })
    }
}
/// Builder for table exists
pub struct GetTableExistsBuilder {
    client: TableServiceClient,
    request: GetTableExistsRequest,
}
impl GetTableExistsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TableServiceClient`.
    pub(crate) fn new(client: TableServiceClient, full_name: impl Into<String>) -> Self {
        let request = GetTableExistsRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetTableExistsBuilder {
    type Output = Result<GetTableExistsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_table_exists(&request).await })
    }
}
/// Builder for deleting a table
pub struct DeleteTableBuilder {
    client: TableServiceClient,
    request: DeleteTableRequest,
}
impl DeleteTableBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TableServiceClient`.
    pub(crate) fn new(client: TableServiceClient, full_name: impl Into<String>) -> Self {
        let request = DeleteTableRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteTableBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_table(&request).await })
    }
}
