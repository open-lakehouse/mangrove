"""
The classes of the `volumes` API.
"""

from typing import final
from unitycatalog_client._enums import VolumeType

__all__ = ["ListVolumesResponse", "Volume", "VolumeClient"]

@final
class ListVolumesResponse:
    """
    List volumes response.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def next_page_token(self, /) -> str | None:
        """
        Opaque token to retrieve the next page of results.

        Absent if there are no more pages. page_token should be set to
        this value for the next request to retrieve the next page of results.
        """
    @property
    def volumes(self, /) -> list[Volume]:
        """
        The volumes returned.
        """

@final
class Volume:
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
        Time at which this volume was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of volume creator.
        """
    @property
    def full_name(self, /) -> str:
        """
        The three-level (fully qualified) name of the volume
        """
    @property
    def metastore_id(self, /) -> str | None:
        """
        The unique identifier of the metastore
        """
    @property
    def name(self, /) -> str:
        """
        Name of volume, relative to parent schema.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of current owner of volume.
        """
    @property
    def schema_name(self, /) -> str:
        """
        Name of parent schema.
        """
    @property
    def storage_location(self, /) -> str:
        """
        The storage location on the cloud
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this volume was last updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of user who last modified the volume.
        """
    @property
    def volume_id(self, /) -> str:
        """
        The unique identifier of the volume
        """
    @property
    def volume_type(self, /) -> VolumeType | None:
        """
        The type of the volume.

        An external volume is located in the specified external location.
        A managed volume is located in the default location which is specified
        by the parent schema, or the parent catalog, or the Metastore.
        """

@final
class VolumeClient:
    """
    The requests on one `Volume`.
    """
    def delete(self, /) -> None: ...
    def get(self, /, include_browse: bool | None = None) -> Volume:
        """
        Args:
            include_browse: Whether to include volumes in the response for which the principal can only
                            access selective metadata for

        Returns:
            The requested resource
        """
    def update(self, /, new_name: str | None = None, comment: str | None = None, owner: str | None = None) -> Volume:
        """
        Args:
            new_name: New name for the volume.
            comment: The comment attached to the volume
            owner: The identifier of the user who owns the volume

        Returns:
            The requested resource
        """
