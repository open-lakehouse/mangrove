"""Unity Catalog Python client.

The public API lives in the compiled extension :mod:`unitycatalog_client._client`;
this package re-exports each symbol explicitly (using the PEP 484
``from X import Foo as Foo`` form) so static type checkers see a clean,
fully-typed public surface. The :file:`_client.pyi` stub describes the
compiled module — codegen emits the proto-derived bulk, and the
``just generate-code`` recipe appends the hand-written supplement
(``python/client/_client_supplement.pyi``) covering the exception
classes, ``parse_uc_url``, and the ergonomic
``TemporaryCredentialClient.temporary_*_credential`` methods.

The list of re-exports below mirrors the symbols actually registered
on the ``_client`` PyO3 module (see ``python/client/src/lib.rs`` plus
the exception/free-function registrations in ``error.rs`` /
``reference.rs``). When adding a new ``m.add_class::<...>()`` or
``create_exception!``, add the matching ``from ._client import Foo as
Foo`` line below.

Optional integrations live in submodules and may require extras:

* :mod:`unitycatalog_client.obstore` — install with
  ``pip install unitycatalog-client[obstore]`` to build native
  `obstore <https://developmentseed.org/obstore/>`_ stores backed by
  Unity Catalog credential vending.
"""

from . import _client as _client
from ._client import (
    Action as Action,
    AlreadyExistsError as AlreadyExistsError,
    AzureManagedIdentity as AzureManagedIdentity,
    AzureServicePrincipal as AzureServicePrincipal,
    AzureStorageKey as AzureStorageKey,
    Catalog as Catalog,
    CatalogClient as CatalogClient,
    CatalogType as CatalogType,
    Column as Column,
    ColumnTypeName as ColumnTypeName,
    Credential as Credential,
    CredentialClient as CredentialClient,
    DataObject as DataObject,
    DataObjectType as DataObjectType,
    DataObjectUpdate as DataObjectUpdate,
    DataSourceFormat as DataSourceFormat,
    ExternalLocation as ExternalLocation,
    ExternalLocationClient as ExternalLocationClient,
    GenericError as GenericError,
    HistoryStatus as HistoryStatus,
    InternalServerError as InternalServerError,
    InvalidParameterError as InvalidParameterError,
    ModelVersion as ModelVersion,
    ModelVersionStatus as ModelVersionStatus,
    NotFoundError as NotFoundError,
    PermissionDeniedError as PermissionDeniedError,
    Purpose as Purpose,
    Recipient as Recipient,
    RecipientClient as RecipientClient,
    RegisteredModel as RegisteredModel,
    RequestLimitError as RequestLimitError,
    Schema as Schema,
    SchemaClient as SchemaClient,
    ServiceUnavailableError as ServiceUnavailableError,
    Share as Share,
    ShareClient as ShareClient,
    Table as Table,
    TableClient as TableClient,
    TableType as TableType,
    TagPolicy as TagPolicy,
    TagPolicyClient as TagPolicyClient,
    TemporaryCredential as TemporaryCredential,
    TemporaryCredentialClient as TemporaryCredentialClient,
    UnauthenticatedError as UnauthenticatedError,
    UnityCatalogClient as UnityCatalogClient,
    UnityCatalogError as UnityCatalogError,
    Value as Value,
    Volume as Volume,
    VolumeClient as VolumeClient,
    VolumeType as VolumeType,
)

# ``parse_uc_url`` is intentionally not re-exported at the package
# root: it is an internal URL parser shared with the obstore
# dispatchers. Import it from :mod:`unitycatalog_client._client` if
# you need it directly.
#
# No ``__all__``: every public name above uses the PEP 484
# ``import Foo as Foo`` form, which is sufficient for both static type
# checkers and for ``from unitycatalog_client import Foo`` at runtime.
