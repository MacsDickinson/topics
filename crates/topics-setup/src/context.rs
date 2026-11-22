//! Pack optional completed draft context after exact mandatory delivery costs.
//! Canonical records are retained whole; neither source bytes nor accepted
//! correction context are trimmed. This is ephemeral model context, not storage.
use crate::error;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use topics_core::*;
const PARTIAL: &str = "Prior draft context is partial to fit this bounded response. Omitted context is not negative evidence; reuse supplied IDs and cite only supplied passages.";

fn fits(response: &Value, schema: &Value) -> bool {
    let mut wire = response.clone();
    wire["candidate_schema"] = schema.clone(); // The runtime bridge adds this.
    let text = wire.to_string();
    if text.len() > topics_runtime::protocol::MAX_FRAME / 2 {
        return false;
    }
    // Include text escaping and representative bounded envelope identities in
    // both SDK and direct-mode response frames; no runtime limit is enlarged.
    let sdk = json!({"jsonrpc":"2.0","id":u64::MAX,"result":{"content":[{"type":"text","text":text}],"isError":false}});
    let direct = json!({"id":"x".repeat(256),"result":{"success":true,"contentItems":[{"type":"inputText","text":text}]}});
    sdk.to_string().len() < topics_runtime::protocol::MAX_FRAME
        && direct.to_string().len() < topics_runtime::protocol::MAX_FRAME
}
fn try_context(
    response: &mut Value,
    context: &SetupDraftContext,
    schema: &Value,
) -> WorkspaceResult<bool> {
    response["draft_context"] = serde_json::to_value(context)
        .map_err(|_| error("internal", "Could not encode scoped draft context"))?;
    Ok(fits(response, schema))
}
fn claim_sources(claim: &Claim, ids: &mut BTreeSet<Id>) {
    for evidence in &claim.sources {
        if let Evidence::Passage { source } = evidence {
            ids.insert(source.source_id.clone());
        }
    }
}
pub(super) fn response(
    sources: Vec<ScopedSource>,
    accepted: Vec<TopicView>,
    draft: SetupDraftContext,
) -> WorkspaceResult<Value> {
    let tool = topics_runtime::bridge::tools()
        .into_iter()
        .find(|tool| tool.name == "propose_changes")
        .ok_or_else(|| error("internal", "Runtime candidate schema unavailable"))?;
    let schema = Value::Object(tool.input_schema.as_ref().clone());
    let mut response = json!({"sources":sources,"topic_contexts":accepted,"draft_context":draft});
    if fits(&response, &schema) {
        return Ok(response);
    }
    let mut packed = SetupDraftContext {
        topics: vec![],
        claims: vec![],
        connections: vec![],
    };
    response["context_notice"] = Value::String(PARTIAL.into());
    if !try_context(&mut response, &packed, &schema)? {
        return Err(error(
            "invalid_input",
            "Selected sources and accepted context exceed the runtime payload bound; preview smaller batches",
        ));
    }
    // Keep canonical identity hints first, then only records whose dependencies
    // remain in this reply. A record and all of its citations stay unchanged.
    for topic in draft.topics {
        packed.topics.push(topic);
        if !try_context(&mut response, &packed, &schema)? {
            packed.topics.pop();
        }
    }
    let mut topics: BTreeSet<_> = packed.topics.iter().map(|topic| topic.id.clone()).collect();
    let mut claims = BTreeSet::new();
    let mut source_ids: BTreeSet<_> = sources
        .iter()
        .map(|source| source.summary.source_id.clone())
        .collect();
    for view in &accepted {
        topics.insert(view.topic.id.clone());
        for claim in view.current_claims.iter().chain(&view.history) {
            claims.extend([claim.id.clone(), claim.revision_id.clone()]);
            claim_sources(claim, &mut source_ids);
        }
        for connection in &view.connections {
            source_ids.extend(
                connection
                    .sources
                    .iter()
                    .map(|source| source.source_id.clone()),
            );
        }
    }
    for claim in draft.claims {
        if !topics.contains(&claim.subject_id) {
            continue;
        }
        packed.claims.push(claim);
        if try_context(&mut response, &packed, &schema)? {
            let claim = packed
                .claims
                .last()
                .ok_or_else(|| error("internal", "Scoped claim unavailable"))?;
            claims.extend([claim.id.clone(), claim.revision_id.clone()]);
            claim_sources(claim, &mut source_ids);
        } else {
            packed.claims.pop();
        }
    }
    for connection in draft.connections {
        let scoped = |endpoint: &Endpoint| match endpoint {
            Endpoint::Topic { id } => topics.contains(id),
            Endpoint::Claim { id } => claims.contains(id),
            Endpoint::Source { source_id, .. } => {
                source_ids.contains(source_id)
                    || connection
                        .sources
                        .iter()
                        .any(|source| &source.source_id == source_id)
            }
        };
        if !scoped(&connection.from) || !scoped(&connection.to) {
            continue;
        }
        packed.connections.push(connection);
        if !try_context(&mut response, &packed, &schema)? {
            packed.connections.pop();
        }
    }
    if !try_context(&mut response, &packed, &schema)? {
        return Err(error(
            "internal",
            "Packed context exceeded its fixed budget",
        ));
    }
    Ok(response)
}
