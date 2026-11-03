use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};
/// Entry point used before desktop startup when --topics-mcp-proxy is present.
/// Returns a process exit status and never writes diagnostics to MCP stdout.
pub fn run_stdio_proxy() -> i32 {
    let Ok(rt) = tokio::runtime::Runtime::new() else {
        return 1;
    };
    let result = rt
        .block_on(async { tokio::time::timeout(std::time::Duration::from_secs(180), run()).await });
    // Tokio stdin uses a blocking reader that cannot be cancelled. Never wait
    // for that reader when the authenticated endpoint or deadline closes.
    rt.shutdown_background();
    match result {
        Ok(Some(())) => 0,
        _ => 1,
    }
}
async fn run() -> Option<()> {
    let endpoint = std::env::var("TOPICS_MCP_ENDPOINT").ok()?;
    if !endpoint.starts_with("127.0.0.1:") {
        return None::<()>;
    }
    let session = std::env::var("TOPICS_MCP_SESSION").ok()?;
    let token = std::env::var("TOPICS_MCP_TOKEN").ok()?;
    let mut stream = TcpStream::connect(endpoint).await.ok()?;
    let mut auth =
        serde_json::to_vec(&serde_json::json!({"session":session,"token":token,"nonce":uuid::Uuid::new_v4().to_string()})).ok()?;
    auth.push(b'\n');
    stream.write_all(&auth).await.ok()?;
    let mut reader = BufReader::new(stream);
    if crate::transport::read_frame(&mut reader, 16).await.ok()? != b"ok\n" {
        return None::<()>;
    }
    let (read, mut write) = reader.into_inner().into_split();
    let upload = async {
        let mut input = BufReader::new(tokio::io::stdin());
        let mut bytes = 0;
        loop {
            let line = crate::transport::read_frame(&mut input, crate::protocol::MAX_FRAME)
                .await
                .ok()?;
            bytes += line.len();
            if bytes > crate::protocol::MAX_OUTPUT {
                return None::<()>;
            }
            write.write_all(&line).await.ok()?;
        }
    };
    let download = async {
        let mut read = BufReader::new(read);
        let mut output = tokio::io::stdout();
        let mut bytes = 0;
        loop {
            let line = crate::transport::read_frame(&mut read, crate::protocol::MAX_FRAME)
                .await
                .ok()?;
            bytes += line.len();
            if bytes > crate::protocol::MAX_OUTPUT {
                return None::<()>;
            }
            output.write_all(&line).await.ok()?;
            output.flush().await.ok()?;
        }
    };
    tokio::select! { _=upload=>{},_=download=>{} }
    Some(())
}
