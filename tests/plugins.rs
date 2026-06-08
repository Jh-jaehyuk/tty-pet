use std::path::PathBuf;

use serde_json::Value;

#[test]
fn codex_plugin_bundle_declares_tty_pet_mcp_server() {
    let plugin_root = repo_root().join("plugins/codex/tty-pet");
    let manifest = read_json(plugin_root.join(".codex-plugin/plugin.json"));

    assert_eq!(manifest["name"], "tty-pet");
    assert_eq!(manifest["skills"], "./skills/");
    assert_eq!(manifest["mcpServers"], "./.mcp.json");

    let mcp = read_json(plugin_root.join(".mcp.json"));
    let server = &mcp["mcpServers"]["tty-pet"];

    assert_eq!(server["command"], "tty-pet-mcp");
    assert_eq!(server["args"], serde_json::json!(["--stdio"]));
}

#[test]
fn claude_plugin_bundle_declares_tty_pet_mcp_server() {
    let plugin_root = repo_root().join("plugins/claude-code/tty-pet");
    let manifest = read_json(plugin_root.join(".claude-plugin/plugin.json"));

    assert_eq!(manifest["name"], "tty-pet");
    assert_eq!(manifest["skills"], "./skills/");
    assert_eq!(manifest["mcpServers"], "./.mcp.json");

    let mcp = read_json(plugin_root.join(".mcp.json"));
    let server = &mcp["mcpServers"]["tty-pet"];

    assert_eq!(server["command"], "tty-pet-mcp");
    assert_eq!(server["args"], serde_json::json!(["--stdio"]));
}

#[test]
fn documentation_mentions_agent_plugin_bundles() {
    let readme = std::fs::read_to_string(repo_root().join("README.md"))
        .expect("README.md should be readable");
    let agent_doc = std::fs::read_to_string(repo_root().join("docs/AGENT_INTEGRATION.md"))
        .expect("agent integration doc should be readable");

    for expected in [
        "plugins/codex/tty-pet",
        "plugins/claude-code/tty-pet",
        "cargo install --git https://github.com/Jh-jaehyuk/tty-pet --locked",
        "tty-pet-mcp",
    ] {
        assert!(
            readme.contains(expected) || agent_doc.contains(expected),
            "documentation should mention {expected}"
        );
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(path: PathBuf) -> Value {
    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    serde_json::from_str(&contents)
        .unwrap_or_else(|error| panic!("{} should contain valid JSON: {error}", path.display()))
}
