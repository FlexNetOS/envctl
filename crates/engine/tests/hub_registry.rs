use envctl_engine::{hub_registry, HubRegistryStatus, Registry};
use std::path::PathBuf;

fn temp_root(slug: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "envctl-hub-registry-{slug}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_component(dir: &PathBuf, id: &str) {
    let text = format!(
        r#"
[[component]]
id = "{id}"
name = "{id}"
description = "{id}"

[component.detect]
kind = "command"
command = "true"
"#
    );
    std::fs::write(dir.join(format!("{id}.toml")), text).unwrap();
}

#[test]
fn hub_registry_loads_and_reconciles_components() {
    let root = temp_root("clean");
    let manifest = root.join("manifest");
    let mcp_hub = root.join("mcp_hub");
    std::fs::create_dir_all(&manifest).unwrap();
    std::fs::create_dir_all(mcp_hub.join("entries")).unwrap();
    write_component(&manifest, "n8n-mcp");
    write_component(&manifest, "kasetto");
    std::fs::write(
        mcp_hub.join("registry.json"),
        r#"
{
  "schema": "hub.registry.v1",
  "entries": [
    {
      "id": "n8n-mcp",
      "name": "n8n MCP server",
      "description": "docs-only",
      "component": "n8n-mcp",
      "status": "stable",
      "tier": 1
    },
    {
      "id": "playwright-mcp",
      "name": "Playwright MCP server",
      "description": "fallback",
      "component": "kasetto",
      "status": "experimental",
      "tier": 2
    }
  ]
}
"#,
    )
    .unwrap();

    let reg = Registry::load(&manifest).unwrap();
    let report = hub_registry::load(&root, &reg).unwrap();
    assert!(report.clean(), "unexpected drift: {report:#?}");
    assert_eq!(report.sources.len(), 1);
    assert_eq!(report.entries.len(), 2);
    assert_eq!(report.entries[0].entry.status, HubRegistryStatus::Stable);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn hub_registry_flags_missing_component_as_drift() {
    let root = temp_root("drift");
    let manifest = root.join("manifest");
    let mcp_hub = root.join("mcp_hub");
    std::fs::create_dir_all(&manifest).unwrap();
    std::fs::create_dir_all(mcp_hub.join("entries")).unwrap();
    write_component(&manifest, "n8n-mcp");
    std::fs::write(
        mcp_hub.join("registry.json"),
        r#"
{
  "schema": "hub.registry.v1",
  "entries": [
    {
      "id": "playwright-mcp",
      "name": "Playwright MCP server",
      "description": "fallback",
      "component": "kasetto",
      "status": "stable",
      "tier": 2
    }
  ]
}
"#,
    )
    .unwrap();

    let reg = Registry::load(&manifest).unwrap();
    let report = hub_registry::load(&root, &reg).unwrap();
    assert!(!report.clean(), "missing component should be drift");
    assert!(
        report.drift.iter().any(|item| item.component == "kasetto"),
        "expected kasetto drift: {report:#?}"
    );

    let _ = std::fs::remove_dir_all(root);
}
