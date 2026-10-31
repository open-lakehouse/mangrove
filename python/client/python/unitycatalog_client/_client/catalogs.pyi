"""
The classes of the `catalogs` API.
"""

from typing import final
from unitycatalog_client._enums import CatalogType

__all__ = ["Catalog", "CatalogClient", "ListCatalogsResponse"]

@final
class Catalog:
    """
    A catalog is a root-level namespace that contains schemas.
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
    def catalog_type(self, /) -> CatalogType | None:
        """
        The type of the catalog.
        """
    @property
    def comment(self, /) -> str | None:
        """
        User-provided free-form text description.
        """
    @property
    def created_at(self, /) -> int | None:
        """
        Time at which this catalog was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of catalog creator.
        """
    @property
    def id(self, /) -> str | None:
        """
        Unique identifier for the catalog.
        """
    @property
    def name(self, /) -> str:
        """
        Name of catalog.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of current owner of catalog.
        """
    @property
    def properties(self, /) -> dict[str, str]:
        """
        A map of key-value properties attached to the securable.
        """
    @property
    def provider_name(self, /) -> str | None:
        """
        The name of delta sharing provider.

        A Delta Sharing catalog is a catalog that is based on a Delta share on a remote sharing server.
        """
    @property
    def share_name(self, /) -> str | None:
        """
        The name of the share under the share provider.
        """
    @property
    def storage_location(self, /) -> str | None:
        """
        Storage location URL (full path) for managed storage of the catalog.

        A unique path under `storage_root`, used as the managed-storage parent for
        managed tables/volumes whose schema has no storage location of its own.
        Absent when the catalog has no `storage_root`.
        Example: `s3://bucket/ucroot/__unitystorage/catalogs/{catalog_id}`.
        """
    @property
    def storage_root(self, /) -> str | None:
        """
        Storage root URL for managed tables within catalog.
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this catalog was last updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of user who last modified catalog.
        """

@final
class CatalogClient:
    """
    The requests on one `Catalog`.
    """
    def delete(self, /, force: bool | None = None) -> None:
        """
        Delete a catalog

        Deletes the catalog that matches the supplied name. The caller must be a metastore admin or the
        owner of the catalog.

        Args:
            force: Force deletion even if the catalog is not empty.
        """
    def get(self, /, include_browse: bool | None = None) -> Catalog:
        """
        Get a catalog

        Gets the specified catalog in a metastore. The caller must be a metastore admin, the owner of the
        catalog, or a user that has the USE_CATALOG privilege set for their account.

        Args:
            include_browse: Whether to include catalogs in the response for which the principal can only
                            access selective metadata for

        Returns:
            A catalog is a root-level namespace that contains schemas.
        """
    def update(self, /, owner: str | None = None, comment: str | None = None, properties: dict[str, str] | None = None, new_name: str | None = None) -> Catalog:
        """
        Update a catalog

        Updates the catalog that matches the supplied name. The caller must be either the owner of the
        catalog, or a metastore admin (when changing the owner field of the catalog).

        Args:
            owner: Username of new owner of catalog.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.
            new_name: New name for the catalog.

        Returns:
            A catalog is a root-level namespace that contains schemas.
        """

@final
class ListCatalogsResponse:
    """
    List catalogs response.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def catalogs(self, /) -> list[Catalog]:
        """
        The catalogs returned.
        """
    @property
    def next_page_token(self, /) -> str | None:
        """
        The next_page_token value to include in the next List request.
        """
