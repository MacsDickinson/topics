//! Direct-mode Codex callbacks are only a transport adapter into the SAME SDK
//! server. This module has no HostTools/store/approval capability.
use crate::{
    bridge::Bridge,
    diagnostics::Failure,
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
        self.call_inner(tool, arguments)
            .await
            .map_err(|e| Failure::McpRequest.tag(e))
    }
    async fn call_inner(&self, tool: &str, arguments: Value) -> WorkspaceResult<Value> {
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
        .map_err(|_| {
            Failure::McpDeadline.error("runtime_unavailable", "Topics MCP request timed out")
        })?
        .map_err(|_| {
            Failure::McpRequest.error(
                "forbidden",
                "Topics tool request denied; review and reconfirm the selected sources, then retry",
            )
        })?;
        if result.is_error == Some(true) {
            return Err(Failure::McpRequest.error(
                "forbidden",
                "Topics tool request denied; review and reconfirm the selected sources, then retry",
            ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::{ErrorData, RoleServer, ServerHandler, service::RequestContext};
    struct StalledSdk;
    impl ServerHandler for StalledSdk {
        fn get_info(&self) -> ServerConfig {
            ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
                .with_protocol_version(ProtocolVersion::V_2025_06_18)
        }
        async fn call_tool(
            &self,
            _: CallToolRequestParams,
            _: RequestContext<RoleServer>,
        ) -> Result<CallToolResponse, ErrorData> {
            std::future::pending().await
        }
    }
    #[tokio::test]
    async fn actual_sdk_deadline_is_distinct_from_host_deadline_without_raw_error_forwarding() {
        let (server, client) = tokio::io::duplex(MAX_FRAME);
        let (read, write) = tokio::io::split(server);
        let server = tokio::spawn(async move {
            let service = StalledSdk
                .serve(rmcp::transport::async_rw::AsyncRwTransport::<
                    RoleServer,
                    _,
                    _,
                >::new(read, write))
                .await
                .unwrap();
            let _ = service.waiting().await;
        });
        let (read, write) = tokio::io::split(client);
        let mut config = ClientConfig::new(
            ClientCapabilities::default(),
            Implementation::new("synthetic", "1"),
        );
        config.protocol_version = ProtocolVersion::V_2025_06_18;
        let service = config
            .serve(rmcp::transport::async_rw::AsyncRwTransport::<
                RoleClient,
                _,
                _,
            >::new(read, write))
            .await
            .unwrap();
        let client = McpClient { service };
        let failure = tokio::time::timeout(
            Duration::from_secs(8),
            client.call("scope_sources", json!({"scope_id":"synthetic"})),
        )
        .await
        .unwrap()
        .unwrap_err();
        assert_eq!(failure.details, Some(Failure::McpDeadline.details()));
        server.abort();
        let _ = server.await;
    }
}
