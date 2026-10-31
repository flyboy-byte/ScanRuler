#!/usr/bin/env python3
"""Validate this planning packet. Does not build or test ScanRuler or a Rust port."""
import csv
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise SystemExit(f"Packet validation failed: {message}")


def rows(name):
    with (ROOT / name).open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


backlog = json.loads((ROOT / "backlog.json").read_text())
tasks = backlog["tasks"]
ids = {task["id"] for task in tasks}
require(len(ids) == len(tasks) == 40, "task IDs missing or duplicated")
require(not backlog["implementation_started"], "implementation status changed: review packet claims")
canonical = [json.loads(path.read_text()) for path in sorted((ROOT / "plan/units").glob("*.json"))]
require(canonical == tasks, "generated backlog differs from canonical unit files")
require(backlog["goal"]["confirmed_by_user"], "confirmed pure-Rust goal missing")
require(backlog["goal"]["endpoint"] == "pure Rust throughout, including geometry kernels", "pure-Rust endpoint changed")
final_task = next(task for task in tasks if task["id"] == "P29")
require({"P28", "P31", "P32", "P33", "P34", "P35"} <= set(final_task["depends_on"]), "required geometry gates missing from final completion")
require(backlog["baseline"] == "2cd89c6769a10c27ee7754b477ada521c9ef5cb7", "baseline mismatch")
for task in tasks:
    require(set(task["depends_on"]) <= ids, f"unknown dependency in {task['id']}")
    require(task["status"] == "planned", f"unexpected implementation claim in {task['id']}")
    require(task["acceptance"] and task["scope"], f"missing acceptance/scope in {task['id']}")
    require(not task["evidence"] and task["implemented_commit"] is None, "unexpected result claim")

pending = {task["id"]: set(task["depends_on"]) for task in tasks}
order = []
while pending:
    ready = sorted(key for key, depends in pending.items() if not depends)
    require(ready, "dependency graph has a cycle")
    order.extend(ready)
    for key in ready:
        del pending[key]
    for depends in pending.values():
        depends.difference_update(ready)

commands = rows("commands.csv")
features = rows("features.csv")
port_map = rows("port-map.csv")
require(len(commands) == len({row['command'] for row in commands}) == 140, "command inventory mismatch")
require(len(features) == len({row['kind'] for row in features}) == 27, "feature inventory mismatch")
require(len(port_map) == len({row['source_path'] for row in port_map}) == 1246, "file inventory mismatch")
for row in commands + features + port_map:
    for task_id in row["task"].split(";"):
        require(task_id in ids, f"unknown task reference {task_id}")
for row in commands:
    require(row["runtime_verified"] == "false" and row["port_status"] == "planned", "command verification overstated")
for row in features:
    require(row["port_status"] == "planned", "feature verification overstated")
for row in port_map:
    require(row["implementation_state"] == "planned", "file implementation overstated")
    require(re.fullmatch(r"[0-9a-f]{40}", row["git_blob_sha"]) is not None, "invalid Git blob ID")

core = json.loads((ROOT / "reference/core-commands.json").read_text())
require(core["app"] == "0.9.2" and len(core["commands"]) == 72, "core snapshot mismatch")
require({c["name"] for c in core["commands"]} == {r["command"] for r in commands if r["descriptor_state"] == "saved_core_descriptor"}, "core descriptors and ledger differ")
review = json.loads((ROOT / "reference/source-review.json").read_text())
by_path = {row["source_path"]: row for row in port_map}
require(review["baseline"] == backlog["baseline"], "source review baseline mismatch")
require(review["reviewed_file_count"] == len(review["files"]) == 47, "source-review count mismatch")
for source in review["files"]:
    require(source["path"] in by_path, f"review source not inventoried: {source['path']}")
    require(source["blob_sha"] == by_path[source["path"]]["git_blob_sha"], "source blob mismatch")
    require(source["bytes"] == int(by_path[source["path"]]["bytes"]), "source size mismatch")

for document in ROOT.rglob("*.md"):
    for task_id in re.findall(r"\bP\d{2}\b", document.read_text()):
        require(task_id in ids, f"unknown task reference {task_id} in {document.name}")

checksums = ROOT / "SHA256SUMS"
require(checksums.exists(), "SHA256SUMS is missing")
hashed = set()
for line in checksums.read_text().splitlines():
    digest, relative = line.split("  ", 1)
    path = ROOT / relative
    require(not Path(relative).is_absolute() and ".." not in Path(relative).parts, "unsafe checksum path")
    require(path.is_file(), f"missing packet member {relative}")
    require(hashlib.sha256(path.read_bytes()).hexdigest() == digest, f"checksum mismatch for {relative}")
    hashed.add(relative)
actual = {str(path.relative_to(ROOT)) for path in ROOT.rglob("*") if path.is_file() and path.name != "SHA256SUMS"}
require(actual == hashed, "checksum manifest does not cover exactly the packet files")

print("Planning packet verified: 40 acyclic tasks, 140 candidate commands, 27 feature kinds, 1,246 source files, 47 source references.")
print("All implementation states remain planned. No application build or runtime parity has been claimed.")
