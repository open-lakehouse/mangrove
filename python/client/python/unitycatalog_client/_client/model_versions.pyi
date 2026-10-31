"""
The classes of the `model_versions` API.
"""

from typing import final
from unitycatalog_client._enums import ModelVersionStatus

__all__ = ["ListModelVersionsResponse", "ModelVersion"]

@final
class ListModelVersionsResponse:
    """
    List model versions response.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def model_versions(self, /) -> list[ModelVersion]:
        """
        The model versions returned.
        """
    @property
    def next_page_token(self, /) -> str | None:
        """
        The next_page_token value to include in the next List request.
        """

@final
class ModelVersion:
    """
    A model version under a registered model.

    A version is identified by its parent model's three-level name plus an
    integer `version` number assigned by the server. It carries its own artifact
    storage location and moves through the PENDING_REGISTRATION -> READY lifecycle
    via the finalize operation.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def browse_only(self, /) -> bool | None:
        """
        Indicates whether the principal is limited to retrieving metadata for the
        associated object through the BROWSE privilege when include_browse is
        enabled in the request.
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
        Time at which this model version was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of model version creator.
        """
    @property
    def id(self, /) -> str | None:
        """
        Unique identifier for the model version.
        """
    @property
    def metastore_id(self, /) -> str | None:
        """
        The unique identifier of the metastore.
        """
    @property
    def model_name(self, /) -> str:
        """
        Name of the parent registered model, relative to parent schema.
        """
    @property
    def run_id(self, /) -> str | None:
        """
        The run id used by the ML package that generated this model.
        """
    @property
    def schema_name(self, /) -> str:
        """
        Name of parent schema.
        """
    @property
    def source(self, /) -> str:
        """
        URI indicating the location of the source artifacts used to create the model
        version.
        """
    @property
    def status(self, /) -> ModelVersionStatus | None:
        """
        Current status of the model version.
        """
    @property
    def storage_location(self, /) -> str | None:
        """
        The storage location under which the model version's artifacts are stored.
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this model version was last updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of user who last modified the model version.
        """
    @property
    def version(self, /) -> int:
        """
        Integer model version number, used to reference the model version in API
        requests. Assigned by the server, monotonically increasing per model.
        """
