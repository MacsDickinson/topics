//! Direct-mode Codex callbacks are only a transport adapter into the SAME SDK
//! server. This module has no HostTools/store/approval capability.
use crate::{
    bridge::Bridge,
    error,
    protocol::{MAX_FRAME, MAX_OUTPUT},
};
use rmcp::{RoleClient, ServiceExt, model::*, service::RunningService};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};
use topics_core::workspace::WorkspaceResult;
pub(crate) struct McpClient {
    service: RunningService<RoleClient, ClientConfig>,
}
impl McpClient {
    pub async fn connect(bridge: &Bridge) -> WorkspaceResult<Self> {
        let mut stream = TcpStream::connect(&bridge.endpoint)
            .await
            .map_err(|_| error("runtime_unavailable", "Topics MCP connection unavailable"))?;
        let bootstrap = json!({"session":bridge.session,"token":bridge.token,"nonce":uuid::Uuid::new_v4().to_string()});
        stream
            .write_all(format!("{bootstrap}\n").as_bytes())
            .await
            .map_err(|_| error("runtime_unavailable", "Topics MCP bootstrap unavailable"))?;
        let mut reader = BufReader::new(stream);
        if crate::transport::read_frame(&mut reader, 16)
            .await
            .map_err(|_| error("forbidden", "Topics MCP authentication failed"))?
            != b"ok\n"
        {
            return Err(error("forbidden", "Topics MCP authentication failed"));
        }
        let (read, write) = reader.into_inner().into_split();
        let transport =
            rmcp::transport::async_rw::AsyncRwTransport::<RoleClient, _, _>::new(read, write)
                .with_max_line_length(MAX_FRAME);
        let mut config = ClientConfig::new(
            ClientCapabilities::default(),
            Implementation::new("topics-codex-adapter", "0.1.0"),
        );
        config.protocol_version = ProtocolVersion::V_2025_06_18;
        let service = tokio::time::timeout(Duration::from_secs(5), config.serve(transport))
            .await
            .map_err(|_| error("runtime_unavailable", "Topics MCP initialization timeout"))?
            .map_err(|_| error("runtime_unavailable", "Topics MCP initialization failed"))?;
        Ok(Self { service })
    }
    pub async fn dynamic_tools(&self) -> WorkspaceResult<Value> {
        let result = tokio::time::timeout(Duration::from_secs(5), self.service.list_tools(None))
            .await
            .map_err(|_| error("runtime_unavailable", "Topics MCP inventory timeout"))?
            .map_err(|_| error("runtime_unavailable", "Topics MCP inventory failed"))?;
        if result.next_cursor.is_some() || result.tools.len() != 2 {
            return Err(error("forbidden", "Unexpected Topics MCP inventory"));
        }
        let expected = crate::bridge::tools();
        let mut tools = vec![];
        for tool in result.tools {
            let reference = expected
                .iter()
                .find(|known| known.name == tool.name)
                .ok_or_else(|| error("forbidden", "Unknown Topics MCP tool"))?;
            if tool.input_schema != reference.input_schema
                || tool.annotations != reference.annotations
            {
                return Err(error("forbidden", "Topics MCP schema drift"));
            }
            tools.push(json!({"type":"function","name":tool.name,"description":tool.description,"inputSchema":tool.input_schema}));
        }
        Ok(
            json!([{"type":"namespace","name":"topics","description":"Bounded tools from the Topics app-hosted MCP server; all changes are human-reviewed proposals","tools":tools}]),
        )
    }
    pub async fn call(&self, tool: &str, arguments: Value) -> WorkspaceResult<Value> {
        if !matches!(tool, "scope_sources" | "propose_changes") {
            return Err(error("forbidden", "Unknown Topics tool callback"));
        }
        let args = arguments
            .as_object()
            .cloned()
            .ok_or_else(|| error("invalid_input", "Invalid callback arguments"))?;
        let result = tokio::time::timeout(
            Duration::from_secs(6),
            self.service
                .call_tool(CallToolRequestParams::new(tool.to_owned()).with_arguments(args)),
        )
        .await
        .map_err(|_| error("runtime_unavailable", "Topics MCP request timed out"))?
        .map_err(|_| error("forbidden", "Topics MCP request denied"))?;
        if result.is_error == Some(true) {
            return Err(error("forbidden", "Topics MCP request denied"));
        }
        let mut output = vec![];
        let mut bytes = 0;
        for content in result.content {
            let value = serde_json::to_value(content)
                .map_err(|_| error("internal", "MCP output encoding failed"))?;
            if value["type"] != "text" {
                return Err(error("forbidden", "Unexpected MCP output kind"));
            }
            let text = value["text"]
                .as_str()
                .ok_or_else(|| error("forbidden", "Invalid MCP text output"))?;
            bytes += text.len();
            if bytes > MAX_OUTPUT / 2 {
                return Err(error("forbidden", "MCP output exceeded bounds"));
            }
            output.push(json!({"type":"inputText","text":text}));
        }
        Ok(json!({"success":true,"contentItems":output}))
    }
}
