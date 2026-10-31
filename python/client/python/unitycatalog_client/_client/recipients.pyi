"""
The classes of the `recipients` API.
"""

from typing import SupportsIndex, final
from unitycatalog_client._enums import AuthenticationType

__all__ = ["ListRecipientsResponse", "Recipient", "RecipientClient", "RecipientToken", "SecurablePropertiesKvPairs"]

@final
class ListRecipientsResponse:
    """
    Response to list recipients.
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
    def recipients(self, /) -> list[Recipient]:
        """
        List of recipients.
        """

@final
class Recipient:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def authentication_type(self, /) -> AuthenticationType | None:
        """
        The delta sharing authentication type.
        """
    @property
    def comment(self, /) -> str | None:
        """
        Description about the recipient.
        """
    @property
    def created_at(self, /) -> int | None:
        """
        Time at which this recipient was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of recipient creator.
        """
    @property
    def id(self, /) -> str | None:
        """
        Unique ID of the recipient.
        """
    @property
    def name(self, /) -> str:
        """
        The name of the recipient.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of the recipient owner.
        """
    @property
    def properties_kvpairs(self, /) -> SecurablePropertiesKvPairs | None:
        """
        Recipient properties as map of string key-value pairs.
        """
    @property
    def tokens(self, /) -> list[RecipientToken]:
        """
        This field is only present when the authentication_type is TOKEN.
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this recipient was updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of recipient updater.
        """

@final
class RecipientClient:
    """
    The requests on one `Recipient`.
    """
    def delete(self, /) -> None:
        """
        Delete a recipient.
        """
    def get(self, /) -> Recipient:
        """
        Get a recipient by name.

        Returns:
            The requested resource
        """
    def rotate_recipient_token(self, /, existing_token_expire_in_seconds: SupportsIndex | None = None) -> Recipient:
        """
        Rotate a recipient's token: issue a new token (and activation link) and retire the existing one.

        Args:
            existing_token_expire_in_seconds: Seconds until the existing token expires. It can only shorten
                                              the existing token's lifetime, never extend it; 0 expires
                                              it immediately.

        Returns:
            The requested resource
        """
    def update(self, /, new_name: str | None = None, owner: str | None = None, comment: str | None = None, properties_kvpairs: SecurablePropertiesKvPairs | None = None, expiration_time: SupportsIndex | None = None) -> Recipient:
        """
        Update a recipient.

        Args:
            new_name: New name for the recipient
            owner: Username of the recipient owner.
            comment: Description about the recipient.
            properties_kvpairs: Recipient properties as map of string key-value pairs. When provided
                                in update request, the specified properties will override the existing
                                properties. To add and remove properties, one would need to perform a read-
                                modify-write.
            expiration_time: Expiration timestamp of the token, in epoch milliseconds.

        Returns:
            The requested resource
        """

@final
class RecipientToken:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def activation_url(self, /) -> str:
        """
        Full activation URL to retrieve the access token. It will be empty if the token is already retrieved.
        """
    @property
    def created_at(self, /) -> int:
        """
        Time at which this recipient token was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str:
        """
        Username of recipient token creator.
        """
    @property
    def expiration_time(self, /) -> int:
        """
        Expiration timestamp of the token in epoch milliseconds.
        """
    @property
    def id(self, /) -> str:
        """
        Unique ID of the recipient token.
        """
    @property
    def updated_at(self, /) -> int:
        """
        Time at which this recipient token was updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str:
        """
        Username of recipient token updater.
        """

@final
class SecurablePropertiesKvPairs:
    """
    An object with the properties of a securable as key-value pairs.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, properties: dict[str, str] | None = None) -> SecurablePropertiesKvPairs: ...
    def __repr__(self, /) -> str: ...
    @property
    def properties(self, /) -> dict[str, str]:
        """
        A map of key-value properties attached to the securable.
        """
