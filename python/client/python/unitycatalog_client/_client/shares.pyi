"""
The classes of the `shares` API.
"""

from _typeshed import SupportsGetItem
from typing import SupportsIndex, final
from unitycatalog_client._enums import Action, DataObjectStatus, DataObjectType, HistoryStatus, PartitionValueOp

__all__ = ["DataObject", "DataObjectUpdate", "GetPermissionsResponse", "ListSharesResponse", "Partition", "PartitionValue", "PermissionsChange", "PrivilegeAssignment", "Share", "ShareClient", "UpdatePermissionsResponse"]

@final
class DataObject:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, name: str | None = None, data_object_type: DataObjectType | str | None = None, added_at: SupportsIndex | None = None, added_by: str | None = None, comment: str | None = None, shared_as: str | None = None, partitions: SupportsGetItem[int, Partition] | None = None, cdf_enabled: bool | None = None, history_data_sharing_status: HistoryStatus | str | None = None, start_version: SupportsIndex | None = None, string_shared_as: str | None = None, status: DataObjectStatus | str | None = None) -> DataObject: ...
    def __repr__(self, /) -> str: ...
    @property
    def added_at(self, /) -> int | None:
        """
        The time when this data object is added to the share, in epoch milliseconds.
        """
    @property
    def added_by(self, /) -> str | None:
        """
        Username of the sharer.
        """
    @property
    def cdf_enabled(self, /) -> bool | None:
        """
        Whether to enable cdf or indicate if cdf is enabled on the shared object.
        """
    @property
    def comment(self, /) -> str | None:
        """
        A user-provided comment when adding the data object to the share.
        """
    @property
    def data_object_type(self, /) -> DataObjectType | None:
        """
        Type of the data object.
        """
    @property
    def history_data_sharing_status(self, /) -> HistoryStatus | None:
        """
        Whether to enable or disable sharing of data history. If not specified, the default is DISABLED.
        """
    @property
    def name(self, /) -> str:
        """
        A fully qualified name that uniquely identifies a data object.

        For example, a table's fully qualified name is in the format of <catalog>.<schema>.<table>,
        """
    @property
    def partitions(self, /) -> list[Partition]:
        """
        Array of partitions for the shared data.
        """
    @property
    def shared_as(self, /) -> str | None:
        """
        A user-provided new name for the data object within the share.

        If this new name is not provided, the object's original name will be used as the shared_as name.
        The shared_as name must be unique within a share.
        For tables, the new name must follow the format of <schema>.<table>: two parts, each
        consisting only of alphanumeric characters and underscores, without the catalog part.
        Do not set it for non-table data objects; use string_shared_as instead.
        """
    @property
    def start_version(self, /) -> int | None:
        """
        The start version associated with the object.

        This allows data providers to control the lowest object version that is accessible by clients.
        If specified, clients can query snapshots or changes for versions >= start_version.
        If not specified, clients can only query starting from the version of the object at the time it was added to the share.

        NOTE: The start_version should be <= the current version of the object.
        """
    @property
    def status(self, /) -> DataObjectStatus | None:
        """
        Whether the data object is still accessible to the share.
        """
    @property
    def string_shared_as(self, /) -> str | None:
        """
        A user-provided alias for a non-table data object (e.g. a volume) within the share.

        For volumes, functions and models it is a two-part name <schema>.<name>, each part
        consisting only of alphanumeric characters and underscores. If not provided, the
        object's original name minus its catalog is used. It must be unique among objects of
        the same type within the share. Do not set it for table-like data objects; use
        shared_as instead.
        """

@final
class DataObjectUpdate:
    """
    Data object update.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, action: Action | str | None = None, data_object: DataObject | None = None) -> DataObjectUpdate: ...
    def __repr__(self, /) -> str: ...
    @property
    def action(self, /) -> Action | None:
        """
        The action to perform on the data object.
        """
    @property
    def data_object(self, /) -> DataObject | None:
        """
        The data object that is being added, removed, or updated.
        """

@final
class GetPermissionsResponse:
    """
    Response to list shares.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def next_page_token(self, /) -> str | None:
        """
        Opaque pagination token to go to next page based on previous query.
        """
    @property
    def privilege_assignments(self, /) -> list[PrivilegeAssignment]:
        """
        The privileges assigned to each principal
        """

@final
class ListSharesResponse:
    """
    Response to list shares.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def next_page_token(self, /) -> str | None:
        """
        Opaque pagination token to go to next page based on previous query.
        """
    @property
    def shares(self, /) -> list[Share]:
        """
        List of shares.
        """

@final
class Partition:
    """
    A partition specification: the conjunction of its partition values.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, values: SupportsGetItem[int, PartitionValue] | None = None) -> Partition: ...
    def __repr__(self, /) -> str: ...
    @property
    def values(self, /) -> list[PartitionValue]:
        """
        An array of partition values.
        """

@final
class PartitionValue:
    """
    A partition column value a shared table is restricted to.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, name: str | None = None, value: str | None = None, recipient_property_key: str | None = None, op: PartitionValueOp | str | None = None) -> PartitionValue: ...
    def __repr__(self, /) -> str: ...
    @property
    def name(self, /) -> str:
        """
        The name of the partition column.
        """
    @property
    def op(self, /) -> PartitionValueOp | None:
        """
        The operator to apply for the value.
        """
    @property
    def recipient_property_key(self, /) -> str | None:
        """
        The key of a Delta Sharing recipient property whose value the partition column is
        matched against, e.g. "databricks-account-id".

        Mutually exclusive with value.
        """
    @property
    def value(self, /) -> str | None:
        """
        The value of the partition column; unset means a `null` value.

        Mutually exclusive with recipient_property_key.
        """

@final
class PermissionsChange:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, principal: str | None = None, add: SupportsGetItem[int, str] | None = None, remove: SupportsGetItem[int, str] | None = None) -> PermissionsChange: ...
    def __repr__(self, /) -> str: ...
    @property
    def add(self, /) -> list[str]:
        """
        The set of privileges to add.
        """
    @property
    def principal(self, /) -> str:
        """
        The principal (user email address or group name).
        """
    @property
    def remove(self, /) -> list[str]:
        """
        The set of privileges to remove.
        """

@final
class PrivilegeAssignment:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def principal(self, /) -> str:
        """
        The principal (user email address or group name).

        For deleted principals, principal is empty while principal_id is populated.
        """
    @property
    def privileges(self, /) -> list[str]:
        """
        The privileges assigned to the principal.
        """

@final
class Share:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def comment(self, /) -> str | None:
        """
        User-provided free-form text description.
        """
    @property
    def created_at(self, /) -> int | None:
        """
        Time at which this share was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of the creator of the share.
        """
    @property
    def id(self, /) -> str | None:
        """
        Unique ID of the share.
        """
    @property
    def name(self, /) -> str:
        """
        Name of the share.
        """
    @property
    def objects(self, /) -> list[DataObject]:
        """
        A list of shared data objects within the share.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of current owner of share.
        """
    @property
    def storage_location(self, /) -> str | None:
        """
        Storage Location URL (full path) for the share.
        """
    @property
    def storage_root(self, /) -> str | None:
        """
        Storage root URL for the share.
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this share was updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of share updater.
        """

@final
class ShareClient:
    """
    The requests on one `Share`.
    """
    def delete(self, /) -> None:
        """
        Deletes a share.
        """
    def get(self, /, include_shared_data: bool | None = None) -> Share:
        """
        Get a share by name.

        Args:
            include_shared_data: Query for data to include in the share.

        Returns:
            The requested resource
        """
    def get_permissions(self, /, max_results: SupportsIndex | None = None, page_token: str | None = None) -> GetPermissionsResponse:
        """
        Gets the permissions for a data share from the metastore.

        Args:
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.
            page_token: Opaque pagination token to go to next page based on previous query.

        Returns:
            Response to list shares.
        """
    def update(self, /, updates: SupportsGetItem[int, DataObjectUpdate] | None = None, new_name: str | None = None, owner: str | None = None, comment: str | None = None) -> Share:
        """
        Update a share.

        Args:
            updates: Array of shared data object updates. At most 100 data objects may be updated per
                     request.
            new_name: A new name for the share.
            owner: Owner of the share.
            comment: User-provided free-form text description.

        Returns:
            The requested resource
        """
    def update_permissions(self, /, changes: SupportsGetItem[int, PermissionsChange] | None = None, omit_permissions_list: bool | None = None) -> UpdatePermissionsResponse:
        """
        Updates the permissions for a data share in the metastore.

        Args:
            changes: Array of permissions change objects.
            omit_permissions_list: Whether to return the latest permissions list of the share in the
                                   response.

        Returns:
            The requested resource
        """

@final
class UpdatePermissionsResponse:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def privilege_assignments(self, /) -> list[PrivilegeAssignment]:
        """
        The privileges assigned to each principal
        """
