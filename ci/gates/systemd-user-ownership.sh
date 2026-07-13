#!/usr/bin/env bash
# Fail closed on systemd-user content ownership and discovery wiring.
#
# Canonical unit content has exactly one manifest owner under META_ROOT. The
# engine may expose that content only through its verified real-XDG symlink
# bridge; tracked home-tree copies and portability-link projections are stale
# parallel owners and must never return.
set -euo pipefail

ROOT="${1:-}"
if [ -z "$ROOT" ]; then
  ROOT="$(git rev-parse --show-toplevel)"
fi
ROOT="$(cd "$ROOT" && pwd -P)"

python3 - "$ROOT" <<'PY'
from __future__ import annotations

from pathlib import Path
import re
import sys
import tomllib

root = Path(sys.argv[1])


def fail(message: str) -> None:
    print(f"SYSTEMD USER OWNERSHIP GATE FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


expected = {
    "env-ctl.service": "manifest/env-ctl.toml",
    "sqld.service": "manifest/sqld.toml",
    "kache.service": "manifest/components.d/epic-h-toolchains.toml",
}
owners: dict[str, set[str]] = {}
imperative_owners: dict[str, set[str]] = {}
manifest_root = root / "manifest"
if not manifest_root.is_dir():
    fail("missing manifest directory")

writer_re = re.compile(
    r"\b(?:cat|tee|install|cp|mv|ln|touch)\b[^\n]*?"
    r"\.config/systemd/user/(?P<unit>[A-Za-z0-9_.@:-]+\.service)"
)

for path in sorted(manifest_root.rglob("*.toml")):
    rel = path.relative_to(root).as_posix()
    text = path.read_text()
    try:
        document = tomllib.loads(text)
    except tomllib.TOMLDecodeError as error:
        fail(f"cannot parse {rel}: {error}")

    components = document.get("component", [])
    if isinstance(components, dict):
        components = [components]
    for component in components:
        if not isinstance(component, dict):
            continue
        wiring = component.get("wiring", {})
        if not isinstance(wiring, dict):
            continue
        units = wiring.get("systemd_user", [])
        if isinstance(units, dict):
            units = [units]
        for unit in units:
            name = unit.get("name") if isinstance(unit, dict) else None
            if not isinstance(name, str) or not name.endswith(".service"):
                fail(f"{rel} has a systemd_user entry without a valid .service name")
            owners.setdefault(name, set()).add(rel)

    # Shell-hook materialization bypasses the engine's ownership validation and
    # real-XDG discovery bridge, so no active unit may retain it.
    for match in writer_re.finditer(text):
        imperative_owners.setdefault(match.group("unit"), set()).add(rel)

if imperative_owners:
    detail = "; ".join(
        f"{unit}: {', '.join(sorted(paths))}"
        for unit, paths in sorted(imperative_owners.items())
    )
    fail(f"imperative systemd unit materializer(s) bypass engine wiring: {detail}")

for unit, owner in expected.items():
    actual = owners.get(unit, set())
    if actual != {owner}:
        if not actual:
            fail(f"missing expected owner {owner} for {unit}")
        fail(f"{unit} has duplicate or wrong owners: {', '.join(sorted(actual))}; expected {owner}")

unexpected = sorted(set(owners) - set(expected))
if unexpected:
    detail = "; ".join(
        f"{unit}: {', '.join(sorted(owners[unit]))}" for unit in unexpected
    )
    fail(f"unregistered active unit projection(s): {detail}")

home_units = root / "home/.config/systemd/user"
if home_units.exists() or home_units.is_symlink():
    residuals = sorted(
        path.relative_to(root).as_posix()
        for path in home_units.rglob("*")
        if path.is_file() or path.is_symlink()
    )
    if residuals:
        fail("tracked home-tree unit projection(s) remain: " + ", ".join(residuals))

portability = root / "manifest/components.d/portability-links.toml"
if not portability.is_file():
    fail("missing manifest/components.d/portability-links.toml")
portability_text = portability.read_text()
if re.search(r"(?m)^\s*link\s+\.config/systemd/user/", portability_text):
    fail("portability-links must not materialize systemd user units")
if re.search(r"\.config/systemd/user/[A-Za-z0-9_.@:-]+\.service", portability_text):
    fail("portability-links contains a stale named systemd unit projection")

retired = re.compile(
    r"(?:%h|\$HOME|\$\{HOME\})/Desktop/meta"
    r"|/home/[^/\s\"']+/(?:Desktop/)?meta(?:/|\b)"
)
for path in sorted(manifest_root.rglob("*.toml")):
    match = retired.search(path.read_text())
    if match:
        fail(
            f"{path.relative_to(root).as_posix()} contains retired workstation path {match.group(0)!r}"
        )

wiring = root / "crates/engine/src/wiring.rs"
layout = root / "crates/engine/src/layout.rs"
if not wiring.is_file() or not layout.is_file():
    fail("missing engine wiring/layout source")
wiring_text = wiring.read_text()
layout_text = layout.read_text()
required_wiring = (
    "MetaLayout::from_env_required()?",
    "real_user_xdg_config_home",
    "ensure_owned_systemd_bridge_or_absent",
    "std::os::unix::fs::symlink(canonical, bridge)",
    "--property=FragmentPath",
    "run_systemctl(&[\"--user\", \"daemon-reload\"])?",
    "run_systemctl(&[\"--user\", \"enable\", \"--now\", &u.name])?",
)
for needle in required_wiring:
    if needle not in wiring_text:
        fail(f"engine wiring is missing fail-closed discovery contract: {needle}")
if "pub fn systemd_user_dir(&self)" not in layout_text:
    fail("layout does not expose the canonical META_ROOT systemd user directory")

section = wiring_text.split("// ============================== systemd --user", 1)
if len(section) != 2:
    fail("cannot locate systemd-user wiring section")
section = section[1].split("// ================================ apt repos", 1)[0]
if "from_env_or_default" in section or 'join("Desktop/meta")' in section:
    fail("systemd-user mutation still has a HOME/Desktop/meta fallback")

print("SYSTEMD USER OWNERSHIP GATE PASS")
PY
