"""DuckDB against mangrove, through the `unity_catalog` and `delta` extensions.

Mirrors docs-factory's `read_delta_duckdb.py`, with assertions. As of DuckDB
2.0.0 the `unity_catalog` extension lists through the UC REST API (`/schemas`,
`/tables`) and reads and writes through `/delta/v1`: `loadTable` for reads
(including commits the catalog ratified but nobody backfilled), and
staging + createTable + updateTable `add-commit` for writes. CTAS, DROP and
ALTER are not implemented by the extension; strict xfails flag when they land.
"""

from __future__ import annotations

from collections.abc import Callable, Iterator

import duckdb
import pytest
from conftest import CATALOG, SCHEMA, SEEDED_ROWS, UcServer

DELTA = "/api/2.1/unity-catalog/delta/v1"


@pytest.fixture
def connect(uc_server: UcServer) -> Iterator[Callable[[], duckdb.DuckDBPyConnection]]:
    """Open fresh DuckDB connections with the catalog attached.

    A second connection doesn't share the first one's catalog cache, so it
    shows what the server stored rather than what DuckDB remembers.
    """
    opened: list[duckdb.DuckDBPyConnection] = []

    def open_connection() -> duckdb.DuckDBPyConnection:
        con = duckdb.connect()
        opened.append(con)
        for extension in ("delta", "unity_catalog"):
            con.execute(f"INSTALL {extension}")
            con.execute(f"LOAD {extension}")
        # An unnamed secret: DuckDB doesn't apply a named one's ENDPOINT to the
        # catalog's requests (docs-factory brief).
        con.execute(
            "CREATE SECRET (TYPE unity_catalog, TOKEN 'not-used', ENDPOINT $endpoint)",
            {"endpoint": uc_server.url},
        )
        con.execute(
            f"ATTACH '{CATALOG}' AS {CATALOG} (TYPE unity_catalog, DEFAULT_SCHEMA '{SCHEMA}')"
        )
        return con

    yield open_connection
    for con in opened:
        con.close()


@pytest.fixture
def con(connect: Callable[[], duckdb.DuckDBPyConnection]) -> duckdb.DuckDBPyConnection:
    return connect()


def test_lists_seeded_tables(con: duckdb.DuckDBPyConnection) -> None:
    # Not `SHOW ALL TABLES`: on a freshly attached catalog DuckDB 2.0.0 returns
    # it empty about a third of the time, though the server answered `/tables`.
    tables = {name for (name,) in con.execute(f"SHOW TABLES FROM {CATALOG}.{SCHEMA}").fetchall()}
    assert {"events", "events_insert"} <= tables


def test_reads_a_catalog_managed_table(con: duckdb.DuckDBPyConnection, uc_server: UcServer) -> None:
    # v1 of the seeded table is a staged commit only the catalog knows about.
    loaded = uc_server.load_table("events")
    assert loaded["latest-table-version"] == 1
    assert [c["version"] for c in loaded["commits"]] == [1]

    rows = con.execute(f"SELECT * FROM {CATALOG}.{SCHEMA}.events ORDER BY id").fetchall()
    assert rows == SEEDED_ROWS
    assert f"GET {DELTA}/catalogs/{CATALOG}/schemas/{SCHEMA}/tables/events" in uc_server.requests()


def test_insert_commits_through_the_catalog(
    con: duckdb.DuckDBPyConnection, uc_server: UcServer
) -> None:
    table = f"{CATALOG}.{SCHEMA}.events_insert"
    con.execute(f"INSERT INTO {table} VALUES (4, 'dave')")

    loaded = uc_server.load_table("events_insert")
    assert loaded["latest-table-version"] == 2
    assert f"POST {DELTA}/catalogs/{CATALOG}/schemas/{SCHEMA}/tables/events_insert" in (
        uc_server.requests()
    )
    rows = con.execute(f"SELECT * FROM {table} ORDER BY id").fetchall()
    assert rows == [*SEEDED_ROWS, (4, "dave")]


def test_create_table_goes_through_staging(
    connect: Callable[[], duckdb.DuckDBPyConnection], uc_server: UcServer
) -> None:
    table = f"{CATALOG}.{SCHEMA}.made_by_duckdb"
    con = connect()
    con.execute(f"CREATE TABLE {table} (id BIGINT, name VARCHAR)")
    con.execute(f"INSERT INTO {table} VALUES (1, 'x'), (2, 'y')")

    requests = uc_server.requests()
    assert f"POST {DELTA}/catalogs/{CATALOG}/schemas/{SCHEMA}/staging-tables" in requests
    assert f"POST {DELTA}/catalogs/{CATALOG}/schemas/{SCHEMA}/tables" in requests

    loaded = uc_server.load_table("made_by_duckdb")
    assert loaded["metadata"]["table-type"] == "MANAGED"
    assert loaded["metadata"]["location"].startswith(uc_server.storage_root)
    assert loaded["latest-table-version"] >= 1

    # A connection that didn't create the table reads it from what the server
    # stored, not from the creating connection's cache.
    rows = connect().execute(f"SELECT * FROM {table} ORDER BY id").fetchall()
    assert rows == [(1, "x"), (2, "y")]


def test_create_existing_table_conflicts(
    con: duckdb.DuckDBPyConnection, uc_server: UcServer
) -> None:
    # DuckDB asks the server for a staging reservation, which answers 409.
    with pytest.raises(duckdb.CatalogException, match="already exists"):
        con.execute(f"CREATE TABLE {CATALOG}.{SCHEMA}.events (id BIGINT)")
    staging = f"POST {DELTA}/catalogs/{CATALOG}/schemas/{SCHEMA}/staging-tables -> 409"
    assert staging in uc_server.responses()

    # IF NOT EXISTS takes the same 409 as success and leaves the table alone.
    con.execute(f"CREATE TABLE IF NOT EXISTS {CATALOG}.{SCHEMA}.events (id BIGINT)")
    rows = con.execute(f"SELECT * FROM {CATALOG}.{SCHEMA}.events ORDER BY id").fetchall()
    assert rows == SEEDED_ROWS


def test_missing_table_is_a_catalog_error(con: duckdb.DuckDBPyConnection) -> None:
    with pytest.raises(duckdb.CatalogException, match="does not exist"):
        con.execute(f"SELECT * FROM {CATALOG}.{SCHEMA}.missing")


@pytest.mark.xfail(
    raises=duckdb.NotImplementedException,
    strict=True,
    reason="DuckDB's unity_catalog extension doesn't implement ALTER (UCTableSet::AlterTable)",
)
def test_alter_table(con: duckdb.DuckDBPyConnection) -> None:
    con.execute(f"ALTER TABLE {CATALOG}.{SCHEMA}.events ADD COLUMN extra INTEGER")


@pytest.mark.xfail(
    raises=duckdb.NotImplementedException,
    strict=True,
    reason="DuckDB's unity_catalog extension doesn't implement CTAS (PlanCreateTableAs)",
)
def test_create_table_as_select(con: duckdb.DuckDBPyConnection) -> None:
    con.execute(
        f"CREATE TABLE {CATALOG}.{SCHEMA}.made_by_ctas AS "
        f"SELECT * FROM {CATALOG}.{SCHEMA}.events WHERE id > 1"
    )


@pytest.mark.xfail(
    raises=duckdb.NotImplementedException,
    strict=True,
    reason="DuckDB's unity_catalog extension doesn't implement DROP (UCTableSet::DropEntry)",
)
def test_drop_table(con: duckdb.DuckDBPyConnection) -> None:
    table = f"{CATALOG}.{SCHEMA}.drop_me"
    con.execute(f"CREATE TABLE {table} (id BIGINT)")
    con.execute(f"DROP TABLE {table}")
