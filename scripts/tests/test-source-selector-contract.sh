#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

python3 - <<'PY'
from pathlib import Path
import re
import tomllib

codex_path = Path("manifest/components.d/codex-global-baseline.toml")
env_ctl_path = Path("manifest/env-ctl.toml")
codex_text = codex_path.read_text()
env_ctl_text = env_ctl_path.read_text()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


# Every manifest must parse and every executable/reference surface must be free of the retired
# checkout layout. ShippedScript paths use the same explicit selector as inline shell hooks; the
# runner expands this token before sudo so isolated-worktree assets remain exact.
manifest_paths = sorted(Path("manifest").rglob("*.toml"))
retired_root = re.compile(r"\$(?:M|\{M\}|META_ROOT|\{META_ROOT\})/envctl(?=/|[\"'\s;]|$)")
ambient_meta_repo = re.compile(r"\$\{[A-Z][A-Z0-9_]*_REPO:-\$(?:M|META_ROOT)/")
parsed_manifests = {}
for path in manifest_paths:
    text = path.read_text()
    parsed_manifests[path] = tomllib.loads(text)
    require(not retired_root.search(text), f"{path}: retired META_ROOT/envctl source fallback returned")
    require("ENVCTL_ROOT" not in text, f"{path}: stale source selector ENVCTL_ROOT returned")
    require("ENV_CTL_REPO" not in text, f"{path}: stale source selector ENV_CTL_REPO returned")
    require(
        not ambient_meta_repo.search(text),
        f"{path}: ambient *_REPO selector still controls a managed META_ROOT sibling checkout",
    )
    require("$PWD/assets" not in text, f"{path}: ambient working-directory asset fallback returned")

# Parse the real manifests first: a textual selector match must never mask invalid TOML.
codex = parsed_manifests[codex_path]["component"]
env_ctl = parsed_manifests[env_ctl_path]["component"]
require(
    len(codex) == 1 and codex[0]["id"] == "codex-global-baseline",
    f"{codex_path}: expected exactly the codex-global-baseline component",
)
require(
    len(env_ctl) == 1 and env_ctl[0]["id"] == "env-ctl",
    f"{env_ctl_path}: expected exactly the env-ctl component",
)

stale_selectors = ("ENVCTL_ROOT", "ENV_CTL_REPO")
for path, text in ((codex_path, codex_text), (env_ctl_path, env_ctl_text)):
    for selector in stale_selectors:
        require(selector not in text, f"{path}: stale source selector {selector} returned")

codex_component = codex[0]
codex_lifecycle = "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-codex-global-baseline-lifecycle.sh"
for phase in ("detect", "install", "verify", "fix", "remove"):
    hook = codex_component[phase]
    require(hook.get("kind") == "shipped_script", (
        f"{codex_path}: {phase} must use the audited shipped lifecycle"
    ))
    require(hook.get("path") == codex_lifecycle, (
        f"{codex_path}: {phase} bypasses the canonical source selector"
    ))
    require(hook.get("args") == [phase], (
        f"{codex_path}: {phase} does not preserve lifecycle phase parity"
    ))

shell_selector = '"${ENVCTL_SOURCE_ROOT:-$META_ROOT/src/envctl}"'
env_ctl_component = env_ctl[0]
for phase in ("install", "fix"):
    hook = env_ctl_component[phase]["script"]
    require(f"repo={shell_selector}" in hook, (
        f"{env_ctl_path}: {phase} must select ENVCTL_SOURCE_ROOT with "
        "$META_ROOT/src/envctl fallback"
    ))
    require("set ENVCTL_SOURCE_ROOT" in hook, (
        f"{env_ctl_path}: {phase} failure guidance must name the canonical selector"
    ))

selector_manifests = (
    Path("manifest/ai-clis.toml"),
    codex_path,
    Path("manifest/components.d/epic-h-toolchains.toml"),
    Path("manifest/components.d/meta-env-plugin.toml"),
    Path("manifest/components.d/portability-links.toml"),
    env_ctl_path,
    Path("manifest/nix-yazelix.toml"),
)
for path in selector_manifests:
    require(
        "ENVCTL_SOURCE_ROOT" in path.read_text(),
        f"{path}: executable source references must honor ENVCTL_SOURCE_ROOT",
    )

managed_repo_contracts = (
    (Path("manifest/grit.toml"), "$META_ROOT/src/grit", "GRIT_REPO"),
    (Path("manifest/prompt_hub.toml"), "$META_ROOT/src/prompt_hub", "PROMPT_HUB_REPO"),
    (Path("manifest/rusty-idd.toml"), "$M/src/rusty-idd", "RUSTY_IDD_REPO"),
    (Path("manifest/components.d/handoff-hf.toml"), "$M/src/handoff", "HANDOFF_REPO"),
)
for path, managed_root, retired_selector in managed_repo_contracts:
    text = path.read_text()
    require(
        text.count(managed_root) >= 2,
        f"{path}: install/fix must use managed sibling checkout {managed_root}",
    )
    require(
        retired_selector not in text,
        f"{path}: ambient {retired_selector} override or misleading guidance returned",
    )

shipped_scripts = []
for path, data in parsed_manifests.items():
    for component in data.get("component", []):
        for phase in ("detect", "install", "verify", "fix", "remove"):
            hook = component.get(phase)
            if isinstance(hook, dict) and hook.get("kind") == "shipped_script":
                shipped_scripts.append((path, component["id"], phase, hook["path"]))
expected_shipped_scripts = {
    (Path("manifest/boot-repair.toml"), "boot-repair-diagnose", "fix"):
        "$ENVCTL_SOURCE_ROOT/assets/scripts/ubuntu-boot-repair.sh",
    (Path("manifest/boot-repair.toml"), "boot-repair-dev", "fix"):
        "$ENVCTL_SOURCE_ROOT/assets/scripts/ubuntu-boot-repair.sh",
    (Path("manifest/boot-repair.toml"), "boot-repair-rename-pro", "fix"):
        "$ENVCTL_SOURCE_ROOT/assets/scripts/ubuntu-boot-repair.sh",
    (Path("manifest/boot-repair.toml"), "boot-repair-finalize", "fix"):
        "$ENVCTL_SOURCE_ROOT/assets/scripts/ubuntu-boot-repair.sh",
    (Path("manifest/gpu.toml"), "gpu-verify-scripts", "install"):
        "$ENVCTL_SOURCE_ROOT/assets/scripts/yazelix-gpu-verify-install.sh",
    (Path("manifest/gpu.toml"), "gpu-verify-scripts", "fix"):
        "$ENVCTL_SOURCE_ROOT/assets/scripts/yazelix-gpu-verify-install.sh",
    **{
        (Path("manifest/ai-clis.toml"), component, phase):
            "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh"
        for component in ("codex-cli", "codex-cli-alpha")
        for phase in ("detect", "install", "verify", "fix", "remove")
    },
    **{
        (Path("manifest/base.toml"), "rtk", phase):
            "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-rtk-profile-lifecycle.sh"
        for phase in ("detect", "install", "verify", "fix", "remove")
    },
    **{
        (codex_path, "codex-global-baseline", phase): codex_lifecycle
        for phase in ("detect", "install", "verify", "fix", "remove")
    },
    (Path("manifest/nix-yazelix.toml"), "yazelix", "install"):
        "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh",
    (Path("manifest/nix-yazelix.toml"), "yazelix", "fix"):
        "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh",
    (Path("manifest/nix-yazelix.toml"), "yazelix", "remove"):
        "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh",
    **{
        (Path("manifest/nix-yazelix.toml"), "home-manager", phase):
            "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh"
        for phase in ("install", "fix")
    },
    (Path("manifest/nix-yazelix.toml"), "yazelix-config", "install"):
        "$ENVCTL_SOURCE_ROOT/assets/scripts/yazelix-config.sh",
    (Path("manifest/nix-yazelix.toml"), "yazelix-config", "fix"):
        "$ENVCTL_SOURCE_ROOT/assets/scripts/yazelix-config.sh",
}
actual_shipped_scripts = {
    (path, component_id, phase): script_path
    for path, component_id, phase, script_path in shipped_scripts
}
require(
    actual_shipped_scripts == expected_shipped_scripts,
    "ShippedScript hook inventory drifted; audit each new/removed phase and its selector explicitly",
)
for path, component_id, phase, script_path in shipped_scripts:
    require(
        script_path.startswith("$ENVCTL_SOURCE_ROOT/assets/scripts/"),
        f"{path}: {component_id}.{phase} must use the canonical ENVCTL_SOURCE_ROOT token",
    )

yazelix = parsed_manifests[Path("manifest/nix-yazelix.toml")]["component"]
yazelix_component = next(item for item in yazelix if item["id"] == "yazelix")
for phase in ("install", "fix", "remove"):
    require(
        yazelix_component[phase].get("args") == [phase],
        f"manifest/nix-yazelix.toml: yazelix.{phase} must dispatch the matching lifecycle action",
    )
PY

echo "source-selector-contract: canonical ENVCTL_SOURCE_ROOT selectors and fallbacks are locked"
