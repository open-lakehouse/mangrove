"""Unity Catalog Python client.

The API lives in the compiled extension :mod:`unitycatalog_client._client`, which trestle
generates: the root client and exceptions in the module itself, and the model classes and
resource clients of each API in a submodule (``unitycatalog_client._client.catalogs``, …).
The enums are the ``enum.StrEnum`` classes of :mod:`unitycatalog_client._enums`. This package
re-exports all of them, so ``from unitycatalog_client import Catalog`` works. The stubs under
``_client/`` come from PyO3's introspection (``just codegen generate-py-stubs``).

Exceptions: a client error raises ``UnityCatalogError`` or the subclass of its HTTP status —
``BadRequestError`` (400), ``UnauthenticatedError`` (401), ``PermissionDeniedError`` (403),
``NotFoundError`` (404), ``ConflictError`` (409), ``RateLimitError`` (429), ``ServerError``
(5xx). Each carries ``message``, ``code``, ``status`` and ``details``.

Optional integrations live in submodules and may require extras:

* :mod:`unitycatalog_client.obstore` — install with
  ``pip install unitycatalog-client[obstore]`` to build native
  `obstore <https://developmentseed.org/obstore/>`_ stores backed by
  Unity Catalog credential vending.
"""

from . import _client as _client
from ._client import (
    BadRequestError as BadRequestError,
    ConflictError as ConflictError,
    NotFoundError as NotFoundError,
    PermissionDeniedError as PermissionDeniedError,
    RateLimitError as RateLimitError,
    ServerError as ServerError,
    TemporaryCredentialClient as TemporaryCredentialClient,
    UnauthenticatedError as UnauthenticatedError,
    UnityCatalogClient as UnityCatalogClient,
    UnityCatalogError as UnityCatalogError,
)
from ._client.agent_skills import *  # noqa: F403
from ._client.agents import *  # noqa: F403
from ._client.catalogs import *  # noqa: F403
from ._client.credentials import *  # noqa: F403
from ._client.external_locations import *  # noqa: F403
from ._client.functions import *  # noqa: F403
from ._client.model_versions import *  # noqa: F403
from ._client.policies import *  # noqa: F403
from ._client.providers import *  # noqa: F403
from ._client.recipients import *  # noqa: F403
from ._client.registered_models import *  # noqa: F403
from ._client.schemas import *  # noqa: F403
from ._client.shares import *  # noqa: F403
from ._client.staging_tables import *  # noqa: F403
from ._client.tables import *  # noqa: F403
from ._client.tags import *  # noqa: F403
from ._client.temporary_credentials import *  # noqa: F403
from ._client.volumes import *  # noqa: F403
from ._enums import *  # noqa: F403

# ``parse_uc_url`` is intentionally not re-exported at the package root: it is an
# internal URL parser shared with the obstore dispatchers. Import it from
# :mod:`unitycatalog_client._client` if you need it directly.
