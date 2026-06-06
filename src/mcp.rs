use std::io::{self, BufRead, Write};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::app;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

pub fn run_stdio() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        if let Some(response) = handle_json_line(&line) {
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }
    }

    Ok(())
}

fn handle_json_line(line: &str) -> Option<JsonRpcResponse> {
    let request: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(request) => request,
        Err(error) => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0",
                id: None,
                result: None,
                error: Some(JsonRpcError {
                    code: -32700,
                    message: format!("parse error: {error}"),
                }),
            });
        }
    };

    request.id.as_ref()?;

    match request.method.as_str() {
        "initialize" => Some(JsonRpcResponse {
            jsonrpc: "2.0",
            id: request.id,
            result: Some(initialize_result()),
            error: None,
        }),
        "tools/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0",
            id: request.id,
            result: Some(tools_list_result()),
            error: None,
        }),
        "tools/call" => Some(handle_tool_call(request.id, request.params)),
        _ => Some(JsonRpcResponse {
            jsonrpc: "2.0",
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("method not found: {}", request.method),
            }),
        }),
    }
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "tty-pet-mcp",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn handle_tool_call(id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
    let Some(name) = params
        .as_ref()
        .and_then(|params| params.get("name"))
        .and_then(Value::as_str)
    else {
        return invalid_params(id, "tools/call requires params.name");
    };

    match name {
        "tty_pet_status" => match app::status_json_string() {
            Ok(status) => JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(text_content_result(status)),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: error.to_string(),
                }),
            },
        },
        "tty_pet_event" => {
            let Some(kind) = params
                .as_ref()
                .and_then(|params| params.get("arguments"))
                .and_then(|arguments| arguments.get("kind"))
                .and_then(Value::as_str)
            else {
                return invalid_params(id, "tty_pet_event requires arguments.kind");
            };

            match app::record_agent_event(kind) {
                Ok(message) => JsonRpcResponse {
                    jsonrpc: "2.0",
                    id,
                    result: Some(text_content_result(message)),
                    error: None,
                },
                Err(error) => invalid_params(id, &error.to_string()),
            }
        }
        _ => invalid_params(id, &format!("unknown tool: {name}")),
    }
}

fn invalid_params(id: Option<Value>, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(JsonRpcError {
            code: -32602,
            message: message.to_string(),
        }),
    }
}

fn text_content_result(text: String) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ]
    })
}

fn tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "tty_pet_status",
                "description": "Return tty-pet project state as JSON.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }
            },
            {
                "name": "tty_pet_event",
                "description": "Record a safe tty-pet event for the current project.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "kind": {
                            "type": "string",
                            "enum": ["poke", "treat", "call", "nap", "pass", "fail"]
                        }
                    },
                    "required": ["kind"],
                    "additionalProperties": false
                }
            }
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_method_returns_json_rpc_error() {
        let response = handle_json_line(r#"{"jsonrpc":"2.0","id":1,"method":"nope"}"#);

        assert_eq!(
            response
                .expect("response expected")
                .error
                .expect("error expected")
                .code,
            -32601
        );
    }
}
