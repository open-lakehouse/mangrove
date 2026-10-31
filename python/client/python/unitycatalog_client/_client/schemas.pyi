"""
The classes of the `schemas` API.
"""

from typing import final

__all__ = ["ListSchemasResponse", "Schema", "SchemaClient"]

@final
class ListSchemasResponse:
    """
    List Schemas response.
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
    def schemas(self, /) -> list[Schema]:
        """
        The schemas returned.
        """

@final
class Schema:
    """
    A schema is a namespace within a catalog that contains tables.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def browse_only(self, /) -> bool | None:
        """
        Indicates whether the principal is limited to retrieving metadata
        for the associated object through the BROWSE privilege when include_browse
        is enabled in the request.
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
        Time at which this schema was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of schema creator.
        """
    @property
    def full_name(self, /) -> str:
        """
        Full name of schema, in form of catalog_name.schema_name.
        """
    @property
    def name(self, /) -> str:
        """
        Name of schema, relative to parent catalog.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of current owner of schema.
        """
    @property
    def properties(self, /) -> dict[str, str]:
        """
        A map of key-value properties attached to the securable.
        """
    @property
    def schema_id(self, /) -> str | None:
        """
        Unique identifier for the schema.
        """
    @property
    def storage_location(self, /) -> str | None:
        """
        Storage location URL (full path) for managed storage of the schema.

        A unique path under `storage_root`. Absent when the schema has no
        `storage_root`, in which case managed securables fall back to the parent
        catalog's storage location.
        Example: `s3://bucket/ucroot/__unitystorage/schemas/{schema_id}`.
        """
    @property
    def storage_root(self, /) -> str | None:
        """
        Storage root URL for managed storage location of the schema.

        Can be set when creating a schema. When set, managed tables/volumes created
        in this schema are rooted here; otherwise the parent catalog's storage
        location is used. Example: `s3://bucket/ucroot`.
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this schema was last updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of user who last modified schema.
        """

@final
class SchemaClient:
    """
    The requests on one `Schema`.
    """
    def delete(self, /, force: bool | None = None) -> None:
        """
        Deletes the specified schema from the parent catalog. The caller must be the owner of the schema or
        an owner of the parent catalog.

        Args:
            force: Force deletion even if the schema is not empty.
        """
    def get(self, /, include_browse: bool | None = None) -> Schema:
        """
        Gets the specified schema within the metastore. The caller must be a metastore admin, the owner of
        the schema, or a user that has the USE_SCHEMA privilege on the schema.

        Args:
            include_browse: Whether to include schemas in the response for which the principal can only
                            access selective metadata for

        Returns:
            A schema is a namespace within a catalog that contains tables.
        """
    def update(self, /, comment: str | None = None, properties: dict[str, str] | None = None, new_name: str | None = None) -> Schema:
        """
        Updates a schema for a catalog. The caller must be the owner of the schema or a metastore admin.
        If the caller is a metastore admin, only the owner field can be changed in the update. If the name
        field must be updated, the caller must be a metastore admin or have the CREATE_SCHEMA privilege on
        the parent catalog.

        Args:
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.
            new_name: New name for the schema.

        Returns:
            A schema is a namespace within a catalog that contains tables.
        """
