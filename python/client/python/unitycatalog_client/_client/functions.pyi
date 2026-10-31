"""
The classes of the `functions` API.
"""

from .tables import DependencyList
from _typeshed import SupportsGetItem
from typing import SupportsIndex, final
from unitycatalog_client._enums import ColumnTypeName, FunctionParameterType, ParameterMode, ParameterStyle, RoutineBody, SecurityType, SqlDataAccess

__all__ = ["CreateFunction", "Function", "FunctionClient", "FunctionParameterInfo", "FunctionParameterInfos", "ListFunctionsResponse"]

@final
class CreateFunction:
    """
    The payload for creating a new function.

    Wrapped in the `function_info` envelope of a `CreateFunctionRequest`, matching
    the Unity Catalog `POST /functions` wire contract.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, name: str | None = None, catalog_name: str | None = None, schema_name: str | None = None, data_type: str | None = None, full_data_type: str | None = None, input_params: FunctionParameterInfos | None = None, parameter_style: ParameterStyle | str | None = None, is_deterministic: bool | None = None, sql_data_access: SqlDataAccess | str | None = None, is_null_call: bool | None = None, security_type: SecurityType | str | None = None, routine_body: RoutineBody | str | None = None, routine_definition: str | None = None, external_language: str | None = None, comment: str | None = None, properties: str | None = None, return_params: FunctionParameterInfos | None = None, routine_dependencies: DependencyList | None = None, specific_name: str | None = None) -> CreateFunction: ...
    def __repr__(self, /) -> str: ...
    @property
    def catalog_name(self, /) -> str:
        """
        Name of parent catalog.
        """
    @property
    def comment(self, /) -> str | None:
        """
        User-provided free-form text description.
        """
    @property
    def data_type(self, /) -> str:
        """
        Full data type specification of the return type of the function.
        """
    @property
    def external_language(self, /) -> str | None:
        """
        External language of the function.

        Set when routine_body is EXTERNAL.
        """
    @property
    def full_data_type(self, /) -> str:
        """
        Full data type specification as SQL/catalogString text.
        """
    @property
    def input_params(self, /) -> FunctionParameterInfos | None:
        """
        The array of function parameter infos.
        """
    @property
    def is_deterministic(self, /) -> bool:
        """
        Indicates whether the function is deterministic.
        """
    @property
    def is_null_call(self, /) -> bool:
        """
        Indicates whether the function is null-calling.
        """
    @property
    def name(self, /) -> str:
        """
        Name of function, relative to parent schema.
        """
    @property
    def parameter_style(self, /) -> ParameterStyle | None:
        """
        The parameter-passing style.
        """
    @property
    def properties(self, /) -> str | None:
        """
        JSON-serialized key-value pair map, encoded (escaped) as a string.
        """
    @property
    def return_params(self, /) -> FunctionParameterInfos | None:
        """
        The return parameters of a table-valued function.
        """
    @property
    def routine_body(self, /) -> RoutineBody | None:
        """
        The routine body.
        """
    @property
    def routine_definition(self, /) -> str | None:
        """
        Function body.
        """
    @property
    def routine_dependencies(self, /) -> DependencyList | None:
        """
        The tables and functions the function depends on.
        """
    @property
    def schema_name(self, /) -> str:
        """
        Name of parent schema.
        """
    @property
    def security_type(self, /) -> SecurityType | None:
        """
        The security type of the function.
        """
    @property
    def specific_name(self, /) -> str | None:
        """
        Specific name of the function; reserved for future use.
        """
    @property
    def sql_data_access(self, /) -> SqlDataAccess | None:
        """
        SQL data access information.
        """

@final
class Function:
    """
    A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def browse_only(self, /) -> bool | None:
        """
        Indicates whether the principal is limited to retrieving metadata
        for the associated object through the BROWSE privilege when include_browse
        is enabled in the request.
        """
    @property
    def catalog_name(self, /) -> str:
        """
        Name of parent catalog.
        """
    @property
    def comment(self, /) -> str | None:
        """
        User-provided free-form text description.
        """
    @property
    def created_at(self, /) -> int | None:
        """
        Time at which this function was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of function creator.
        """
    @property
    def data_type(self, /) -> str:
        """
        Full data type specification of the return type of the function.
        """
    @property
    def external_language(self, /) -> str | None:
        """
        External language of the function.

        Set when routine_body is EXTERNAL.
        """
    @property
    def full_data_type(self, /) -> str:
        """
        Full data type specification as SQL/catalogString text.
        """
    @property
    def full_name(self, /) -> str:
        """
        The three-level (fully qualified) name of the function.

        Format: catalog_name.schema_name.function_name
        """
    @property
    def function_id(self, /) -> str | None:
        """
        Unique identifier for the function.
        """
    @property
    def input_params(self, /) -> FunctionParameterInfos | None:
        """
        The array of function parameter infos.
        """
    @property
    def is_deterministic(self, /) -> bool:
        """
        Indicates whether the function is deterministic.
        """
    @property
    def is_null_call(self, /) -> bool:
        """
        Indicates whether the function is null-calling.
        """
    @property
    def name(self, /) -> str:
        """
        Name of function, relative to parent schema.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of current owner of the function.
        """
    @property
    def parameter_style(self, /) -> ParameterStyle | None:
        """
        The parameter-passing style.
        """
    @property
    def properties(self, /) -> str | None:
        """
        JSON-serialized key-value pair map, encoded (escaped) as a string.
        """
    @property
    def return_params(self, /) -> FunctionParameterInfos | None:
        """
        The return parameters of a table-valued function.
        """
    @property
    def routine_body(self, /) -> RoutineBody | None:
        """
        The routine body.
        """
    @property
    def routine_definition(self, /) -> str | None:
        """
        Function body.
        """
    @property
    def routine_dependencies(self, /) -> DependencyList | None:
        """
        The tables and functions the function depends on.
        """
    @property
    def schema_name(self, /) -> str:
        """
        Name of parent schema.
        """
    @property
    def security_type(self, /) -> SecurityType | None:
        """
        The security type of the function.
        """
    @property
    def specific_name(self, /) -> str | None:
        """
        Specific name of the function; reserved for future use.
        """
    @property
    def sql_data_access(self, /) -> SqlDataAccess | None:
        """
        SQL data access information.
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this function was last updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of user who last modified the function.
        """

@final
class FunctionClient:
    """
    The requests on one `Function`.
    """
    def delete(self, /, force: bool | None = None) -> None:
        """
        Delete a function

        Deletes the function that matches the supplied name. For the deletion to succeed, the caller must be
        the owner of the function.

        Args:
            force: Force deletion even if the function is not empty.
        """
    def get(self, /, include_browse: bool | None = None) -> Function:
        """
        Get a function

        Gets a function from within a parent catalog and schema. For the fetch to succeed, the caller must
        be a metastore admin, the owner of the function, or have SELECT on the function.

        Args:
            include_browse: Whether to include functions in the response for which the principal can only
                            access selective metadata for

        Returns:
            A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
        """
    def update(self, /, owner: str | None = None) -> Function:
        """
        Update a function

        Updates the function that matches the supplied name. Only the owner of the function can be updated.

        Args:
            owner: Username of new owner of the function.

        Returns:
            A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
        """

@final
class FunctionParameterInfo:
    """
    Information about a single function parameter.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, name: str | None = None, type_text: str | None = None, type_json: str | None = None, type_name: ColumnTypeName | str | None = None, type_precision: SupportsIndex | None = None, type_scale: SupportsIndex | None = None, type_interval_type: str | None = None, position: SupportsIndex | None = None, parameter_mode: ParameterMode | str | None = None, parameter_type: FunctionParameterType | str | None = None, parameter_default: str | None = None, comment: str | None = None) -> FunctionParameterInfo: ...
    def __repr__(self, /) -> str: ...
    @property
    def comment(self, /) -> str | None:
        """
        User-provided free-form text description.
        """
    @property
    def name(self, /) -> str:
        """
        Name of parameter.
        """
    @property
    def parameter_default(self, /) -> str | None:
        """
        Default value of the parameter.
        """
    @property
    def parameter_mode(self, /) -> ParameterMode | None:
        """
        The mode of the function parameter.
        """
    @property
    def parameter_type(self, /) -> FunctionParameterType | None:
        """
        The type of function parameter.
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
    def type_json(self, /) -> str | None:
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
        Digits to right of decimal; required for DecimalTypes.
        """
    @property
    def type_text(self, /) -> str:
        """
        Full data type specification as SQL/catalogString text.
        """

@final
class FunctionParameterInfos:
    """
    A collection of function parameters.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, parameters: SupportsGetItem[int, FunctionParameterInfo] | None = None) -> FunctionParameterInfos: ...
    def __repr__(self, /) -> str: ...
    @property
    def parameters(self, /) -> list[FunctionParameterInfo]:
        """
        The parameters of the function.
        """

@final
class ListFunctionsResponse:
    """
    List functions response.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def functions(self, /) -> list[Function]:
        """
        The functions returned.
        """
    @property
    def next_page_token(self, /) -> str | None:
        """
        The next_page_token value to include in the next List request.
        """
