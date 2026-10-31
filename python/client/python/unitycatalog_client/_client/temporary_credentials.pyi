"""
The classes of the `temporary_credentials` API.
"""

from typing import final

__all__ = ["AwsTemporaryCredentials", "AzureAad", "AzureUserDelegationSas", "GcpOauthToken", "R2TemporaryCredentials", "TemporaryCredential"]

@final
class AwsTemporaryCredentials:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def access_key_id(self, /) -> str:
        """
        The access key ID that identifies the temporary credentials.
        """
    @property
    def access_point(self, /) -> str:
        """
        The Amazon Resource Name (ARN) of the S3 access point for temporary credentials related the external location.
        """
    @property
    def secret_access_key(self, /) -> str:
        """
        The secret access key that can be used to sign AWS API requests.
        """
    @property
    def session_token(self, /) -> str:
        """
        The token that users must pass to AWS API to use the temporary credentials.
        """

@final
class AzureAad:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def aad_token(self, /) -> str:
        """
        Opaque token that contains claims that you can use in Azure Active Directory to access cloud services.
        """

@final
class AzureUserDelegationSas:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def sas_token(self, /) -> str:
        """
        The signed URI (SAS Token) used to access blob services for a given path
        """

@final
class GcpOauthToken:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def oauth_token(self, /) -> str:
        """
        The OAuth token used to access Google Cloud services.
        """

@final
class R2TemporaryCredentials:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def access_key_id(self, /) -> str:
        """
        The access key ID that identifies the temporary credentials.
        """
    @property
    def secret_access_key(self, /) -> str:
        """
        The secret access key associated with the access key.
        """
    @property
    def session_token(self, /) -> str:
        """
        The generated JWT that users must pass to use the temporary credentials.
        """

@final
class TemporaryCredential:
    """
    The response to the GenerateTemporaryTableCredentialsRequest.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def aws_temp_credentials(self, /) -> AwsTemporaryCredentials | None:
        """
        Credentials for AWS S3.
        """
    @property
    def azure_aad(self, /) -> AzureAad | None:
        """
        Credentials for Azure Active Directory.
        """
    @property
    def azure_user_delegation_sas(self, /) -> AzureUserDelegationSas | None:
        """
        Credentials for Azure Blob Storage.
        """
    @property
    def expiration_time(self, /) -> int:
        """
        Server time when the credential will expire, in epoch milliseconds.
        The API client is advised to cache the credential given this expiration time.
        """
    @property
    def gcp_oauth_token(self, /) -> GcpOauthToken | None:
        """
        Credentials for Google Cloud Storage.
        """
    @property
    def r2_temp_credentials(self, /) -> R2TemporaryCredentials | None:
        """
        Credentials for R2.
        """
    @property
    def url(self, /) -> str:
        """
        The URL of the storage path accessible by the temporary credential.
        """
