#!/usr/bin/env python3
"""Verify the REQ-031 operator command surface contract."""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "generated" / "nu_plugin_command_manifest.json"
TEMPLATE = ROOT / "examples" / "nu" / "operator-session-template.nu"

REQUIRED_COMMANDS = {
    "envctl migration target list": "read",
    "envctl migration run plan": "read",
    "envctl migration run start": "mutate",
    "envctl migration pause": "mutate",
    "envctl migration resume": "mutate",
    "envctl migration approve": "mutate",
    "envctl migration status": "read",
    "envctl migration proof": "read",
    "envctl migration replay": "read",
}

MUTATING_COMMANDS = {
    command for command, mode in REQUIRED_COMMANDS.items() if mode == "mutate"
}


def fail(message: str) -> None:
    print(json.dumps({"status": "failed", "error": message}, indent=2))
    sys.exit(1)


def load_manifest() -> dict:
    try:
        return json.loads(MANIFEST.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing manifest: {MANIFEST.relative_to(ROOT)}")
    except json.JSONDecodeError as error:
        fail(f"manifest is not valid JSON: {error}")


def command_map(manifest: dict) -> dict[str, dict]:
    commands = manifest.get("commands")
    if not isinstance(commands, list):
        fail("manifest.commands must be an array")
    mapped: dict[str, dict] = {}
    for command in commands:
        name = command.get("name")
        if not isinstance(name, str):
            fail("every command must have a string name")
        mapped[name] = command
    return mapped


def verify_command(name: str, expected_mode: str, command: dict) -> None:
    mode = command.get("mode")
    if mode != expected_mode:
        fail(f"{name} mode is {mode!r}; expected {expected_mode!r}")

    endpoint = command.get("envctl_endpoint", "")
    if not isinstance(endpoint, str) or not endpoint.startswith("envctl migration "):
        fail(f"{name} endpoint must route through envctl migration")

    if expected_mode == "mutate" and "--emit-event" not in endpoint:
        fail(f"{name} mutates without --emit-event")

    output = command.get("output")
    if not isinstance(output, dict):
        fail(f"{name} missing structured output")

    shape = output.get("shape")
    columns = output.get("columns")
    if shape not in {"record", "table"}:
        fail(f"{name} output shape must be record or table")
    if not isinstance(columns, list) or not all(isinstance(item, str) for item in columns):
        fail(f"{name} output columns must be a string array")
    if not columns:
        fail(f"{name} output columns must not be empty")


def verify_template() -> None:
    try:
        text = TEMPLATE.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing operator template: {TEMPLATE.relative_to(ROOT)}")

    missing = [name for name in REQUIRED_COMMANDS if name not in text]
    if missing:
        fail(f"operator template does not exercise commands: {', '.join(missing)}")


def main() -> None:
    manifest = load_manifest()
    commands = command_map(manifest)

    declared = manifest.get("operator_command_surface", {}).get("required_commands", [])
    if sorted(declared) != sorted(REQUIRED_COMMANDS):
        fail("operator_command_surface.required_commands does not match REQ-031")

    missing = [name for name in REQUIRED_COMMANDS if name not in commands]
    if missing:
        fail(f"missing commands: {', '.join(missing)}")

    for name, expected_mode in REQUIRED_COMMANDS.items():
        verify_command(name, expected_mode, commands[name])

    direct_state_writes = [
        name
        for name in MUTATING_COMMANDS
        if "database" in json.dumps(commands[name], sort_keys=True).lower()
    ]
    if direct_state_writes:
        fail(f"mutating commands mention direct database writes: {direct_state_writes}")

    verify_template()

    summary = {
        "status": "passed",
        "manifest": str(MANIFEST.relative_to(ROOT)),
        "operator_template": str(TEMPLATE.relative_to(ROOT)),
        "required_command_count": len(REQUIRED_COMMANDS),
        "mutating_command_count": len(MUTATING_COMMANDS),
        "read_command_count": len(REQUIRED_COMMANDS) - len(MUTATING_COMMANDS),
        "required_commands": sorted(REQUIRED_COMMANDS),
    }
    print(json.dumps(summary, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
