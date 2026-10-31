"""
The classes of the `registered_models` API.
"""

from typing import final

__all__ = ["ListRegisteredModelsResponse", "RegisteredModel", "RegisteredModelClient"]

@final
class ListRegisteredModelsResponse:
    """
    List registered models response.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def next_page_token(self, /) -> str | None:
        """
        The next_page_token value to include in the next List request.
        """
    @property
    def registered_models(self, /) -> list[RegisteredModel]:
        """
        The registered models returned.
        """

@final
class RegisteredModel:
    """
    A registered model within the three-level namespace (catalog.schema.model).

    Registered models group a collection of model versions. Individual versions
    carry their own artifact storage location and are created, finalized, and
    governed independently.
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
        Time at which this registered model was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of registered model creator.
        """
    @property
    def full_name(self, /) -> str:
        """
        The three-level (fully qualified) name of the registered model.

        Format: catalog_name.schema_name.model_name
        """
    @property
    def id(self, /) -> str | None:
        """
        Unique identifier for the registered model.
        """
    @property
    def metastore_id(self, /) -> str | None:
        """
        The unique identifier of the metastore.
        """
    @property
    def name(self, /) -> str:
        """
        Name of registered model, relative to parent schema.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of current owner of the registered model.
        """
    @property
    def schema_name(self, /) -> str:
        """
        Name of parent schema.
        """
    @property
    def storage_location(self, /) -> str | None:
        """
        The storage location under which model version data files are stored.
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this registered model was last updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of user who last modified the registered model.
        """

@final
class RegisteredModelClient:
    """
    The requests on one `RegisteredModel`.
    """
    def delete(self, /, force: bool | None = None) -> None:
        """
        Delete a registered model

        Deletes the registered model that matches the supplied name. For the deletion to succeed, the caller
        must be the owner of the registered model.

        Args:
            force: Force deletion even if the registered model still has model versions.
        """
    def get(self, /, include_browse: bool | None = None) -> RegisteredModel:
        """
        Get a registered model

        Gets a registered model from within a parent catalog and schema. For the fetch to succeed, the
        caller must be a metastore admin, the owner of the registered model, or have a privilege on the
        registered model.

        Args:
            include_browse: Whether to include registered models in the response for which the principal can
                            only access selective metadata for.

        Returns:
            A registered model within the three-level namespace (catalog.schema.model). Registered
            models
            group a collection of model versions. Individual versions carry their own artifact storage
            location and are created, finalized, and governed independently.
        """
    def update(self, /, new_name: str | None = None, comment: str | None = None, owner: str | None = None) -> RegisteredModel:
        """
        Update a registered model

        Updates the registered model that matches the supplied name.

        Args:
            new_name: New name for the registered model.
            comment: User-provided free-form text description.
            owner: Username of new owner of the registered model.

        Returns:
            A registered model within the three-level namespace (catalog.schema.model). Registered
            models
            group a collection of model versions. Individual versions carry their own artifact storage
            location and are created, finalized, and governed independently.
        """
