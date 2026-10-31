"""
The classes of the `credentials` API.
"""

from typing import final
from unitycatalog_client._enums import Purpose

__all__ = ["AwsIamRoleConfig", "AzureManagedIdentity", "AzureServicePrincipal", "AzureStorageKey", "Credential", "CredentialClient", "DatabricksGcpServiceAccount", "ListCredentialsResponse"]

@final
class AwsIamRoleConfig:
    """
    The AWS IAM role configuration used server-side to call STS AssumeRole.

    This is an internal configuration type, not exposed in the public API response.
    It stores the static credentials (or ambient credential fallback) used to
    authorize the STS:AssumeRole call when vending temporary credentials.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, role_arn: str | None = None, region: str | None = None, access_key_id: str | None = None, secret_access_key: str | None = None, session_token: str | None = None) -> AwsIamRoleConfig: ...
    def __repr__(self, /) -> str: ...
    @property
    def access_key_id(self, /) -> str | None:
        """
        AWS access key ID used to authorize the STS AssumeRole call.

        When set together with secret_access_key, these static credentials are used
        as the base identity to call STS:AssumeRole. When omitted, the server falls
        back to ambient credentials (instance profile, ECS task role, WebIdentity, etc.).
        """
    @property
    def region(self, /) -> str | None:
        """
        The AWS region where STS calls are made. Defaults to "us-east-1" if absent.
        """
    @property
    def role_arn(self, /) -> str:
        """
        The Amazon Resource Name (ARN) of the AWS IAM role used to vend temporary credentials.
        """
    @property
    def secret_access_key(self, /) -> str | None:
        """
        AWS secret access key paired with access_key_id.
        """
    @property
    def session_token(self, /) -> str | None:
        """
        Optional AWS session token for temporary base credentials (e.g. when the
        caller itself holds STS-vended credentials as the base identity).
        """

@final
class AzureManagedIdentity:
    """
    The Azure managed identity configuration.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, access_connector_id: str | None = None, credential_id: str | None = None, managed_identity_id: str | None = None) -> AzureManagedIdentity: ...
    def __repr__(self, /) -> str: ...
    @property
    def access_connector_id(self, /) -> str:
        """
        The Azure resource ID of the Azure Databricks Access Connector.
        Use the format /subscriptions/{guid}/resourceGroups/{rg-name}/providers/
        Microsoft.Databricks/accessConnectors/{connector-name}.
        """
    @property
    def credential_id(self, /) -> str | None:
        """
        The Databricks internal ID that represents this managed identity.
        """
    @property
    def managed_identity_id(self, /) -> str | None:
        """
        The Azure resource ID of the managed identity.
        Use the format /subscriptions/{guid}/resourceGroups/{rg-name}/providers/
        Microsoft.ManagedIdentity/userAssignedIdentities/{identity-name}.
        This is only available for user-assigned identities. For system-assigned
        identities, the access_connector_id is used to identify the identity.
        If this field is not provided, then the system-assigned identity of the
        Access Connector is used.
        """

@final
class AzureServicePrincipal:
    """
    The Azure service principal configuration.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, directory_id: str | None = None, application_id: str | None = None, client_secret: str | None = None, federated_token_file: str | None = None) -> AzureServicePrincipal: ...
    def __repr__(self, /) -> str: ...
    @property
    def application_id(self, /) -> str:
        """
        The application ID of the application registration within the referenced AAD tenant.
        """
    @property
    def client_secret(self, /) -> str | None:
        """
        The client secret generated for the above app ID in AAD.
        """
    @property
    def directory_id(self, /) -> str:
        """
        The directory ID corresponding to the Azure Active Directory (AAD) tenant of the application.
        """
    @property
    def federated_token_file(self, /) -> str | None:
        """
        Location of the file containing a federated token, for workload identity federation.
        """

@final
class AzureStorageKey:
    """
    The Azure storage key configuration.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, account_name: str | None = None, account_key: str | None = None) -> AzureStorageKey: ...
    def __repr__(self, /) -> str: ...
    @property
    def account_key(self, /) -> str:
        """
        The account key of the storage account.
        """
    @property
    def account_name(self, /) -> str:
        """
        The name of the storage account.
        """

@final
class Credential:
    """
    A credential used to access external data sources or services.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def aws_iam_role(self, /) -> AwsIamRoleConfig | None:
        """
        The AWS IAM role configuration.
        """
    @property
    def azure_managed_identity(self, /) -> AzureManagedIdentity | None:
        """
        The Azure managed identity configuration.
        """
    @property
    def azure_service_principal(self, /) -> AzureServicePrincipal | None:
        """
        The Azure service principal configuration.

        Sealed into the object's inline sensitive blob (envelope-encrypted, redacted
        from ordinary reads) rather than stored in the searchable properties.
        """
    @property
    def azure_storage_key(self, /) -> AzureStorageKey | None:
        """
        The Azure storage key configuration.
        """
    @property
    def comment(self, /) -> str | None:
        """
        Comment associated with the credential.
        """
    @property
    def created_at(self, /) -> int | None:
        """
        Time at which this credential was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of credential creator.
        """
    @property
    def databricks_gcp_service_account(self, /) -> DatabricksGcpServiceAccount | None:
        """
        The Databricks managed GCP service account configuration.
        """
    @property
    def full_name(self, /) -> str:
        """
        The full name of the credential.
        """
    @property
    def id(self, /) -> str | None:
        """
        The unique identifier of the credential.
        """
    @property
    def name(self, /) -> str:
        """
        The credential name.

        The name must be unique among storage and service credentials within the metastore.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of current owner of credential.
        """
    @property
    def purpose(self, /) -> Purpose | None:
        """
        Indicates the purpose of the credential.
        """
    @property
    def read_only(self, /) -> bool:
        """
        Whether the credential is usable only for read operations.

        Only applicable when purpose is STORAGE.
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this credential was last modified, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of user who last modified the credential.
        """
    @property
    def used_for_managed_storage(self, /) -> bool:
        """
        Whether this credential is the current metastore's root storage credential.

        Only applicable when purpose is STORAGE.
        """

@final
class CredentialClient:
    """
    The requests on one `Credential`.
    """
    def delete(self, /) -> None: ...
    def get(self, /) -> Credential:
        """
        Returns:
            A credential used to access external data sources or services.
        """
    def update(self, /, new_name: str | None = None, comment: str | None = None, read_only: bool | None = None, owner: str | None = None, skip_validation: bool | None = None, force: bool | None = None, azure_service_principal: AzureServicePrincipal | None = None, azure_managed_identity: AzureManagedIdentity | None = None, azure_storage_key: AzureStorageKey | None = None, aws_iam_role: AwsIamRoleConfig | None = None, databricks_gcp_service_account: DatabricksGcpServiceAccount | None = None) -> Credential:
        """
        Args:
            new_name: New name of the credential.
            comment: Comment associated with the credential.
            read_only: Whether the credential is usable only for read operations. Only applicable when
                       purpose is STORAGE.
            owner: Username of current owner of credential.
            skip_validation: Supply true to this argument to skip validation of the updated credential.
                             Defaults to false.
            force: Force an update even if there are dependent services (when purpose is SERVICE) or
                   dependent external locations and external tables (when purpose is STORAGE).
            azure_service_principal: The Azure service principal configuration.
            azure_managed_identity: The Azure managed identity configuration.
            azure_storage_key: The Azure storage key configuration.
            aws_iam_role: The AWS IAM role configuration.
            databricks_gcp_service_account: The Databricks managed GCP service account configuration.

        Returns:
            A credential used to access external data sources or services.
        """

@final
class DatabricksGcpServiceAccount:
    """
    The Databricks managed GCP service account configuration.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, credential_id: str | None = None, email: str | None = None, private_key_id: str | None = None) -> DatabricksGcpServiceAccount: ...
    def __repr__(self, /) -> str: ...
    @property
    def credential_id(self, /) -> str | None:
        """
        The Databricks internal ID that represents this managed identity.
        """
    @property
    def email(self, /) -> str | None:
        """
        The email of the service account.
        """
    @property
    def private_key_id(self, /) -> str | None:
        """
        The ID that represents the private key for this Service Account.
        """

@final
class ListCredentialsResponse:
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def credentials(self, /) -> list[Credential]:
        """
        The credentials returned.
        """
    @property
    def next_page_token(self, /) -> str | None:
        """
        The next_page_token value to include in the next List request.
        """
