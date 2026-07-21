//! Runtime-ownership validation for declarative agent environments.
//!
//! Agent assets are only safe when the shell that launches them has one unambiguous owner.
//! The Yazelix contract keeps editable user config under `$HOME/.config`, while the generated
//! Nushell routing module, native RTK binary, and every executable frontdoor live under exactly
//! one direct Nix profile. This module is read-only and fails closed before agent sync,
//! lock, or doctor can bless a drifted runtime.

use std::fs;
use std::path::Path;

#[cfg(test)]
use std::path::PathBuf;

use crate::config::RuntimeContract;
use crate::dirs::dirs_home;
use crate::{err, Result};

const FOUNDATION_ELEMENT: &str = "lifeos_foundation_yzx";
const FOUNDATION_PRIORITY: u64 = 5;
const PROFILE_RELATIVE: &str = ".nix-profile";
const FRONTDOOR_NAME: &str = ".nix-profile";
const HOST_NU_CONFIG_RELATIVE: &str = ".config/nushell/config.nu";
const YAZELIX_NU_HOOK_RELATIVE: &str = ".config/yazelix/shell_nu.nu";
const PROFILE_NU_CONFIG_RELATIVE: &str = "nushell/config/config.nu";
const RTK_MODULE_RELATIVE: &str = "nushell/config/rtk_wrappers.nu";

/// Audit the configured runtime contract against the current real home.  Configurations that do
/// not opt into a runtime contract retain the portable agent-env behavior unchanged.
pub fn validate_runtime_contract(contract: Option<RuntimeContract>) -> Result<()> {
    match contract {
        None => Ok(()),
        Some(RuntimeContract::YazelixNushell) => {
            let home = dirs_home()?;
            validate_yazelix_nushell_at(&home, Path::new("/nix/store"))
        }
    }
}

fn validate_yazelix_nushell_at(home: &Path, store_root: &Path) -> Result<()> {
    let profile = home.join(PROFILE_RELATIVE);
    let selector = fs::read_link(&profile).map_err(|source| {
        err(format!(
            "Yazelix Nix profile selector is unreadable at {}: {source}",
            profile.display()
        ))
    })?;
    let selector_text = selector.to_string_lossy();
    if selector.is_absolute()
        || selector.components().count() != 1
        || !is_profile_generation_name(&selector_text)
    {
        return Err(err(format!(
            "Yazelix Nix profile selector must name one direct .nix-profile-N-link generation: {}",
            profile.display()
        )));
    }
    reject_parallel_profile_frontdoors(home, &selector_text)?;
    let generation_target = resolve_profile_generation(
        profile.parent().expect("profile has parent"),
        &selector,
        store_root,
    )?;
    let profile_root = profile.canonicalize().map_err(|source| {
        err(format!(
            "Yazelix Nix profile frontdoor cannot be resolved at {}: {source}",
            profile.display()
        ))
    })?;
    if profile_root != generation_target {
        return Err(err(format!(
            "Yazelix Nix profile frontdoor does not resolve to its selected generation: {}",
            profile.display()
        )));
    }

    validate_foundation_manifest(&profile_root, store_root)?;
    validate_packaged_nushell(&profile_root)?;
    validate_editable_nushell_inputs(home)?;
    Ok(())
}

fn reject_parallel_profile_frontdoors(home: &Path, selected: &str) -> Result<()> {
    let entries = fs::read_dir(home).map_err(|source| {
        err(format!(
            "cannot inspect home for retired Nix profile frontdoors at {}: {source}",
            home.display()
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| err(format!("cannot read home entry: {source}")))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(".nix-profile-") && name.ends_with("-link") && name != selected {
            return Err(err(format!(
                "parallel Nix profile generation remains at {}; only {} may select one generation",
                entry.path().display(),
                home.join(FRONTDOOR_NAME).display()
            )));
        }
    }
    Ok(())
}

fn is_profile_generation_name(value: &str) -> bool {
    let Some(mut suffix) = value.strip_prefix(".nix-profile-") else {
        return false;
    };
    loop {
        let Some((number, rest)) = suffix.split_once("-link") else {
            return false;
        };
        if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) || number == "0" {
            return false;
        }
        if rest.is_empty() {
            return true;
        }
        let Some(next) = rest.strip_prefix('-') else {
            return false;
        };
        suffix = next;
    }
}

fn resolve_profile_generation(
    profile_dir: &Path,
    selector: &Path,
    store_root: &Path,
) -> Result<std::path::PathBuf> {
    let mut link = profile_dir.join(selector);
    for _ in 0..16 {
        let target = fs::read_link(&link).map_err(|source| {
            err(format!(
                "Yazelix Nix profile generation is unreadable at {}: {source}",
                link.display()
            ))
        })?;
        if target.is_absolute() {
            if target.parent() == Some(store_root)
                && target
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().ends_with("-profile"))
            {
                return Ok(target);
            }
            return Err(err(format!(
                "Yazelix Nix profile generation must resolve directly under {}: {}",
                store_root.display(),
                link.display()
            )));
        }
        if target.components().count() != 1
            || !is_profile_generation_name(&target.to_string_lossy())
        {
            return Err(err(format!(
                "Yazelix Nix profile generation must link only to a direct .nix-profile-N-link generation: {}",
                link.display()
            )));
        }
        link = profile_dir.join(target);
    }
    Err(err(
        "Yazelix Nix profile generation link chain exceeds 16 entries",
    ))
}

fn validate_foundation_manifest(profile_root: &Path, store_root: &Path) -> Result<()> {
    let manifest_path = profile_root.join("manifest.json");
    let bytes = fs::read(&manifest_path).map_err(|source| {
        err(format!(
            "Yazelix profile manifest is unreadable at {}: {source}",
            manifest_path.display()
        ))
    })?;
    let manifest: serde_json::Value = serde_json::from_slice(&bytes).map_err(|source| {
        err(format!(
            "Yazelix profile manifest is invalid JSON at {}: {source}",
            manifest_path.display()
        ))
    })?;
    if manifest.get("version").and_then(serde_json::Value::as_u64) != Some(3) {
        return Err(err(
            "Yazelix profile manifest must use Nix profile schema v3",
        ));
    }
    let elements = manifest
        .get("elements")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| err("Yazelix profile manifest has no elements object"))?;
    if elements.len() != 1 || !elements.contains_key(FOUNDATION_ELEMENT) {
        return Err(err(format!(
            "Yazelix profile manifest must contain only `{FOUNDATION_ELEMENT}`"
        )));
    }
    let foundation = &elements[FOUNDATION_ELEMENT];
    if foundation
        .get("active")
        .and_then(serde_json::Value::as_bool)
        != Some(true)
        || foundation
            .get("priority")
            .and_then(serde_json::Value::as_u64)
            != Some(FOUNDATION_PRIORITY)
    {
        return Err(err(format!(
            "Yazelix foundation profile element must be active at priority {FOUNDATION_PRIORITY}"
        )));
    }
    let Some(attribute) = foundation
        .get("attrPath")
        .and_then(serde_json::Value::as_str)
    else {
        return Err(err(
            "Yazelix foundation profile element is missing attrPath",
        ));
    };
    if !attribute.starts_with("packages.")
        || !attribute.ends_with(&format!(".{FOUNDATION_ELEMENT}"))
    {
        return Err(err(format!(
            "Yazelix foundation profile element has a non-canonical attrPath: {attribute}"
        )));
    }
    let Some(paths) = foundation
        .get("storePaths")
        .and_then(serde_json::Value::as_array)
    else {
        return Err(err(
            "Yazelix foundation profile element is missing storePaths",
        ));
    };
    if paths.len() != 1 {
        return Err(err(
            "Yazelix foundation profile element must own exactly one store path",
        ));
    }
    let Some(path) = paths[0].as_str() else {
        return Err(err("Yazelix foundation store path must be a string"));
    };
    let path = Path::new(path);
    if !path.starts_with(store_root)
        || !path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with("-lifeos-foundation-yzx"))
    {
        return Err(err("Yazelix foundation store path is not profile-owned"));
    }
    Ok(())
}

fn validate_packaged_nushell(profile_root: &Path) -> Result<()> {
    for relative in ["toolbin/nu", "bin/rtk", "toolbin/rtk", RTK_MODULE_RELATIVE] {
        require_regular_file(&profile_root.join(relative), "Yazelix profile runtime")?;
    }
    let profile_rtk = fs::canonicalize(profile_root.join("bin/rtk")).map_err(|source| {
        err(format!(
            "Yazelix profile RTK frontdoor is unavailable at {}: {source}",
            profile_root.join("bin/rtk").display()
        ))
    })?;
    let nu_rtk = fs::canonicalize(profile_root.join("toolbin/rtk")).map_err(|source| {
        err(format!(
            "Yazelix Nu RTK frontdoor is unavailable at {}: {source}",
            profile_root.join("toolbin/rtk").display()
        ))
    })?;
    if profile_rtk != nu_rtk {
        return Err(err(
            "Yazelix profile bin/rtk and toolbin/rtk must resolve to the same native binary",
        ));
    }
    let runtime_config = profile_root.join(PROFILE_NU_CONFIG_RELATIVE);
    require_regular_file(&runtime_config, "Yazelix packaged Nu config")?;
    let runtime_config_text = read_utf8(&runtime_config, "Yazelix packaged Nu config")?;
    if !contains_active_line(&runtime_config_text, "use rtk_wrappers.nu *") {
        return Err(err(format!(
            "Yazelix packaged Nu config must import its native RTK module: {}",
            runtime_config.display()
        )));
    }
    let module = profile_root.join(RTK_MODULE_RELATIVE);
    let module_text = read_utf8(&module, "Yazelix packaged RTK module")?;
    for required in [
        "export def --wrapped codex",
        "^rtk codex",
        "export def --wrapped cargo",
        "^rtk cargo",
    ] {
        if !module_text.contains(required) {
            return Err(err(format!(
                "Yazelix packaged RTK module is missing native Nu routing `{required}`"
            )));
        }
    }
    Ok(())
}

fn validate_editable_nushell_inputs(home: &Path) -> Result<()> {
    let host_config = home.join(HOST_NU_CONFIG_RELATIVE);
    let host_text = read_utf8(&host_config, "editable Nushell config")?;
    const PROFILE_MODULE_SOURCE: &str = "use ~/.nix-profile/nushell/config/rtk_wrappers.nu *";
    if !contains_active_line(&host_text, PROFILE_MODULE_SOURCE) {
        return Err(err(format!(
            "editable Nushell config must import only the profile-owned RTK module: {}",
            host_config.display()
        )));
    }
    let user_hook = home.join(YAZELIX_NU_HOOK_RELATIVE);
    let hook_text = read_utf8(&user_hook, "editable Yazelix Nushell hook")?;
    for line in hook_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
    {
        if ((line.starts_with("source") || line.starts_with("use"))
            && line.contains("rtk_wrappers.nu"))
            || (line.starts_with("def --wrapped") && line.contains("^rtk"))
        {
            return Err(err(format!(
                "editable Yazelix Nushell hook must not duplicate packaged RTK routing: {}",
                user_hook.display()
            )));
        }
    }
    Ok(())
}

fn require_regular_file(path: &Path, label: &str) -> Result<()> {
    let metadata = fs::metadata(path).map_err(|source| {
        err(format!(
            "{label} is unavailable at {}: {source}",
            path.display()
        ))
    })?;
    if !metadata.is_file() {
        return Err(err(format!(
            "{label} must be a regular file at {}",
            path.display()
        )));
    }
    Ok(())
}

fn read_utf8(path: &Path, label: &str) -> Result<String> {
    fs::read_to_string(path).map_err(|source| {
        err(format!(
            "{label} is unreadable at {}: {source}",
            path.display()
        ))
    })
}

fn contains_active_line(text: &str, expected: &str) -> bool {
    text.lines()
        .map(str::trim)
        .any(|line| !line.starts_with('#') && line == expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "{prefix}-{}-{nonce}-{sequence}",
            std::process::id()
        ))
    }

    #[cfg(unix)]
    fn fixture() -> (PathBuf, PathBuf, PathBuf) {
        let root = temp_dir("agent-env-yazelix-runtime");
        let home = root.join("home");
        let store = root.join("nix/store");
        let profile_root = store.join("fixture-profile");
        fs::create_dir_all(profile_root.join("toolbin")).unwrap();
        fs::create_dir_all(profile_root.join("bin")).unwrap();
        fs::create_dir_all(profile_root.join("nushell/config")).unwrap();
        fs::create_dir_all(home.join(".config/nushell")).unwrap();
        fs::create_dir_all(home.join(".config/yazelix")).unwrap();
        fs::write(profile_root.join("toolbin/nu"), "nu").unwrap();
        fs::write(profile_root.join("bin/rtk"), "rtk").unwrap();
        symlink("../bin/rtk", profile_root.join("toolbin/rtk")).unwrap();
        fs::write(
            profile_root.join(RTK_MODULE_RELATIVE),
            "export def --wrapped codex [...rest] { ^rtk codex ...$rest }\nexport def --wrapped cargo [...rest] { ^rtk cargo ...$rest }\n",
        )
        .unwrap();
        fs::write(
            profile_root.join(PROFILE_NU_CONFIG_RELATIVE),
            "use rtk_wrappers.nu *\n",
        )
        .unwrap();
        fs::write(
            profile_root.join("manifest.json"),
            serde_json::json!({
                "version": 3,
                "elements": {
                    FOUNDATION_ELEMENT: {
                        "active": true,
                        "priority": FOUNDATION_PRIORITY,
                        "attrPath": "packages.x86_64-linux.lifeos_foundation_yzx",
                        "storePaths": [store.join("fixture-lifeos-foundation-yzx").to_string_lossy()],
                    }
                }
            })
            .to_string(),
        )
        .unwrap();
        symlink(&profile_root, home.join(".nix-profile-1-link")).unwrap();
        symlink(".nix-profile-1-link", home.join(FRONTDOOR_NAME)).unwrap();
        fs::write(
            home.join(HOST_NU_CONFIG_RELATIVE),
            "use ~/.nix-profile/nushell/config/rtk_wrappers.nu *\n",
        )
        .unwrap();
        fs::write(
            home.join(YAZELIX_NU_HOOK_RELATIVE),
            "def n8n-up [] { ^n8n-up }\n",
        )
        .unwrap();
        (root, home, store)
    }

    #[cfg(unix)]
    #[test]
    fn yazelix_contract_accepts_one_profile_and_native_nu_routing() {
        let (root, home, store) = fixture();
        validate_yazelix_nushell_at(&home, &store).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn yazelix_contract_rejects_parallel_profile_generation() {
        let (root, home, store) = fixture();
        symlink(
            store.join("fixture-profile"),
            home.join(".nix-profile-2-link"),
        )
        .unwrap();
        let error = validate_yazelix_nushell_at(&home, &store).unwrap_err();
        assert!(error
            .to_string()
            .contains("parallel Nix profile generation"));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn yazelix_contract_rejects_absolute_or_store_bypassing_frontdoors() {
        let (root, home, store) = fixture();
        fs::remove_file(home.join(FRONTDOOR_NAME)).unwrap();
        symlink(store.join("fixture-profile"), home.join(FRONTDOOR_NAME)).unwrap();
        let error = validate_yazelix_nushell_at(&home, &store).unwrap_err();
        assert!(error.to_string().contains("must name one direct"));
        fs::remove_file(home.join(FRONTDOOR_NAME)).unwrap();
        symlink("../outside", home.join(FRONTDOOR_NAME)).unwrap();
        let error = validate_yazelix_nushell_at(&home, &store).unwrap_err();
        assert!(error.to_string().contains("must name one direct"));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn yazelix_contract_rejects_selector_outside_the_profile_directory() {
        let (root, home, store) = fixture();
        let selector = home.join(PROFILE_RELATIVE);
        fs::remove_file(&selector).unwrap();
        symlink("../outside", &selector).unwrap();
        let error = validate_yazelix_nushell_at(&home, &store).unwrap_err();
        assert!(error
            .to_string()
            .contains("must name one direct .nix-profile-N-link generation"));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn yazelix_contract_rejects_invalid_generation_and_foundation_manifest_variants() {
        let (root, home, store) = fixture();
        let selector = home.join(PROFILE_RELATIVE);
        fs::remove_file(&selector).unwrap();
        fs::remove_file(home.join(".nix-profile-1-link")).unwrap();
        symlink(".nix-profile-99-link", &selector).unwrap();
        symlink(
            store.join("not-a-generation"),
            home.join(".nix-profile-99-link"),
        )
        .unwrap();
        let error = validate_yazelix_nushell_at(&home, &store).unwrap_err();
        assert!(error.to_string().contains("must resolve directly under"));
        fs::remove_file(&selector).unwrap();
        fs::remove_file(home.join(".nix-profile-99-link")).unwrap();
        symlink(
            store.join("fixture-profile"),
            home.join(".nix-profile-1-link"),
        )
        .unwrap();
        symlink(".nix-profile-1-link", &selector).unwrap();

        let profile_root = fs::canonicalize(home.join(FRONTDOOR_NAME)).unwrap();
        let manifest = profile_root.join("manifest.json");
        for invalid in [
            serde_json::json!({"version": 3, "elements": {}}),
            serde_json::json!({"version": 3, "elements": { FOUNDATION_ELEMENT: {
                "active": true, "priority": 3,
                "attrPath": "packages.x86_64-linux.lifeos_foundation_yzx",
                "storePaths": [store.join("fixture-lifeos-foundation-yzx").to_string_lossy()],
            }}}),
            serde_json::json!({"version": 3, "elements": { FOUNDATION_ELEMENT: {
                "active": true, "priority": FOUNDATION_PRIORITY,
                "attrPath": "legacy.lifeos_foundation_yzx",
                "storePaths": [store.join("fixture-lifeos-foundation-yzx").to_string_lossy()],
            }}}),
            serde_json::json!({"version": 3, "elements": { FOUNDATION_ELEMENT: {
                "active": true, "priority": FOUNDATION_PRIORITY,
                "attrPath": "packages.x86_64-linux.lifeos_foundation_yzx",
                "storePaths": [
                    store.join("fixture-lifeos-foundation-yzx").to_string_lossy(),
                    store.join("another-lifeos-foundation-yzx").to_string_lossy(),
                ],
            }}}),
        ] {
            fs::write(&manifest, invalid.to_string()).unwrap();
            assert!(validate_yazelix_nushell_at(&home, &store).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn yazelix_contract_rejects_duplicate_user_hook_wrapper() {
        let (root, home, store) = fixture();
        fs::write(
            home.join(YAZELIX_NU_HOOK_RELATIVE),
            "def --wrapped cargo [...rest] { ^rtk cargo ...$rest }\n",
        )
        .unwrap();
        let error = validate_yazelix_nushell_at(&home, &store).unwrap_err();
        assert!(error
            .to_string()
            .contains("must not duplicate packaged RTK routing"));
        fs::remove_dir_all(root).unwrap();
    }
}
