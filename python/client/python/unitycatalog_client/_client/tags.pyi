"""
The classes of the `tags` API.
"""

from _typeshed import SupportsGetItem
from typing import final
from unitycatalog_client._enums import TagAssignmentSourceType

__all__ = ["EntityTagAssignment", "ListEntityTagAssignmentsResponse", "ListTagPoliciesResponse", "TagPolicy", "TagPolicyClient", "Value"]

@final
class EntityTagAssignment:
    """
    The assignment of a tag to a Unity Catalog entity.

    Unlike a TagPolicy (a governed-tag *definition*), an assignment is the application
    of a tag to a specific securable. It has no identifier of its own — its identity is
    the composite of (entity_type, entity_name, tag_key). It is intentionally NOT a
    `google.api.resource`: assignments are stored as associations between the entity and
    its tag, not as standalone objects.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def entity_name(self, /) -> str:
        """
        The fully qualified name of the entity to which the tag is assigned.
        """
    @property
    def entity_type(self, /) -> str:
        """
        The type of the entity to which the tag is assigned.

        Supported values: catalogs, schemas, tables, columns, volumes.
        """
    @property
    def source_type(self, /) -> TagAssignmentSourceType | None:
        """
        The source of the tag assignment, e.g. user-assigned or system-assigned.
        """
    @property
    def tag_key(self, /) -> str:
        """
        The key of the tag.
        """
    @property
    def tag_value(self, /) -> str | None:
        """
        The value of the tag.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        The user or principal who last updated the tag assignment.
        """

@final
class ListEntityTagAssignmentsResponse:
    """
    List entity tag assignments response.
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
    def tag_assignments(self, /) -> list[EntityTagAssignment]:
        """
        The tag assignments returned.
        """

@final
class ListTagPoliciesResponse:
    """
    List tag policies response.
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
    def tag_policies(self, /) -> list[TagPolicy]:
        """
        The tag policies returned.
        """

@final
class TagPolicy:
    """
    A governed tag definition (tag policy).

    A tag policy defines a tag key together with the rules that govern how it can
    be used, including the optional set of allowed values. Assigning a governed
    tag to an entity is done through the Entity Tag Assignments API.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def created_at(self, /) -> int | None:
        """
        Time at which this tag policy was created, in epoch milliseconds.
        """
    @property
    def description(self, /) -> str | None:
        """
        User-provided free-form text description of the tag policy.
        """
    @property
    def id(self, /) -> str | None:
        """
        Unique identifier for the tag policy.
        """
    @property
    def tag_key(self, /) -> str:
        """
        The key of the governed tag.
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this tag policy was last updated, in epoch milliseconds.
        """
    @property
    def values(self, /) -> list[Value]:
        """
        The set of allowed values for the governed tag.

        When empty, the governed tag does not restrict the values that may be assigned.
        The list is unordered.
        """

@final
class TagPolicyClient:
    """
    The requests on one `TagPolicy`.
    """
    def delete(self, /) -> None:
        """
        Delete a tag policy

        Deletes the governed tag definition that matches the supplied tag key.
        """
    def get(self, /) -> TagPolicy:
        """
        Get a tag policy

        Gets the governed tag definition for the specified tag key.

        Returns:
            A governed tag definition (tag policy). A tag policy defines a tag key together with the
            rules
            that govern how it can be used, including the optional set of allowed values. Assigning a
            governed tag to an entity is done through the Entity Tag Assignments API.
        """
    def update(self, /, description: str | None = None, values: SupportsGetItem[int, Value] | None = None, update_mask: str | None = None) -> TagPolicy:
        """
        Update a tag policy

        Updates the governed tag definition that matches the supplied tag key.

        Args:
            description: User-provided free-form text description of the tag policy.
            values: The set of allowed values for the governed tag. The list is unordered.
            update_mask: The fields to update, as a comma-separated list of field names. If empty or "*",
                         every field set in the request is updated. Otherwise only the listed fields are
                         updated, and a listed field that is unset in the request is cleared.

        Returns:
            A governed tag definition (tag policy). A tag policy defines a tag key together with the
            rules
            that govern how it can be used, including the optional set of allowed values. Assigning a
            governed tag to an entity is done through the Entity Tag Assignments API.
        """

@final
class Value:
    """
    An allowed value for a governed tag.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __new__(cls, /, name: str | None = None) -> Value: ...
    def __repr__(self, /) -> str: ...
    @property
    def name(self, /) -> str:
        """
        The name of the allowed value.
        """
