#!/usr/bin/env python3
"""Validate the durable Yazelix policy and, on request, the live profile runtime."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys


PROFILE_YZX = Path("/home/flexnetos/.nix-profile/bin/yzx")
REQUIRED_COMMANDS = {
    "yzx",
    "yzx agent",
    "yzx config",
    "yzx config set",
    "yzx config ui",
    "yzx config unset",
    "yzx cursors",
    "yzx cursors ghostty setup",
    "yzx desktop",
    "yzx desktop install",
    "yzx desktop launch",
    "yzx desktop macos_preview install",
    "yzx desktop macos_preview uninstall",
    "yzx desktop uninstall",
    "yzx dev",
    "yzx dev inspect_session",
    "yzx dev perf",
    "yzx dev profile",
    "yzx doctor",
    "yzx edit",
    "yzx edit config",
    "yzx enter",
    "yzx env",
    "yzx home_manager",
    "yzx home_manager prepare",
    "yzx import",
    "yzx import helix",
    "yzx import yazi",
    "yzx import zellij",
    "yzx inspect",
    "yzx keys",
    "yzx keys helix",
    "yzx keys hx",
    "yzx keys nu",
    "yzx keys nushell",
    "yzx keys yazi",
    "yzx keys yzx",
    "yzx launch",
    "yzx menu",
    "yzx onboard",
    "yzx popup",
    "yzx reset",
    "yzx reset config",
    "yzx restart",
    "yzx reveal",
    "yzx run",
    "yzx screen",
    "yzx sidebar refresh",
    "yzx sidebar yazi",
    "yzx sponsor",
    "yzx status",
    "yzx tutor",
    "yzx tutor begin",
    "yzx tutor discovery",
    "yzx tutor helix",
    "yzx tutor hx",
    "yzx tutor list",
    "yzx tutor nu",
    "yzx tutor nushell",
    "yzx tutor tool_tutors",
    "yzx tutor troubleshooting",
    "yzx tutor workspace",
    "yzx update",
    "yzx update home_manager",
    "yzx update local_source",
    "yzx update nix",
    "yzx update upstream",
    "yzx whats_new",
    "yzx why",
}
YAZI_PLUGINS = {
    "auto-layout.yazi",
    "git.yazi",
    "lazygit.yazi",
    "sidebar-state.yazi",
    "sidebar-status.yazi",
    "smart-tabs.yazi",
    "starship.yazi",
    "zoxide-editor.yazi",
}
ZELLIJ_PLUGINS = {
    "yazelix_pane_orchestrator.wasm",
    "yzpp.wasm",
    "zjstatus.wasm",
}


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def run_json(argv: list[str]) -> dict:
    completed = subprocess.run(
        argv,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if completed.returncode:
        fail(
            f"{' '.join(argv)} exited {completed.returncode}: "
            f"{completed.stderr.strip()}"
        )
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        fail(f"{' '.join(argv)} did not emit JSON: {error}")


def require_text(path: Path, needles: list[str]) -> None:
    if not path.is_file():
        fail(f"missing required policy file: {path}")
    text = path.read_text()
    missing = [needle for needle in needles if needle not in text]
    if missing:
        fail(f"{path} is missing: {', '.join(missing)}")


def check_static(root: Path, skill_root: Path) -> None:
    require_text(
        skill_root / "SKILL.md",
        [
            "references/yazelix-cli-plugin-policy.md",
            "A toggle may be off",
            "latest available Nix/Yazelix/fenix/Bun-owned binaries",
            "yzx update",
            "yazelix-yazi-assets",
        ],
    )
    require_text(
        skill_root / "references/yazelix-cli-plugin-policy.md",
        [
            "Do not invent a `yzx sync` command",
            "yzx update local_source",
            "yzx update upstream",
            "yzx update home_manager",
            "yzx doctor --fix-plan --json",
            "/home/flexnetos/meta/src/yazelix-yazi-assets",
            "yazelix_helix_cogs_noop_wt",
            "yazelix-helix",
            "yazelix_pane_orchestrator.wasm",
            "yzpp.wasm",
            "zjstatus.wasm",
        ],
    )
    reference = (
        skill_root / "references/yazelix-cli-plugin-policy.md"
    ).read_text()
    missing_documented_commands = []
    for command in sorted(REQUIRED_COMMANDS):
        display = command if command == "yzx" else command.removeprefix("yzx ")
        if f"`{display}`" not in reference:
            missing_documented_commands.append(command)
    if missing_documented_commands:
        fail(
            "Yazelix policy does not document command snapshot entries: "
            + ", ".join(missing_documented_commands)
        )
    require_text(
        root
        / ".codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md",
        [
            "Mandatory-task, latest-toolchain, and Yazelix convergence controller",
            "The word `optional` means mandatory when attached to work",
            "latest available profile-owned toolchain",
            "plugin and add-on source/package/manifest authority",
        ],
    )
    print(
        f"Yazelix durable policy: PASS "
        f"({len(REQUIRED_COMMANDS)} documented commands)"
    )


def check_live(yzx: Path) -> None:
    if yzx != PROFILE_YZX:
        fail(f"live proof must use profile frontdoor {PROFILE_YZX}, got {yzx}")
    if not yzx.is_file():
        fail(f"profile yzx is missing: {yzx}")
    resolved = yzx.resolve()
    if not str(resolved).startswith("/nix/store/"):
        fail(f"profile yzx does not resolve into the Nix store: {resolved}")

    inspect = run_json([str(yzx), "inspect", "--json"])
    status = run_json([str(yzx), "status", "--json"])
    doctor = run_json([str(yzx), "doctor", "--json"])
    helix_doctor = run_json([str(yzx), "doctor", "helix-steel", "--json"])

    commands = {
        entry.get("command")
        for entry in inspect.get("command_metadata", {}).get("commands", [])
    }
    missing_commands = sorted(REQUIRED_COMMANDS - commands)
    if missing_commands:
        fail(f"profile command registry is missing: {', '.join(missing_commands)}")

    runtime = inspect.get("runtime", {})
    if runtime.get("invoked_yzx_path") != str(PROFILE_YZX):
        fail("inspect did not report the profile yzx as the invoked frontdoor")
    runtime_dir = Path(runtime.get("dir", ""))
    if not str(runtime_dir).startswith("/nix/store/") or not runtime_dir.is_dir():
        fail(f"invalid profile runtime root: {runtime_dir}")

    generated = inspect.get("generated_state", {})
    if generated.get("repair_needed") is not False:
        fail("generated Yazelix state requires repair")
    if generated.get("missing_artifacts"):
        fail(f"generated Yazelix artifacts missing: {generated['missing_artifacts']}")
    status_summary = status.get("summary", {})
    if status_summary.get("default_shell") != "nu":
        fail("profile Yazelix default shell is not Nushell")

    runtime_yazi = runtime_dir / "configs/yazi/plugins"
    generated_yazi = Path(
        "/home/flexnetos/var/lib/yazelix/configs/yazi/plugins"
    )
    for root in [runtime_yazi, generated_yazi]:
        missing = sorted(name for name in YAZI_PLUGINS if not (root / name).is_dir())
        if missing:
            fail(f"{root} is missing Yazi plugins: {', '.join(missing)}")

    runtime_zellij = runtime_dir / "configs/zellij/plugins"
    generated_zellij = Path(
        "/home/flexnetos/var/lib/yazelix/configs/zellij/plugins"
    )
    for root in [runtime_zellij, generated_zellij]:
        missing = sorted(name for name in ZELLIJ_PLUGINS if not (root / name).is_file())
        if missing:
            fail(f"{root} is missing Zellij plugins: {', '.join(missing)}")

    steel_root = runtime_dir / "configs/helix/steel_plugins"
    if not (steel_root / "manifest.toml").is_file():
        fail(f"packaged Helix Steel plugin manifest is missing: {steel_root}")
    generated_helix = Path("/home/flexnetos/var/lib/yazelix/configs/helix")
    for name in ["helix.scm", "init.scm", "cogs"]:
        if not (generated_helix / name).exists():
            fail(f"generated Helix Steel surface is missing: {generated_helix / name}")
    if not helix_doctor.get("summary", {}).get("healthy"):
        fail("yzx doctor helix-steel did not report healthy")

    doctor_results = doctor.get("results", [])
    permission_ok = any(
        row.get("status") == "ok"
        and row.get("message") == "Yazelix pane-orchestrator permissions granted"
        for row in doctor_results
    )
    if not permission_ok:
        fail("Yazelix pane-orchestrator is not permissioned and connected")

    tools = {
        row.get("name"): row
        for row in inspect.get("runtime_tools", {}).get("entries", [])
    }
    for name in ["ccboard", "codedb"]:
        row = tools.get(name)
        if not row or row.get("source") != "bundled":
            fail(f"runtime add-on is not bundled: {name}")
        if not any("yazi-assets" in note for note in row.get("notes", [])):
            fail(f"runtime add-on lacks yazi-assets provenance: {name}")

    print(f"Yazelix live profile: PASS ({runtime.get('version')} at {resolved})")
    print(f"Yazelix command registry: PASS ({len(commands)} commands)")
    print("Yazelix generated state: PASS")
    print("Yazelix Yazi/Helix/Zellij plugins and add-ons: PASS")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--root",
        type=Path,
        default=Path("/home/flexnetos/meta/src/envctl"),
        help="envctl checkout containing the durable skill and prompt",
    )
    parser.add_argument(
        "--yzx",
        type=Path,
        default=PROFILE_YZX,
        help="profile yzx frontdoor; live mode rejects non-profile paths",
    )
    parser.add_argument(
        "--live",
        action="store_true",
        help="also prove installed profile/runtime/plugin connectivity",
    )
    args = parser.parse_args()
    skill_root = Path(__file__).resolve().parent.parent
    check_static(args.root.resolve(), skill_root)
    if args.live:
        check_live(args.yzx)


if __name__ == "__main__":
    main()
