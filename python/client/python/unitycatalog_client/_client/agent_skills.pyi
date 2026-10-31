"""
The classes of the `agent_skills` API.
"""

from _typeshed import SupportsGetItem
from typing import final
from unitycatalog_client._enums import AgentSkillType

__all__ = ["AgentSkill", "AgentSkillClient", "ListAgentSkillsResponse"]

@final
class AgentSkill:
    """
    An agent skill registered in Unity Catalog.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def agent_skill_id(self, /) -> str:
        """
        The unique identifier of the agent skill.
        """
    @property
    def agent_skill_type(self, /) -> AgentSkillType | None:
        """
        How the storage location is provisioned (external or managed).
        """
    @property
    def allowed_tools(self, /) -> list[str]:
        """
        The tools the skill is permitted to use, as declared in its SKILL.md.
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
        Time at which this agent skill was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of agent skill creator.
        """
    @property
    def description(self, /) -> str | None:
        """
        A human-readable description of what the skill does and when to use it.

        Mirrors the Agent Skills spec `description` frontmatter field.
        """
    @property
    def full_name(self, /) -> str:
        """
        The three-level (fully qualified) name of the agent skill.
        """
    @property
    def license(self, /) -> str | None:
        """
        SPDX license identifier or free-form license text for the skill.
        """
    @property
    def metadata(self, /) -> dict[str, str]:
        """
        Arbitrary additional metadata declared by the skill.
        """
    @property
    def metastore_id(self, /) -> str | None:
        """
        The unique identifier of the metastore.
        """
    @property
    def name(self, /) -> str:
        """
        Name of the agent skill, relative to parent schema.

        Per the Agent Skills spec a skill name is lowercase letters, numbers, and
        hyphens, but Unity Catalog object names disallow hyphens, so UC stores the
        catalog object name with underscores; the spec-shaped name lives in the
        skill's own SKILL.md.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of current owner of the agent skill.
        """
    @property
    def schema_name(self, /) -> str:
        """
        Name of parent schema.
        """
    @property
    def storage_location(self, /) -> str:
        """
        The storage location of the skill's directory (the root that holds
        `SKILL.md`).
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this agent skill was last updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of user who last modified the agent skill.
        """

@final
class AgentSkillClient:
    """
    The requests on one `AgentSkill`.
    """
    def delete(self, /) -> None: ...
    def get(self, /, include_browse: bool | None = None) -> AgentSkill:
        """
        Args:
            include_browse: Whether to include agent skills in the response for which the principal can only
                            access selective metadata for.

        Returns:
            An agent skill registered in Unity Catalog.
        """
    def update(self, /, new_name: str | None = None, description: str | None = None, allowed_tools: SupportsGetItem[int, str] | None = None, comment: str | None = None, owner: str | None = None) -> AgentSkill:
        """
        Args:
            new_name: New name for the agent skill.
            description: Updated description of what the skill does and when to use it.
            allowed_tools: Updated tools the skill is permitted to use.
            comment: The comment attached to the agent skill.
            owner: The identifier of the user who owns the agent skill.

        Returns:
            An agent skill registered in Unity Catalog.
        """

@final
class ListAgentSkillsResponse:
    """
    List agent skills response.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def agent_skills(self, /) -> list[AgentSkill]:
        """
        The agent skills returned.
        """
    @property
    def next_page_token(self, /) -> str | None:
        """
        Opaque token to retrieve the next page of results.

        Absent if there are no more pages. page_token should be set to
        this value for the next request to retrieve the next page of results.
        """
