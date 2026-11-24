//! User-configured MCP stdio transport. The process arguments bind the root
//! and topic allowlist. Tool arguments cannot widen either.
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::io::{self, BufRead, Write};
use topics_core::{RetrievalSearchRequest, RetrievalTopicRequest, validate_workspace_id};
use topics_store::{WorkspaceService, application_data_dir};

pub(super) fn serve(
    mut args: impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = args
        .next()
        .ok_or("Usage: topics mcp ROOT_ID TOPIC_ID [TOPIC_ID ...]")?;
    validate_workspace_id(&root)?;
    let topics: BTreeSet<_> = args.collect();
    if topics.is_empty() {
        return Err("Configure at least one topic ID for this client".into());
    }
    for id in &topics {
        validate_workspace_id(id)?;
    }
    let service = WorkspaceService::open(application_data_dir()?)?;
    let mut phase = Phase::New;
    let mut output = io::stdout().lock();
    for line in io::stdin().lock().lines() {
        let line = line?;
        let response = match serde_json::from_str::<Value>(&line) {
            Err(_) => Some(
                json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Invalid JSON"}}),
            ),
            Ok(req) => dispatch(&service, &root, &topics, &mut phase, req),
        };
        if let Some(response) = response {
            writeln!(output, "{response}")?;
            output.flush()?;
        }
    }
    Ok(())
}

#[derive(PartialEq)]
enum Phase {
    New,
    Negotiated,
    Ready,
}

fn dispatch(
    service: &WorkspaceService,
    root: &str,
    allowed: &BTreeSet<String>,
    phase: &mut Phase,
    req: Value,
) -> Option<Value> {
    let id = req.get("id");
    let error = |code: i32, message: &str| {
        Some(json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}}))
    };
    let method = req["method"].as_str();
    if req["jsonrpc"] != "2.0"
        || method.is_none()
        || id.is_some_and(|id| !(id.is_string() || id.is_i64() || id.is_u64()))
    {
        return error(-32600, "Invalid request");
    }
    if id.is_none() {
        if method == Some("notifications/initialized") && *phase == Phase::Negotiated {
            *phase = Phase::Ready;
        }
        return None;
    }
    let result = match method? {
        "ping" => json!({}),
        "initialize" if *phase == Phase::New => {
            if !req["params"]["protocolVersion"].is_string() {
                return error(-32602, "Missing protocol version");
            }
            *phase = Phase::Negotiated;
            json!({"protocolVersion":"2025-06-18","serverInfo":{"name":"topics-retrieval","version":env!("CARGO_PKG_VERSION")},"capabilities":{"tools":{}}})
        }
        _ if *phase != Phase::Ready => return error(-32600, "Initialize the server first"),
        "tools/list" => json!({"tools":[
            {"name":"topic_search","description":"Find approved topics in the configured scope by name, alias or assertion.","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer","minimum":1,"maximum":50}},"required":["query"],"additionalProperties":false}},
            {"name":"topic_detail","description":"Read approved assertions and typed relationships for a stable topic ID.","inputSchema":{"type":"object","properties":{"topic_id":{"type":"string"}},"required":["topic_id"],"additionalProperties":false}}
        ]}),
        "tools/call" => {
            let args = &req["params"]["arguments"];
            let value = match req["params"]["name"].as_str() {
                Some("topic_search") => {
                    let Some(query) = args["query"].as_str() else {
                        return error(-32602, "A query is required");
                    };
                    if !args
                        .as_object()?
                        .keys()
                        .all(|k| k == "limit" || k == "query")
                        || args
                            .get("limit")
                            .is_some_and(|n| !n.as_u64().is_some_and(|n| (1..=50).contains(&n)))
                    {
                        return error(-32602, "Invalid search arguments");
                    }
                    service
                        .retrieval_search(
                            root,
                            allowed,
                            RetrievalSearchRequest {
                                query: query.into(),
                                limit: args["limit"].as_u64().map(|n| n as u32),
                            },
                        )
                        .map(|r| json!(r))
                }
                Some("topic_detail") => {
                    let Ok(request) = serde_json::from_value::<RetrievalTopicRequest>(args.clone())
                    else {
                        return error(-32602, "Expected one topic_id");
                    };
                    service.retrieval_topic(root, allowed, request).map(|r| json!({"topic":r.topic,"assertions":r.assertions,"relationships":r.relationships,"checks":r.checks}))
                }
                _ => return error(-32602, "Unknown tool"),
            };
            match value {
                Ok(value) => json!({"content":[{"type":"text","text":value.to_string()}]}),
                Err(_) => {
                    json!({"isError":true,"content":[{"type":"text","text":"Knowledge is unavailable for this client."}]})
                }
            }
        }
        _ => return error(-32601, "Unknown method"),
    };
    Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
}
