"""
The classes of the `providers` API.
"""

from typing import final
from unitycatalog_client._enums import ProviderAuthenticationType

__all__ = ["ListProvidersResponse", "Provider", "ProviderClient"]

@final
class ListProvidersResponse:
    """
    Response to list providers.
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
    def providers(self, /) -> list[Provider]:
        """
        List of providers.
        """

@final
class Provider:
    """
    A provider represents an organization sharing data with this metastore.

    A provider is the inbound counterpart of a recipient: it is registered from a
    share activation/credential file and is used to access shares offered by an
    upstream Delta Sharing server.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def authentication_type(self, /) -> ProviderAuthenticationType | None:
        """
        The delta sharing authentication type.
        """
    @property
    def comment(self, /) -> str | None:
        """
        Description about the provider.
        """
    @property
    def created_at(self, /) -> int | None:
        """
        Time at which this provider was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of the creator of the provider.
        """
    @property
    def id(self, /) -> str | None:
        """
        Unique ID of the provider.
        """
    @property
    def name(self, /) -> str:
        """
        The name of the provider.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of the provider owner.
        """
    @property
    def properties(self, /) -> dict[str, str]:
        """
        A map of key-value properties attached to the provider.
        """
    @property
    def recipient_profile_str(self, /) -> str | None:
        """
        The recipient profile (credential file contents) used to connect to the
        sharing server. Present for TOKEN and OAUTH_CLIENT_CREDENTIALS authentication.
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this provider was updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of the provider updater.
        """

@final
class ProviderClient:
    """
    The requests on one `Provider`.
    """
    def delete(self, /) -> None:
        """
        Delete a provider.
        """
    def get(self, /) -> Provider:
        """
        Get a provider by name.

        Returns:
            A provider represents an organization sharing data with this metastore. A provider is the
            inbound counterpart of a recipient: it is registered from a share activation/ credential
            file
            and is used to access shares offered by an upstream Delta Sharing server.
        """
    def update(self, /, new_name: str | None = None, owner: str | None = None, comment: str | None = None, recipient_profile_str: str | None = None, properties: dict[str, str] | None = None) -> Provider:
        """
        Update a provider.

        Args:
            new_name: New name for the provider.
            owner: Username of the provider owner.
            comment: Description about the provider.
            recipient_profile_str: The recipient profile (credential file contents) used to connect to
                                   the sharing server. Required when authentication_type is TOKEN or
                                   OAUTH_CLIENT_CREDENTIALS.
            properties: Provider properties as map of string key-value pairs. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.

        Returns:
            A provider represents an organization sharing data with this metastore. A provider is the
            inbound counterpart of a recipient: it is registered from a share activation/ credential
            file
            and is used to access shares offered by an upstream Delta Sharing server.
        """
