"""
The classes of the `tables` API.
"""

from _typeshed import SupportsGetItem
from typing import SupportsIndex, final
from unitycatalog_client._enums import ColumnTypeName, DataSourceFormat, TableType

__all__ = ["Column", "Dependency", "DependencyList", "FunctionDependency", "GetTableExistsResponse", "ListTableSummariesResponse", "ListTablesResponse", "Table", "TableClient", "TableDependency", "TableSummary"]

@final
class Column:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, name: str | None = None, type_text: str | None = None, type_json: str | None = None, position: SupportsIndex | None = None, type_name: ColumnTypeName | str | None = None, type_precision: SupportsIndex | None = None, type_scale: SupportsIndex | None = None, type_interval_type: str | None = None, comment: str | None = None, nullable: bool | None = None, partition_index: SupportsIndex | None = None, column_id: str | None = None) -> Column: ...
    def __repr__(self, /) -> str: ...
    @property
    def column_id(self, /) -> str | None:
        """
        a unique id for the column
        """
    @property
    def comment(self, /) -> str | None:
        """
        User-provided free-form text description.
        """
    @property
    def name(self, /) -> str:
        """
        Name of the column
        """
    @property
    def nullable(self, /) -> bool | None:
        """
        Whether field may be Null (default: true).
        """
    @property
    def partition_index(self, /) -> int | None:
        """
        Partition index for column.
        """
    @property
    def position(self, /) -> int | None:
        """
        Ordinal position of column (starting at position 0).
        """
    @property
    def type_interval_type(self, /) -> str | None:
        """
        Format of IntervalType.
        """
    @property
    def type_json(self, /) -> str:
        """
        Full data type specification, JSON-serialized.
        """
    @property
    def type_name(self, /) -> ColumnTypeName | None:
        """
        Data type name.
        """
    @property
    def type_precision(self, /) -> int | None:
        """
        Digits of precision; required for DecimalTypes.
        """
    @property
    def type_scale(self, /) -> int | None:
        """
        Digits to right of decimal; Required for DecimalTypes.
        """
    @property
    def type_text(self, /) -> str:
        """
        Full data type specification as SQL/catalogString text.
        """

@final
class Dependency:
    """
    A dependency of a SQL object. Exactly one of the fields must be set.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, table: TableDependency | None = None, function: FunctionDependency | None = None) -> Dependency: ...
    def __repr__(self, /) -> str: ...
    @property
    def function(self, /) -> FunctionDependency | None: ...
    @property
    def table(self, /) -> TableDependency | None: ...

@final
class DependencyList:
    """
    A list of dependencies referenced by a view-like table.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, dependencies: SupportsGetItem[int, Dependency] | None = None) -> DependencyList: ...
    def __repr__(self, /) -> str: ...
    @property
    def dependencies(self, /) -> list[Dependency]:
        """
        Array of dependencies.
        """

@final
class FunctionDependency:
    """
    A function that a SQL object (such as a view or metric view) depends on.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, function_full_name: str | None = None) -> FunctionDependency: ...
    def __repr__(self, /) -> str: ...
    @property
    def function_full_name(self, /) -> str:
        """
        Full name of the dependent function, in the form of
        catalog_name.schema_name.function_name.
        """

@final
class GetTableExistsResponse:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def table_exists(self, /) -> bool:
        """
        Boolean reflecting if table exists.
        """

@final
class ListTableSummariesResponse:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def next_page_token(self, /) -> str | None:
        """
        The next_page_token value to include in the next List request.
        """
    @property
    def tables(self, /) -> list[TableSummary]:
        """
        The table summaries returned.
        """

@final
class ListTablesResponse:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def next_page_token(self, /) -> str | None:
        """
        The next_page_token value to include in the next List request.
        """
    @property
    def tables(self, /) -> list[Table]:
        """
        The tables returned.
        """

@final
class Table:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def catalog_name(self, /) -> str:
        """
        Name of parent catalog.
        """
    @property
    def columns(self, /) -> list[Column]:
        """
        The array of Column definitions of the table's columns.
        """
    @property
    def comment(self, /) -> str | None:
        """
        User-provided free-form text description.
        """
    @property
    def created_at(self, /) -> int | None:
        """
        Time at which this table was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of table creator.
        """
    @property
    def data_source_format(self, /) -> DataSourceFormat | None:
        """
        Data source format of the table.
        """
    @property
    def deleted_at(self, /) -> int | None:
        """
        Time at which this table was deleted, in epoch milliseconds. Field is omitted if table is not deleted.
        """
    @property
    def full_name(self, /) -> str:
        """
        Full name of table, in form of catalog_name.schema_name.table_name.
        """
    @property
    def name(self, /) -> str:
        """
        Name of table, relative to parent schema.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of current owner of table.
        """
    @property
    def properties(self, /) -> dict[str, str]:
        """
        A map of key-value properties attached to the securable.
        """
    @property
    def schema_name(self, /) -> str:
        """
        Name of parent schema.
        """
    @property
    def storage_credential_name(self, /) -> str | None:
        """
        Name of the storage credential, when a storage credential is configured for use with this table.
        """
    @property
    def storage_location(self, /) -> str | None:
        """
        Storage root URL for table (for MANAGED, EXTERNAL tables)
        """
    @property
    def table_id(self, /) -> str | None:
        """
        Unique identifier for the table.
        """
    @property
    def table_type(self, /) -> TableType | None: ...
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this table was last updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of user who last modified table.
        """
    @property
    def view_definition(self, /) -> str | None:
        """
        Definition text for view-like table types (VIEW, MATERIALIZED_VIEW,
        STREAMING_TABLE, METRIC_VIEW). The format depends on the table type:
        SQL for views, YAML for metric views.
        """
    @property
    def view_dependencies(self, /) -> DependencyList | None:
        """
        Tables and functions the view-like table reads. For metric views this is
        derived from the view_definition by the server (the definition is the
        single source of truth).

        When unset, dependencies were not provided; an empty list means there are none.
        """

@final
class TableClient:
    """
    The requests on one `Table`.
    """
    def delete(self, /) -> None:
        """
        Delete a table
        """
    def get(self, /, include_delta_metadata: bool | None = None, include_browse: bool | None = None, include_manifest_capabilities: bool | None = None) -> Table:
        """
        Get a table

        Args:
            include_delta_metadata: Whether delta metadata should be included in the response.
            include_browse: Whether to include tables in the response for which the principal can only
                            access selective metadata for
            include_manifest_capabilities: Whether to include a manifest containing capabilities the table
                                           has.

        Returns:
            The requested resource
        """
    def get_table_exists(self, /) -> GetTableExistsResponse:
        """
        Get boolean reflecting if table exists

        Returns:
            The requested resource
        """

@final
class TableDependency:
    """
    A table that a SQL object (such as a view or metric view) depends on.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, table_full_name: str | None = None) -> TableDependency: ...
    def __repr__(self, /) -> str: ...
    @property
    def table_full_name(self, /) -> str:
        """
        Full name of the dependent table, in the form of
        catalog_name.schema_name.table_name.
        """

@final
class TableSummary:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def full_name(self, /) -> str:
        """
        The full name of the table.
        """
    @property
    def table_type(self, /) -> TableType | None: ...
