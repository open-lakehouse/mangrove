"""
The classes of the `agents` API.
"""

from _typeshed import SupportsGetItem
from typing import final
from unitycatalog_client._enums import InvocationProtocol

__all__ = ["Agent", "AgentClient", "ListAgentsResponse"]

@final
class Agent:
    """
    An agent registered in Unity Catalog.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def agent_id(self, /) -> str:
        """
        The unique identifier of the agent.
        """
    @property
    def capabilities(self, /) -> list[str]:
        """
        Capability identifiers advertised by the agent (e.g. `sql_query`,
        `document_search`, `code_execution`).
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
        Time at which this agent was created, in epoch milliseconds.
        """
    @property
    def created_by(self, /) -> str | None:
        """
        Username of agent creator.
        """
    @property
    def description(self, /) -> str | None:
        """
        An LLM-readable description of what the agent does and the inputs it
        expects.
        """
    @property
    def endpoint(self, /) -> str:
        """
        The agent's invocation endpoint URL.
        """
    @property
    def full_name(self, /) -> str:
        """
        The three-level (fully qualified) name of the agent.
        """
    @property
    def input_schema(self, /) -> str | None:
        """
        A JSON Schema (encoded as a JSON string) describing the agent's expected
        input payload. Kept as an opaque string so the schema can evolve without
        changing this message.
        """
    @property
    def invocation_protocol(self, /) -> InvocationProtocol | None:
        """
        The protocol a recipient uses to invoke the agent.
        """
    @property
    def metastore_id(self, /) -> str | None:
        """
        The unique identifier of the metastore.
        """
    @property
    def name(self, /) -> str:
        """
        Name of the agent, relative to parent schema.
        """
    @property
    def owner(self, /) -> str | None:
        """
        Username of current owner of the agent.
        """
    @property
    def schema_name(self, /) -> str:
        """
        Name of parent schema.
        """
    @property
    def updated_at(self, /) -> int | None:
        """
        Time at which this agent was last updated, in epoch milliseconds.
        """
    @property
    def updated_by(self, /) -> str | None:
        """
        Username of user who last modified the agent.
        """

@final
class AgentClient:
    """
    The requests on one `Agent`.
    """
    def delete(self, /) -> None: ...
    def get(self, /, include_browse: bool | None = None) -> Agent:
        """
        Args:
            include_browse: Whether to include agents in the response for which the principal can only
                            access selective metadata for.

        Returns:
            An agent registered in Unity Catalog.
        """
    def update(self, /, new_name: str | None = None, invocation_protocol: InvocationProtocol | str | None = None, endpoint: str | None = None, description: str | None = None, capabilities: SupportsGetItem[int, str] | None = None, input_schema: str | None = None, comment: str | None = None, owner: str | None = None) -> Agent:
        """
        Args:
            new_name: New name for the agent.
            invocation_protocol: The protocol a recipient uses to invoke the agent.
            endpoint: The agent's invocation endpoint URL.
            description: Updated LLM-readable description.
            capabilities: Updated capability identifiers advertised by the agent.
            input_schema: Updated JSON Schema (encoded as a JSON string) describing the expected input.
            comment: The comment attached to the agent.
            owner: The identifier of the user who owns the agent.

        Returns:
            An agent registered in Unity Catalog.
        """

@final
class ListAgentsResponse:
    """
    List agents response.
    """
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __repr__(self, /) -> str: ...
    @property
    def agents(self, /) -> list[Agent]:
        """
        The agents returned.
        """
    @property
    def next_page_token(self, /) -> str | None:
        """
        Opaque token to retrieve the next page of results.

        Absent if there are no more pages. page_token should be set to
        this value for the next request to retrieve the next page of results.
        """
