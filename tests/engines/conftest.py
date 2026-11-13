"""Fixtures: an in-process mangrove server, seeded through the Delta v1 API.

The server is the `uc-test-server` binary from `crates/acceptance`: the same
service `uc-server serve` builds, over in-memory SQLite and a temporary
`file://` managed storage root. Build it first (`just test engines`) or point
`UC_TEST_SERVER_BIN` at a prebuilt binary.
"""

from __future__ import annotations

import json
import os
import subprocess
import urllib.error
import urllib.request
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
CATALOG = "unity"
SCHEMA = "default"
# Tables the server seeds before the tests start, each with rows
# (1, alice), (2, bob), (3, carol), written through /delta/v1.
SEEDED = ("events", "events_insert")
SEEDED_ROWS = [(1, "alice"), (2, "bob"), (3, "carol")]


@dataclass
class UcServer:
    url: str
    storage_root: str
    log: Path

    def requests(self) -> list[str]:
        """Requests served so far, as `<METHOD> <path>` (no status)."""
        lines = self.log.read_text().splitlines()
        return [line.rsplit(" -> ", 1)[0] for line in lines if " -> " in line]

    def delta(self, path: str) -> dict:
        """GET a /delta/v1 path and return the JSON body."""
        url = f"{self.url}/api/2.1/unity-catalog/delta/v1{path}"
        try:
            with urllib.request.urlopen(url) as response:
                return json.load(response)
        except urllib.error.HTTPError as err:
            raise AssertionError(f"GET {path} -> {err.code}: {err.read().decode()}") from err

    def load_table(self, table: str) -> dict:
        return self.delta(f"/catalogs/{CATALOG}/schemas/{SCHEMA}/tables/{table}")


def _server_binary() -> Path:
    if env := os.environ.get("UC_TEST_SERVER_BIN"):
        return Path(env)
    target = Path(os.environ.get("CARGO_TARGET_DIR", REPO_ROOT / "target"))
    return target / "debug" / "uc-test-server"


@pytest.fixture(scope="session")
def uc_server(tmp_path_factory: pytest.TempPathFactory) -> Iterator[UcServer]:
    binary = _server_binary()
    if not binary.exists():
        pytest.fail(
            f"{binary} not found: build it with "
            "`cargo build -p unitycatalog-acceptance --bin uc-test-server`"
        )
    work = tmp_path_factory.mktemp("uc")
    log = work / "server.log"
    args = [str(binary), "--root", str(work / "root"), "--log-requests"]
    for table in SEEDED:
        args += ["--seed", f"{CATALOG}.{SCHEMA}.{table}"]
    with log.open("w") as stderr:
        # stdin stays a pipe: the server exits when it closes, so it can't
        # outlive the test session.
        proc = subprocess.Popen(
            args, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True
        )
        try:
            ready = proc.stdout.readline()
            if not ready:
                proc.wait(timeout=10)
                pytest.fail(f"uc-test-server exited ({proc.returncode}):\n{log.read_text()}")
            info = json.loads(ready)
            yield UcServer(url=info["url"], storage_root=info["storage_root"], log=log)
        finally:
            proc.stdin.close()
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()
