"""
The classes of the `policies` API.
"""

from _typeshed import SupportsGetItem
from typing import final
from unitycatalog_client._enums import PolicyType

__all__ = ["FunctionArg", "FunctionRef", "ListPoliciesResponse", "MatchColumn", "PolicyClient", "PolicyInfo", "PrivilegeOptions"]

@final
class FunctionArg:
    """
    An argument passed to a row-filter or column-mask function.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, alias: str | None = None, constant: str | None = None) -> FunctionArg: ...
    def __repr__(self, /) -> str: ...
    @property
    def alias(self, /) -> str | None:
        """
        References a column via its `MatchColumn.alias`.
        """
    @property
    def constant(self, /) -> str | None:
        """
        A literal constant value, passed as-is to the function.
        """

@final
class FunctionRef:
    """
    A reference to the SQL function that implements a row-filter or column-mask policy,
    together with the arguments it is invoked with.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, function_name: str | None = None, using: SupportsGetItem[int, FunctionArg] | None = None, on_column: str | None = None) -> FunctionRef: ...
    def __repr__(self, /) -> str: ...
    @property
    def function_name(self, /) -> str:
        """
        Fully qualified name of the function (catalog.schema.function) to invoke.
        """
    @property
    def on_column(self, /) -> str | None:
        """
        The alias of the column to be masked; it must refer to one of the policy's match_columns.

        Required for column masks: the column value is passed to the mask function as its
        first argument. Not used for row filters.
        """
    @property
    def using(self, /) -> list[FunctionArg]:
        """
        The arguments passed to the function, in declaration order.
        """

@final
class ListPoliciesResponse:
    """
    List policies response.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def next_page_token(self, /) -> str | None:
        """
        The next_page_token value to include in the next List request.
        """
    @property
    def policies(self, /) -> list[PolicyInfo]:
        """
        The policies returned.
        """

@final
class MatchColumn:
    """
    A column referenced by a policy, together with the alias it is bound to in the
    row-filter or column-mask function invocation.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, alias: str | None = None, condition: str | None = None) -> MatchColumn: ...
    def __repr__(self, /) -> str: ...
    @property
    def alias(self, /) -> str | None:
        """
        The name of the alias the column is bound to when invoking the row-filter or
        column-mask function.
        """
    @property
    def condition(self, /) -> str:
        """
        The condition on the column, e.g. its name, that must match for this binding to apply.
        """

@final
class PolicyClient:
    """
    The requests on one `PolicyInfo`.
    """
    def delete(self, /) -> None:
        """
        Delete a policy

        Deletes the policy that matches the supplied name, defined on the specified securable.
        """
    def get(self, /) -> PolicyInfo:
        """
        Get a policy

        Gets the policy that matches the supplied name, defined on the specified securable.

        Returns:
            A row-filter or column-mask policy bound to principals and scoped to a securable. Policies
            are
            read as whole documents; enforcement is performed by the query engine, not by this service.
            `when_condition` is stored and returned opaque — this service does not parse or evaluate it.
        """
    def update(self, /, policy_type: PolicyType | str | None = None, comment: str | None = None, to_principals: SupportsGetItem[int, str] | None = None, except_principals: SupportsGetItem[int, str] | None = None, for_securable_type: str | None = None, when_condition: str | None = None, match_columns: SupportsGetItem[int, MatchColumn] | None = None, row_filter: FunctionRef | None = None, column_mask: FunctionRef | None = None, deny: PrivilegeOptions | None = None, grant: PrivilegeOptions | None = None, update_mask: str | None = None) -> PolicyInfo:
        """
        Update a policy

        Updates the policy that matches the supplied name, defined on the specified securable.

        Args:
            policy_type: The kind of enforcement this policy applies.
            comment: User-provided free-form text description of the policy.
            to_principals: The principals the policy applies to. Empty means "all principals".
            except_principals: The principals explicitly excluded from the policy, even if matched by
                               to_principals.
            for_securable_type: The type of securables the policy takes effect on.
            when_condition: An opaque condition expression evaluated by the query engine at enforcement
                            time.
            match_columns: The columns referenced by the policy function, and the aliases they are bound to.
            row_filter: The row-filter function to invoke. Set only when policy_type is
                        POLICY_TYPE_ROW_FILTER.
            column_mask: The column-mask function to invoke. Set only when policy_type is
                         POLICY_TYPE_COLUMN_MASK.
            deny: The privileges to deny. Set only when policy_type is POLICY_TYPE_DENY.
            grant: The privileges to grant. Set only when policy_type is POLICY_TYPE_GRANT.
            update_mask: The fields to update, as a comma-separated list of field names. If empty or "*",
                         every field set in the request is updated. Otherwise only the listed fields are
                         updated, and a listed field that is unset in the request is cleared.

        Returns:
            A row-filter or column-mask policy bound to principals and scoped to a securable. Policies
            are
            read as whole documents; enforcement is performed by the query engine, not by this service.
            `when_condition` is stored and returned opaque — this service does not parse or evaluate it.
        """

@final
class PolicyInfo:
    """
    A row-filter or column-mask policy bound to principals and scoped to a securable.

    Policies are read as whole documents; enforcement is performed by the query engine,
    not by this service. `when_condition` is stored and returned opaque — this service does
    not parse or evaluate it.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def column_mask(self, /) -> FunctionRef | None:
        """
        The column-mask function to invoke. Set only when policy_type is POLICY_TYPE_COLUMN_MASK.
        """
    @property
    def comment(self, /) -> str | None:
        """
        User-provided free-form text description of the policy.
        """
    @property
    def created_at(self, /) -> int | None:
        """
        Time at which this policy was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of the user who created the policy.
        """
    @property
    def deny(self, /) -> PrivilegeOptions | None:
        """
        The privileges to deny. Set only when policy_type is POLICY_TYPE_DENY.
        """
    @property
    def except_principals(self, /) -> list[str]:
        """
        The principals explicitly excluded from the policy, even if matched by to_principals.
        """
    @property
    def for_securable_type(self, /) -> str | None:
        """
        The type of securables the policy takes effect on, e.g. TABLE for a policy defined
        on a schema that applies to the tables within it.
        """
    @property
    def grant(self, /) -> PrivilegeOptions | None:
        """
        The privileges to grant. Set only when policy_type is POLICY_TYPE_GRANT.
        """
    @property
    def id(self, /) -> str | None:
        """
        Server-assigned unique identifier for the policy.
        """
    @property
    def match_columns(self, /) -> list[MatchColumn]:
        """
        The columns referenced by the row-filter or column-mask function, and the aliases
        they are bound to.

        Only valid when for_securable_type is TABLE; the policy then applies only to tables
        whose columns satisfy all match conditions.
        """
    @property
    def name(self, /) -> str:
        """
        The name of the policy. Unique within the securable it is defined on.
        """
    @property
    def on_securable_fullname(self, /) -> str:
        """
        The fully qualified name of the securable the policy is defined on.
        """
    @property
    def on_securable_type(self, /) -> str:
        """
        The type of securable the policy is defined on.

        Supported values: CATALOG, SCHEMA, TABLE.
        """
    @property
    def policy_type(self, /) -> PolicyType | None:
        """
        The kind of enforcement this policy applies.
        """
    @property
    def row_filter(self, /) -> FunctionRef | None:
        """
        The row-filter function to invoke. Set only when policy_type is POLICY_TYPE_ROW_FILTER.
        """
    @property
    def to_principals(self, /) -> list[str]:
        """
        The principals the policy applies to. Empty means "all principals".
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this policy was last updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of the user who last modified the policy.
        """
    @property
    def when_condition(self, /) -> str | None:
        """
        An opaque condition expression, e.g. "hasTagValue('env','prod')", evaluated by the
        query engine at enforcement time. Stored and returned as-is; not parsed server-side.
        """

@final
class PrivilegeOptions:
    """
    The privileges a deny or grant policy applies to.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, privileges: SupportsGetItem[int, str] | None = None) -> PrivilegeOptions: ...
    def __repr__(self, /) -> str: ...
    @property
    def privileges(self, /) -> list[str]:
        """
        The privileges to deny or grant when the principal and condition match.
        """
