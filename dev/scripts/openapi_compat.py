# /// script
# requires-python = ">=3.12"
# dependencies = ["pyyaml>=6"]
# ///
"""Semantic OpenAPI compatibility check: our generated specs vs a reference.

We may serve a *superset* of the reference (UC OSS), so this does not compare
specs literally. For each (ours, reference) pair it:

1. normalizes away representation-only differences (``servers``, gnostic's
   ``format: enum``, proto ``*_UNSPECIFIED`` enum sentinels),
2. scopes both specs to the operations both serve; reference operations we
   don't serve become ``coverage`` findings,
3. runs ``oasdiff breaking`` in both directions:
   - ``server``: reference -> ours. Would a client of the reference break
     against our server?
   - ``client``: ours -> reference. Would our client break against the
     reference server?
4. matches every finding against the accepted-diffs file. Unmatched findings
   fail, and so do accepted entries that no longer match anything, so the
   file stays exactly as large as the real deviations.

It also checks that each reference spec's ``@ <tag>`` header matches the
image tag in the compose file, so bumping the server under test can't leave
the reference spec behind.

Usage (see ``just test openapi-compat``)::

    uv run dev/scripts/openapi_compat.py \
        --pair openapi/openapi.yaml:openapi/reference/uc-oss/all.yaml \
        --accepted openapi/compat/uc-oss.accepted.yaml \
        --compose dev/uc-oss.compose.yaml --out target/openapi-compat
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml

METHODS = {"get", "put", "post", "delete", "patch", "head", "options", "trace"}
DIRECTIONS = {"server", "client", "coverage"}
OASDIFF_FLAGS = ["-f", "json", "--flatten-params", "--flatten-allof", "--allow-external-refs=false"]
HEADER_RE = re.compile(r"^#.*@\s*(\S+)\s*$")
IMAGE_RE = re.compile(r"image:\s*unitycatalog/unitycatalog:(\S+)")
PARAM_RE = re.compile(r"\{[^}]+\}")


@dataclass(frozen=True)
class Finding:
    direction: str
    rule: str
    method: str
    path: str
    text: str

    def render(self) -> str:
        return f"[{self.direction}] {self.rule}  {self.method} {self.path}\n    {self.text}"


# --- normalization -----------------------------------------------------------


def _strip_representation(node: Any) -> Any:
    """Drop gnostic/proto artifacts that carry no wire meaning."""
    if isinstance(node, dict):
        out = {}
        for key, value in node.items():
            if key == "format" and value == "enum":
                continue
            if key == "enum" and isinstance(value, list):
                value = [
                    v for v in value if not (isinstance(v, str) and v.endswith("_UNSPECIFIED"))
                ]
            out[key] = _strip_representation(value)
        return out
    if isinstance(node, list):
        return [_strip_representation(v) for v in node]
    return node


def _operations(spec: dict) -> dict[tuple[str, str], str]:
    """Map (METHOD, path shape) -> literal path. Param names don't count."""
    ops = {}
    for path, item in (spec.get("paths") or {}).items():
        for method in item:
            if method in METHODS:
                ops[(method.upper(), PARAM_RE.sub("{}", path))] = path
    return ops


def _scope(spec: dict, keep: set[tuple[str, str]]) -> dict:
    paths = {}
    for path, item in (spec.get("paths") or {}).items():
        shape = PARAM_RE.sub("{}", path)
        scoped = {k: v for k, v in item.items() if k not in METHODS or (k.upper(), shape) in keep}
        if any(k in METHODS for k in scoped):
            paths[path] = scoped
    return {**spec, "paths": paths}


def normalize(ours: dict, ref: dict) -> tuple[dict, dict, list[Finding], int]:
    ours = _strip_representation({k: v for k, v in ours.items() if k != "servers"})
    ref = _strip_representation({k: v for k, v in ref.items() if k != "servers"})
    ours_ops, ref_ops = _operations(ours), _operations(ref)
    shared = set(ours_ops) & set(ref_ops)
    coverage = [
        Finding(
            "coverage",
            "operation-not-served",
            method,
            ref_ops[(method, shape)],
            "reference operation is not served by us",
        )
        for method, shape in sorted(set(ref_ops) - shared)
    ]
    return _scope(ours, shared), _scope(ref, shared), coverage, len(set(ours_ops) - shared)


# --- diffing -----------------------------------------------------------------


def oasdiff(base: Path, revision: Path, direction: str) -> list[Finding]:
    proc = subprocess.run(
        ["oasdiff", "breaking", str(base), str(revision), *OASDIFF_FLAGS],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        sys.exit(f"oasdiff failed ({direction}):\n{proc.stderr}")
    return [
        Finding(direction, c["id"], c.get("operation", ""), c.get("path", ""), c["text"])
        for c in json.loads(proc.stdout or "[]")
    ]


# --- accepted diffs ----------------------------------------------------------


@dataclass
class Accepted:
    direction: str
    rule: str
    reason: str
    method: str | None = None
    path: re.Pattern | None = None
    text: re.Pattern | None = None
    issue: str | None = None
    hits: int = 0

    def matches(self, f: Finding) -> bool:
        return (
            self.direction == f.direction
            and self.rule == f.rule
            and (self.method is None or self.method == f.method)
            and (self.path is None or self.path.fullmatch(f.path) is not None)
            and (self.text is None or self.text.search(f.text) is not None)
        )

    def label(self) -> str:
        parts = [
            self.direction,
            self.rule,
            self.method or "*",
            self.path.pattern if self.path else "*",
        ]
        if self.text:
            parts.append(f"text~{self.text.pattern!r}")
        return " ".join(parts)


def load_accepted(path: Path) -> list[Accepted]:
    entries = (yaml.safe_load(path.read_text()) or {}).get("accepted") or []
    out = []
    for i, e in enumerate(entries):
        unknown = set(e) - {"direction", "rule", "method", "path", "text", "reason", "issue"}
        if (
            unknown
            or e.get("direction") not in DIRECTIONS
            or not e.get("rule")
            or not e.get("reason")
        ):
            sys.exit(
                f"{path}: entry {i} needs direction ({'/'.join(sorted(DIRECTIONS))}), "
                f"rule and reason, and no unknown keys {sorted(unknown)}: {e}"
            )
        out.append(
            Accepted(
                direction=e["direction"],
                rule=e["rule"],
                reason=e["reason"],
                method=e.get("method"),
                path=re.compile(e["path"]) if e.get("path") else None,
                text=re.compile(e["text"]) if e.get("text") else None,
                issue=e.get("issue"),
            )
        )
    return out


# --- version guard -----------------------------------------------------------


def check_reference_tag(ref_path: Path, compose: Path | None) -> str | None:
    match = HEADER_RE.match(ref_path.read_text().splitlines()[0])
    if not match:
        return f"{ref_path}: first line must be a `# ... @ <tag>` header naming the upstream tag"
    if compose is None:
        return None
    image = IMAGE_RE.search(compose.read_text())
    if not image:
        return f"{compose}: no unitycatalog/unitycatalog image tag found"
    if image.group(1) != match.group(1):
        return (
            f"{ref_path} is from {match.group(1)} but {compose} runs {image.group(1)}: "
            f"run `just codegen bump-uc-oss {image.group(1)}`"
        )
    return None


# --- main --------------------------------------------------------------------


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--pair", action="append", required=True, metavar="OURS:REFERENCE")
    parser.add_argument("--accepted", type=Path, required=True)
    parser.add_argument(
        "--compose", type=Path, help="compose file whose image tag must match the references"
    )
    parser.add_argument("--out", type=Path, default=Path("target/openapi-compat"))
    args = parser.parse_args()

    if shutil.which("oasdiff") is None:
        sys.exit(
            "oasdiff not found on PATH (https://github.com/oasdiff/oasdiff; `brew install oasdiff`)"
        )
    args.out.mkdir(parents=True, exist_ok=True)

    errors: list[str] = []
    findings: list[Finding] = []
    for pair in args.pair:
        ours_path, ref_path = (Path(p) for p in pair.split(":", 1))
        if err := check_reference_tag(ref_path, args.compose):
            errors.append(err)
        ours, ref, coverage, superset = normalize(
            yaml.safe_load(ours_path.read_text()), yaml.safe_load(ref_path.read_text())
        )
        stem = ours_path.stem
        ours_norm, ref_norm = args.out / f"{stem}.ours.yaml", args.out / f"{stem}.reference.yaml"
        ours_norm.write_text(yaml.safe_dump(ours, sort_keys=False))
        ref_norm.write_text(yaml.safe_dump(ref, sort_keys=False))
        pair_findings = (
            coverage
            + oasdiff(ref_norm, ours_norm, "server")
            + oasdiff(ours_norm, ref_norm, "client")
        )
        print(
            f"{ours_path} vs {ref_path}: {len(_operations(ours))} shared operations, "
            f"{superset} ours-only (superset), {len(pair_findings)} findings"
        )
        findings += pair_findings

    accepted = load_accepted(args.accepted)
    unaccepted = []
    for f in findings:
        hits = [a for a in accepted if a.matches(f)]
        for a in hits:
            a.hits += 1
        if not hits:
            unaccepted.append(f)
    stale = [a for a in accepted if a.hits == 0]

    (args.out / "report.json").write_text(
        json.dumps(
            {
                "findings": [f.__dict__ for f in findings],
                "unaccepted": [f.__dict__ for f in unaccepted],
                "stale": [a.label() for a in stale],
            },
            indent=2,
        )
    )

    for err in errors:
        print(f"\nERROR: {err}")
    if unaccepted:
        print(
            f"\n{len(unaccepted)} finding(s) not in {args.accepted}: fix the proto, or accept with a reason:\n"
        )
        for f in unaccepted:
            print(f.render())
    if stale:
        print(
            f"\n{len(stale)} accepted entr(y/ies) in {args.accepted} no longer match anything: remove them:\n"
        )
        for a in stale:
            print(f"  {a.label()}")
    ok = not (errors or unaccepted or stale)
    print(
        f"\nopenapi-compat: {len(findings)} findings, {len(findings) - len(unaccepted)} accepted, "
        f"{len(unaccepted)} unaccepted, {len(stale)} stale -> {'OK' if ok else 'FAIL'}"
    )
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
