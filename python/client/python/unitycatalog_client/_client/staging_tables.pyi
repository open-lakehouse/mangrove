"""
The classes of the `staging_tables` API.
"""

from typing import final

__all__ = ["StagingTable", "StagingTableClient"]

@final
class StagingTable:
    """
    A staging table is a transient reservation created while provisioning a
    Unity Catalog managed table.

    On creation the server allocates an immutable table id and storage location
    under the parent schema/catalog's managed storage root. The client then
    writes the initial Delta commit (`_delta_log/00...0.json`) at that location
    and finalizes the table via the CreateTable API, which commits this staging
    row. A staging table is not yet a usable table and is not returned by the
    table APIs.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def catalog_name(self, /) -> str:
        """
        Name of the parent catalog.
        """
    @property
    def created_at(self, /) -> int | None:
        """
        Time at which this staging table was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of the principal that created the staging table. Only this
        principal may finalize the table.
        """
    @property
    def id(self, /) -> str:
        """
        Unique identifier generated for the staging table. Becomes the managed
        table's id once finalized.
        """
    @property
    def name(self, /) -> str:
        """
        Name of the staging table, relative to the parent schema.
        """
    @property
    def schema_name(self, /) -> str:
        """
        Name of the parent schema relative to its parent catalog.
        """
    @property
    def stage_committed(self, /) -> bool:
        """
        Whether the staging table has been finalized into a managed table.
        """
    @property
    def staging_location(self, /) -> str:
        """
        Storage location allocated for the staging table, where the client writes
        the initial Delta commit. Immutable once allocated.
        """

@final
class StagingTableClient:
    """
    The requests on one `StagingTable`.
    """
