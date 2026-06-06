use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

#[test]
fn status_json_reports_project_state_for_agents() {
    let term_pet_home = unique_temp_dir("status-json");
    let output = Command::new(env!("CARGO_BIN_EXE_tty-pet"))
        .args(["status", "--json"])
        .env("TERM_PET_HOME", &term_pet_home)
        .output()
        .expect("status command should run");

    assert!(
        output.status.success(),
        "status command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value =
        serde_json::from_slice(&output.stdout).expect("status --json should print valid JSON");

    assert_eq!(value["project"]["name"], "tty-pet");
    assert_eq!(value["state"]["bond"], 0);
    assert_eq!(value["pet"]["image"]["kind"], "built-in");
    assert!(value["debug"]["database_path"]
        .as_str()
        .expect("database path should be a string")
        .contains("tty-pet-status-json"));
}

#[test]
fn status_json_can_resolve_project_from_agent_env() {
    let term_pet_home = unique_temp_dir("agent-env");
    let project_dir = unique_temp_dir("agent-project");
    std::fs::create_dir_all(&project_dir).expect("project dir should be created");

    let output = Command::new(env!("CARGO_BIN_EXE_tty-pet"))
        .args(["status", "--json"])
        .env("TERM_PET_HOME", &term_pet_home)
        .env("TTY_PET_PROJECT_DIR", &project_dir)
        .output()
        .expect("status command should run");

    assert!(
        output.status.success(),
        "status command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value =
        serde_json::from_slice(&output.stdout).expect("status --json should print valid JSON");

    let expected_root = std::fs::canonicalize(&project_dir)
        .expect("project dir should canonicalize")
        .display()
        .to_string();

    assert_eq!(value["project"]["root_path"], expected_root);
}

#[test]
fn mcp_tools_list_exposes_tty_pet_tools() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tty-pet-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("mcp server should start");

    {
        let stdin = child.stdin.as_mut().expect("stdin should be available");
        stdin
            .write_all(br#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#)
            .expect("request should write");
        stdin.write_all(b"\n").expect("newline should write");
    }

    let output = child
        .wait_with_output()
        .expect("mcp server should exit after stdin closes");

    assert!(
        output.status.success(),
        "mcp server failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value =
        serde_json::from_slice(&output.stdout).expect("mcp response should be valid JSON");
    let tools = value["result"]["tools"]
        .as_array()
        .expect("tools/list should return tools array");
    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();

    assert!(tool_names.contains(&"tty_pet_status"));
    assert!(tool_names.contains(&"tty_pet_event"));
}

#[test]
fn mcp_initialize_reports_server_capabilities() {
    let term_pet_home = unique_temp_dir("mcp-init");
    let response = run_mcp_request(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.0.0"}}}"#,
        &term_pet_home,
    );

    assert_eq!(response["result"]["serverInfo"]["name"], "tty-pet-mcp");
    assert_eq!(
        response["result"]["capabilities"]["tools"],
        serde_json::json!({})
    );
}

#[test]
fn mcp_notifications_do_not_emit_responses() {
    let term_pet_home = unique_temp_dir("mcp-notification");
    let output = run_mcp_raw(
        r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#,
        &term_pet_home,
    );

    assert!(
        output.trim().is_empty(),
        "notification should not receive response, got: {output}"
    );
}

#[test]
fn mcp_status_tool_returns_tty_pet_status_json() {
    let term_pet_home = unique_temp_dir("mcp-status");
    let response = run_mcp_request(
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"tty_pet_status","arguments":{}}}"#,
        &term_pet_home,
    );

    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("status tool should return text content");
    let status: Value = serde_json::from_str(text).expect("tool text should be status JSON");

    assert_eq!(status["project"]["name"], "tty-pet");
    assert_eq!(status["pet"]["image"]["kind"], "built-in");
    assert!(status["debug"]["database_path"]
        .as_str()
        .expect("database path should be a string")
        .contains("tty-pet-mcp-status"));
}

#[test]
fn mcp_event_tool_records_safe_pet_event() {
    let term_pet_home = unique_temp_dir("mcp-event");
    let response = run_mcp_request(
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"tty_pet_event","arguments":{"kind":"treat"}}}"#,
        &term_pet_home,
    );

    assert_eq!(response["error"], Value::Null);
    assert!(response["result"]["content"][0]["text"]
        .as_str()
        .expect("event tool should return text content")
        .contains("treat"));

    let output = Command::new(env!("CARGO_BIN_EXE_tty-pet"))
        .args(["status", "--json"])
        .env("TERM_PET_HOME", &term_pet_home)
        .output()
        .expect("status command should run");
    let status: Value = serde_json::from_slice(&output.stdout).expect("status should be JSON");

    assert_eq!(status["state"]["bond"], 2);
    assert_eq!(status["state"]["last_event"]["kind"], "treat");
}

fn unique_temp_dir(name: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join(format!("tty-pet-{name}-{nanos}"))
        .to_string_lossy()
        .into_owned()
}

fn run_mcp_request(request: &str, term_pet_home: &str) -> Value {
    let stdout = run_mcp_raw(request, term_pet_home);

    serde_json::from_str(&stdout).expect("mcp response should be valid JSON")
}

fn run_mcp_raw(request: &str, term_pet_home: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tty-pet-mcp"))
        .env("TERM_PET_HOME", term_pet_home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("mcp server should start");

    {
        let stdin = child.stdin.as_mut().expect("stdin should be available");
        stdin
            .write_all(request.as_bytes())
            .expect("request should write");
        stdin.write_all(b"\n").expect("newline should write");
    }

    let output = child
        .wait_with_output()
        .expect("mcp server should exit after stdin closes");

    assert!(
        output.status.success(),
        "mcp server failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("mcp stdout should be utf8")
}
