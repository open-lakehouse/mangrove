"""
The classes of the `external_locations` API.
"""

from typing import final

__all__ = ["ExternalLocation", "ExternalLocationClient", "ListExternalLocationsResponse"]

@final
class ExternalLocation:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def browse_only(self, /) -> bool | None:
        """
        Indicates whether the principal is limited to retrieving metadata
        for the associated object through the BROWSE privilege when include_browse is enabled in the request.
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
    def credential_id(self, /) -> str:
        """
        Unique ID of the location's storage credential.
        """
    @property
    def credential_name(self, /) -> str:
        """
        Name of the storage credential used with this location.
        """
    @property
    def external_location_id(self, /) -> str | None: ...
    @property
    def name(self, /) -> str:
        """
        Name of the external location.
        """
    @property
    def owner(self, /) -> str | None:
        """
        The owner of the external location.
        """
    @property
    def read_only(self, /) -> bool:
        """
        Indicates whether the external location is read-only.
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
    @property
    def url(self, /) -> str:
        """
        Path URL of the external location.
        """

@final
class ExternalLocationClient:
    """
    The requests on one `ExternalLocation`.
    """
    def delete(self, /, force: bool | None = None) -> None:
        """
        Delete an external location

        Args:
            force: Force deletion even if the external location is not empty.
        """
    def get(self, /, include_browse: bool | None = None) -> ExternalLocation:
        """
        Get an external location

        Args:
            include_browse: Whether to include external locations in the response for which the principal
                            can only access selective metadata for

        Returns:
            The requested resource
        """
    def update(self, /, url: str | None = None, credential_name: str | None = None, read_only: bool | None = None, owner: str | None = None, comment: str | None = None, new_name: str | None = None, force: bool | None = None, skip_validation: bool | None = None) -> ExternalLocation:
        """
        Update an external location

        Args:
            url: Path URL of the external location.
            credential_name: Name of the storage credential used with this location.
            read_only: Indicates whether the external location is read-only.
            owner: owner of the external location.
            comment: User-provided free-form text description.
            new_name: new name of the external location.
            force: force update of the external location.
            skip_validation: Skips validation of the storage credential associated with the external
                             location.

        Returns:
            The requested resource
        """

@final
class ListExternalLocationsResponse:
    """
    List external locations response.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def external_locations(self, /) -> list[ExternalLocation]:
        """
        The external locations returned.
        """
    @property
    def next_page_token(self, /) -> str | None:
        """
        The next_page_token value to include in the next List request.
        """
