//! Bounded NDJSON process transport; no jsonrpc field on the app-server wire.
use crate::{
    diagnostics::Failure,
    error,
    protocol::{MAX_FRAME, MAX_OUTPUT},
};
use serde_json::{Value, json};
use std::{collections::VecDeque, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};
use topics_core::workspace::WorkspaceResult;

pub async fn read_frame<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    limit: usize,
) -> std::io::Result<Vec<u8>> {
    let mut line = Vec::new();
    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            return Err(std::io::Error::new(
                if line.is_empty() {
                    std::io::ErrorKind::UnexpectedEof
                } else {
                    std::io::ErrorKind::InvalidData
                },
                "runtime exited or returned a truncated frame",
            ));
        }
        let count = available
            .iter()
            .position(|b| *b == b'\n')
            .map_or(available.len(), |i| i + 1);
        if line.len() + count > limit {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "frame too large",
            ));
        }
        line.extend_from_slice(&available[..count]);
        reader.consume(count);
        if line.last() == Some(&b'\n') {
            return Ok(line);
        }
    }
}
pub(crate) struct Process {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    stderr: tokio::task::JoinHandle<()>,
    next: u64,
    bytes: usize,
    pub queued: VecDeque<Value>,
    pub thread: Option<String>,
    pub turn: Option<String>,
    pub callbacks_allowed: bool,
}
impl Process {
    pub fn spawn(mut command: Command) -> WorkspaceResult<Self> {
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command.spawn().map_err(|_| {
            Failure::Transport.error(
                "runtime_unavailable",
                "Cannot start pinned Codex; configure codex-cli 0.166.0",
            )
        })?;
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut stderr_pipe = child.stderr.take().unwrap();
        // Drain without retaining sensitive diagnostics. Prevent full-pipe deadlocks.
        let stderr = tokio::spawn(async move {
            let mut buf = [0; 4096];
            let mut total = 0;
            while let Ok(n) = stderr_pipe.read(&mut buf).await {
                if n == 0 {
                    break;
                }
                total += n;
                if total > MAX_OUTPUT {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input,
            output,
            stderr,
            next: 1,
            bytes: 0,
            queued: VecDeque::new(),
            thread: None,
            turn: None,
            callbacks_allowed: false,
        })
    }
    pub async fn send(&mut self, value: Value) -> WorkspaceResult<()> {
        let mut frame = serde_json::to_vec(&value)
            .map_err(|_| error("internal", "Cannot encode runtime frame"))?;
        if frame.len() > MAX_FRAME {
            return Err(error("invalid_input", "Runtime request exceeds bounds"));
        }
        frame.push(b'\n');
        self.input
            .write_all(&frame)
            .await
            .map_err(|_| Failure::Transport.error("runtime_unavailable", "Codex disconnected"))
    }
    pub async fn next(&mut self) -> WorkspaceResult<Value> {
        self.next_inner()
            .await
            .map_err(|e| Failure::Protocol.tag(e))
    }
    async fn next_inner(&mut self) -> WorkspaceResult<Value> {
        let frame = tokio::time::timeout(
            Duration::from_secs(40),
            read_frame(&mut self.output, MAX_FRAME),
        )
        .await
        .map_err(|_| {
            Failure::ReadDeadline.error("runtime_unavailable", "Codex response timed out")
        })?
        .map_err(|_| {
            Failure::Transport.error(
                "runtime_unavailable",
                "Codex exited or returned an oversized frame",
            )
        })?;
        self.bytes += frame.len();
        if self.bytes > MAX_OUTPUT {
            return Err(error("runtime_unavailable", "Codex output exceeded bounds"));
        }
        let value: Value = serde_json::from_slice(&frame).map_err(|_| {
            error(
                "runtime_unavailable",
                "Malformed Codex frame; update required",
            )
        })?;
        if !value.is_object() || value.get("jsonrpc").is_some() {
            return Err(error(
                "runtime_unavailable",
                "Codex protocol schema mismatch",
            ));
        }
        if value.get("id").is_some() && value.get("method").is_some() {
            if self.callbacks_allowed
                && value["method"] == "item/tool/call"
                && value
                    .get("id")
                    .is_some_and(|id| id.is_string() || id.as_i64().is_some())
            {
                return Ok(value);
            }
            self.send(json!({"id":value["id"],"error":{"code":-32601,"message":"Topics denies runtime-initiated requests"}})).await?;
            return Err(error(
                "forbidden",
                "Codex requested an unauthorized approval or capability",
            ));
        }
        Ok(value)
    }
    pub async fn request(&mut self, method: &str, params: Value) -> WorkspaceResult<Value> {
        self.request_inner(method, params)
            .await
            .map_err(|e| Failure::Protocol.tag(e))
    }
    async fn request_inner(&mut self, method: &str, params: Value) -> WorkspaceResult<Value> {
        let id = self.next;
        self.next += 1;
        self.send(json!({"id":id,"method":method,"params":params}))
            .await?;
        loop {
            let frame = self.next().await?;
            if frame.get("method").is_some() && frame.get("id").is_some() {
                if self.queued.len() >= 256 {
                    return Err(error("runtime_unavailable", "Excessive callback queue"));
                }
                self.queued.push_back(frame);
                continue;
            }
            if frame.get("id") == Some(&json!(id)) {
                if frame.get("error").is_some() {
                    return Err(error(
                        "runtime_unavailable",
                        "Codex rejected the pinned protocol request",
                    ));
                }
                return frame
                    .get("result")
                    .cloned()
                    .ok_or_else(|| error("runtime_unavailable", "Missing Codex result"));
            }
            if frame.get("id").is_some() {
                return Err(error("runtime_unavailable", "Unknown Codex response ID"));
            }
            crate::check_notification(&frame)?;
            if self.queued.len() >= 256 {
                return Err(error(
                    "runtime_unavailable",
                    "Excessive Codex notifications",
                ));
            }
            self.queued.push_back(frame);
        }
    }
    pub async fn initialize(&mut self) -> WorkspaceResult<()> {
        self.request("initialize",json!({"clientInfo":{"name":"topics","version":"0.1.0"},"capabilities":{"experimentalApi":true}})).await?;
        self.send(json!({"method":"initialized","params":{}})).await
    }
    pub async fn interrupt_and_wait(&mut self) {
        self.callbacks_allowed = false;
        let (Some(thread), Some(turn)) = (self.thread.clone(), self.turn.clone()) else {
            return;
        };
        let wait = async {
            self.request("turn/interrupt", json!({"threadId":thread,"turnId":turn}))
                .await?;
            loop {
                let frame = if let Some(frame) = self.queued.pop_front() {
                    frame
                } else {
                    self.next().await?
                };
                if frame["method"] == "turn/completed"
                    && frame["params"]["threadId"] == thread
                    && frame["params"]["turn"]["id"] == turn
                {
                    return Ok::<(), topics_core::workspace::ApiError>(());
                }
            }
        };
        let _ = tokio::time::timeout(Duration::from_secs(3), wait).await;
    }
    pub async fn stop(mut self) {
        #[cfg(unix)]
        if let Some(pid) = self.child.id().and_then(|id| i32::try_from(id).ok()) {
            // Terminate this child group, never the IDE group. Codex may place
            // descendants in separate groups; endpoint/pipe EOF, not this signal,
            // is the separately tested proxy shutdown mechanism.
            unsafe {
                libc::kill(-pid, libc::SIGKILL);
            }
        }
        let _ = self.child.start_kill();
        let _ = tokio::time::timeout(Duration::from_secs(3), self.child.wait()).await;
        self.stderr.abort();
        let _ = self.stderr.await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fake(script: &str) -> Process {
        let mut cmd = Command::new("python3");
        cmd.args(["-u", "-c", script]);
        Process::spawn(cmd).unwrap()
    }
    #[tokio::test]
    async fn interleaved_notification_keeps_request_id_and_no_jsonrpc() {
        let mut p = fake(
            "import sys,json\nr=json.loads(sys.stdin.readline());assert 'jsonrpc' not in r\nprint(json.dumps({'method':'thread/status/changed','params':{'status':{'type':'idle'},'threadId':'thread'}}))\nprint(json.dumps({'id':r['id'],'result':{'ok':True}}))",
        );
        assert_eq!(p.request("x", json!({})).await.unwrap(), json!({"ok":true}));
        assert_eq!(p.queued.len(), 1);
        p.stop().await;
    }
    #[tokio::test]
    async fn malformed_unknown_id_exit_and_approval_fail_closed() {
        for script in [
            "import sys;sys.stdin.readline();print('not json')",
            "import sys;sys.stdin.readline();print('{\"id\":999,\"result\":{}}')",
            "import sys;sys.stdin.readline()",
            "import sys;sys.stdin.readline();print('{\"id\":8,\"method\":\"item/commandExecution/requestApproval\",\"params\":{}}');r=sys.stdin.readline();assert '\"error\"' in r",
            "import sys;sys.stdin.readline();print('{\"method\":\"future/schema\",\"params\":{}}')",
        ] {
            let mut p = fake(script);
            assert!(p.request("x", json!({})).await.is_err());
            p.stop().await;
        }
    }
    #[tokio::test]
    async fn output_frame_and_no_newline_are_bounded() {
        let mut reader = BufReader::new(&b"123456789"[..]);
        assert!(read_frame(&mut reader, 4).await.is_err());
        let mut reader = BufReader::new(&b"{}"[..]);
        assert!(read_frame(&mut reader, 8).await.is_err());
        let mut p =
            fake("import sys;sys.stdin.readline();sys.stdout.write('x'*600000);sys.stdout.flush()");
        assert!(p.request("x", json!({})).await.is_err());
        p.stop().await;
    }
    #[tokio::test]
    async fn stderr_is_drained_and_children_reaped() {
        let mut p = fake(
            "import sys,json\nr=json.loads(sys.stdin.readline());sys.stderr.write('x'*1000000);sys.stderr.flush();print(json.dumps({'id':r['id'],'result':{}}))",
        );
        assert!(p.request("x", json!({})).await.is_ok());
        p.stop().await;
    }
    #[tokio::test]
    async fn actual_read_deadline_is_local_and_process_is_reaped() {
        let mut p = fake("import time\ntime.sleep(50)");
        let failure = p.next().await.unwrap_err();
        assert_eq!(failure.details, Some(Failure::ReadDeadline.details()));
        p.stop().await;
    }
    #[tokio::test]
    async fn interrupt_ack_requires_terminal_and_interleaving() {
        let mut p = fake(
            "import sys,json\nr=json.loads(sys.stdin.readline());assert r['method']=='turn/interrupt';assert r['params']=={'threadId':'thread','turnId':'turn'}\nprint(json.dumps({'id':r['id'],'result':{}}));print(json.dumps({'method':'turn/completed','params':{'threadId':'thread','turn':{'id':'turn','status':'interrupted'}}}))",
        );
        p.thread = Some("thread".into());
        p.turn = Some("turn".into());
        p.interrupt_and_wait().await;
        p.stop().await;
    }
}
