# Hand-written supplement appended to `_client.pyi` after codegen.
#
# Declares the PyO3 symbols hand-registered in
# `python/client/src/{error,reference}.rs` that the protobuf-driven
# trestle codegen does not see. Edit this file (not `_client.pyi`) when
# adding new hand-written `#[pyfunction]`s, `#[pyclass]`es, or
# `create_exception!` calls to the `_client` PyO3 module.
#
# This file is appended verbatim to `_client.pyi` by `just generate-code`.
# It is also a valid stub on its own (the import below is harmless when
# appended to `_client.pyi`, which already imports the same names).

from typing import Literal

# Exceptions (declared in python/client/src/error.rs).

class UnityCatalogError(Exception):
    """Base class for all Python-facing Unity Catalog exceptions."""

    ...

class GenericError(UnityCatalogError):
    """Wraps a generic server-side error."""

    ...

class NotFoundError(UnityCatalogError):
    """Raised when a requested resource does not exist (`RESOURCE_NOT_FOUND`)."""

    ...

class AlreadyExistsError(UnityCatalogError):
    """Raised when a resource already exists (`RESOURCE_ALREADY_EXISTS`)."""

    ...

class PermissionDeniedError(UnityCatalogError):
    """Raised when the caller lacks permission (`PERMISSION_DENIED`)."""

    ...

class UnauthenticatedError(UnityCatalogError):
    """Raised when the request is not authenticated (`UNAUTHENTICATED`)."""

    ...

class InvalidParameterError(UnityCatalogError):
    """Raised when a request parameter is invalid (`INVALID_PARAMETER_VALUE`)."""

    ...

class RequestLimitError(UnityCatalogError):
    """Raised when the request rate limit is exceeded (`REQUEST_LIMIT_EXCEEDED`)."""

    ...

class InternalServerError(UnityCatalogError):
    """Raised when the server encounters an internal error (`INTERNAL_ERROR`)."""

    ...

class ServiceUnavailableError(UnityCatalogError):
    """Raised when the service is temporarily unavailable (`TEMPORARILY_UNAVAILABLE`)."""

    ...

# Free functions (declared in python/client/src/reference.rs).

def parse_uc_url(
    url: str,
    /,
) -> tuple[Literal["volume", "table", "path"], dict[str, str]]:
    """Parse a Unity Catalog URL into ``(kind, payload)``.

    Delegates to ``unitycatalog_common::UCReference::parse``. See that
    type's documentation for the supported grammar (``uc://`` URLs,
    raw cloud URLs, ``vol+dbfs:`` aliases, case-insensitive kind
    segments, percent-decoded path components).

    ``payload`` carries the variant-specific fields:

    * ``"volume"`` → ``{catalog, schema, volume, path}``
    * ``"table"``  → ``{catalog, schema, table}``
    * ``"path"``   → ``{url}``
    """
    ...

# Hand-written `#[pymethods]` for `TemporaryCredentialClient`
# (declared in python/client/src/client.rs). These wrap the underlying
# `Generate*Credentials` RPCs with name → UUID resolution. The trestle
# codegen does not see them, so they are declared here as a complete
# replacement for the empty `class TemporaryCredentialClient: ...`
# placeholder emitted by the codegen — the `just generate-code` recipe
# strips that placeholder line before appending this file.

# `TemporaryCredential` is declared earlier in the concatenated
# `_client.pyi` (codegen-emitted from the proto schema). Use a string
# forward reference so this file also type-checks in isolation.

class TemporaryCredentialClient:
    def __init__(self, base_url: str, token: str | None = None) -> None: ...
    def temporary_table_credential(
        self,
        table: str,
        operation: Literal["read", "read_write"],
    ) -> tuple[TemporaryCredential, str]:
        """Vend a temporary credential for a Unity Catalog table.

        ``table`` is the three-level ``catalog.schema.table`` name.
        The returned tuple holds the credential and the resolved table
        UUID (as a string).
        """
        ...

    def temporary_volume_credential(
        self,
        volume: str,
        operation: Literal["read", "read_write", "write"],
    ) -> tuple[TemporaryCredential, str]:
        """Vend a temporary credential for a Unity Catalog volume.

        ``volume`` is the three-level ``catalog.schema.volume`` name.
        Server support requires the metastore's
        ``external_access_enabled`` flag and the caller's
        ``EXTERNAL_USE_SCHEMA`` privilege.
        """
        ...

    def temporary_path_credential(
        self,
        path: str,
        operation: Literal["read", "read_write", "create_table"],
        dry_run: bool | None = None,
    ) -> tuple[TemporaryCredential, str]:
        """Vend a temporary credential for an arbitrary cloud URL."""
        ...

    def temporary_model_version_credential(
        self,
        full_name: str,
        version: int,
        operation: Literal["read", "read_write", "write"],
    ) -> TemporaryCredential:
        """Vend a temporary credential for a Unity Catalog model version.

        ``full_name`` is the three-level ``catalog.schema.model`` name of the
        parent registered model and ``version`` its integer version number.
        Server support requires the metastore's ``external_access_enabled``
        flag and the caller's ``EXTERNAL_USE_SCHEMA`` privilege.
        """
        ...
