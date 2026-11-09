//! Full runtime boundary scenarios using a fake process, not fake model success.
#![cfg(target_os = "macos")]
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};
use topics_core::workspace::*;
use topics_runtime::{CommitPermit, HostTools, RunRequest, Runtime, RuntimeConfig};
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
struct Host {
    calls: Mutex<usize>,
    stages: Mutex<usize>,
}
impl HostTools for Host {
    fn call(&self, _: &str, _: Value) -> WorkspaceResult<Value> {
        *self.calls.lock().unwrap() += 1;
        Ok(json!({"validated":true}))
    }
    fn commit_candidates(&self, _: Vec<Candidate>, permit: CommitPermit) -> WorkspaceResult<()> {
        permit.publish(vec![id()], &mut || {
            *self.stages.lock().unwrap() += 1;
            Ok(())
        })
    }
}
fn fixture(mode: &str) -> (tempfile::TempDir, Runtime, RunRequest, Arc<Host>) {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let notes = temp.path().join("notes");
    std::fs::create_dir(&notes).unwrap();
    std::fs::write(notes.join("source.md"), "SYNTHETIC_SELECTED_BYTES").unwrap();
    let script = temp.path().join("codex-fixture");
    let tools = topics_runtime::bridge::tools();
    let toolmap: serde_json::Map<String, Value> = tools
        .into_iter()
        .map(|tool| (tool.name.to_string(), serde_json::to_value(tool).unwrap()))
        .collect();
    let source = format!(
        r#"#!/usr/bin/env python3
import sys,json,os,socket,subprocess
assert not {{'CODEX_ROLLOUT_TRACE_ROOT','CODEX_CA_CERTIFICATE','SSLKEYLOGFILE','CODEX_ACCESS_TOKEN','CODEX_APP_SERVER_CHATGPT_BASE_URL','CODEX_AUTHAPI_BASE_URL','OPENAI_ORGANIZATION','OPENAI_PROJECT'}}.intersection(os.environ)
if '--version' in sys.argv: print('codex-cli 0.166.0');sys.exit()
def emit(v):print(json.dumps(v),flush=True)
def result(i,v):emit({{'id':i,'result':v}})
mode={mode:?}
flags={flags}
tools=json.loads({tools:?})
thread='thread';turn='turn'
for line in sys.stdin:
 r=json.loads(line);m=r.get('method');i=r.get('id');p=r.get('params',{{}})
 if m=='initialize':result(i,{{}})
 elif m=='initialized':pass
 elif m=='config/read':
  servers={{'rogue':{{'enabled':not any('rogue.enabled=false' in a for a in sys.argv)}}}}
  c=json.loads('{{"openai_base_url":"https://chatgpt.com/backend-api/codex","chatgpt_base_url":"https://chatgpt.com/backend-api/","notify": [], "model_provider": "openai", "skills": {{"include_instructions": false}}, "agents": {{"enabled": false}}, "analytics": {{"enabled": false}}, "feedback": {{"enabled": false}}, "compact_prompt": "", "experimental_thread_store": {{"type": "local"}}, "otel": {{"exporter": "none", "trace_exporter": "none", "metrics_exporter": "none", "log_user_prompt": false, "log_agent_responses": false, "log_guardian_assessments": false}}}}');c['model_catalog_json']=json.loads(next(a.split('=',1)[1] for a in sys.argv if a.startswith('model_catalog_json=')));c.update({{'web_search':'disabled','features':{{**{{f:False for f in flags}},'multi_agent_v2':{{'enabled':False,'disable_native_tools':True}}}},'mcp_servers':servers}});result(i,{{'config':c}})
 elif m=='account/read':result(i,{{'account':{{'type':'chatgpt'}},'requiresOpenaiAuth':True}})
 elif m=='thread/start':
  assert p['model']=='gpt-6.1-sol' and len(p['dynamicTools'])==1 and p['dynamicTools'][0]['name']=='topics'
  assert sorted(t['name'] for t in p['dynamicTools'][0]['tools'])==['propose_changes','scope_sources']
  assert all(t['inputSchema']==tools[t['name']]['inputSchema'] for t in p['dynamicTools'][0]['tools'])
  result(i,{{'thread':{{'id':thread,'ephemeral':True}},'cwd':p['cwd'],'approvalPolicy':'never','sandbox':{{'type':'readOnly','networkAccess':False}},'instructionSources':[],'runtimeWorkspaceRoots':[],'modelProvider':'openai','model':'gpt-6.1-sol'}})
 elif m=='mcpServerStatus/list':result(i,{{'data':[{{'name':'rogue','runtimeStatus':'connected' if mode=='rogue' else 'disabled','tools':{{}},'resources':[],'resourceTemplates':[]}}],'nextCursor':None}})
 elif m=='turn/start':
  open({marker:?},'w').write('turn_started')
  result(i,{{'turn':{{'id':turn}}}})
  def toolcall(call,tool,args):
   emit({{'id':call,'method':'item/tool/call','params':{{'threadId':thread,'turnId':turn,'callId':str(call),'namespace':'topics','tool':tool,'arguments':args}}}})
   reply=json.loads(sys.stdin.readline());assert reply['id']==call;return reply
  if not mode.startswith('empty'):
   reply=toolcall(10000,'propose_changes',{{'candidates':[{{'kind':'topic','topic':{{'id':'10000000-0000-4000-8000-000000000001','topic_kind':'person','name':'Synthetic Alex','aliases':[],'sensitivity':'cloud_allowed'}}}}]}});assert reply['result']['success']
  emit({{'method':'item/agentMessage/delta','params':{{'threadId':thread,'turnId':turn,'itemId':'message','delta':'buffered'}}}})
  if mode in ['empty_complete','empty_failed']:
   emit({{'method':'turn/completed','params':{{'threadId':thread,'turn':{{'id':turn,'status':'completed' if mode=='empty_complete' else 'failed'}}}}}});continue
  if mode=='invalid_after_buffer':
   assert 'error' in toolcall(10001,'read_file',{{'path':'private.md'}})
   emit({{'method':'turn/completed','params':{{'threadId':thread,'turn':{{'id':turn,'status':'completed'}}}}}});continue
  if mode=='malformed':print('broken frame',flush=True);sys.exit()
  if mode=='exit':sys.exit()
  if mode in ['unknown','failed','complete']:
   status={{'unknown':'futureStatus','failed':'failed','complete':'completed'}}[mode]
   emit({{'method':'turn/completed','params':{{'threadId':thread,'turn':{{'id':turn,'status':status}}}}}})
   if mode=='complete':emit({{'method':'turn/completed','params':{{'threadId':thread,'turn':{{'id':turn,'status':'completed'}}}}}})
   continue
  if mode=='wrong_turn':emit({{'method':'item/agentMessage/delta','params':{{'threadId':thread,'turnId':'other','itemId':'late','delta':'late'}}}});continue

 elif m=='turn/interrupt':
  result(i,{{}})
  assert 'error' in toolcall(10002,'propose_changes',{{'candidates':[{{'kind':'topic','topic':{{'id':'10000000-0000-4000-8000-000000000002','topic_kind':'person','name':'Late','aliases':[],'sensitivity':'cloud_allowed'}}}}]}})
  emit({{'method':'turn/completed','params':{{'threadId':thread,'turn':{{'id':turn,'status':'completed'}}}}}})
"#,
        flags = serde_json::to_string(&topics_runtime::protocol::DISABLED).unwrap(),
        tools = serde_json::to_string(&toolmap).unwrap(),
        marker = temp.path().join("model-sent.marker").display().to_string()
    );
    std::fs::write(&script, source).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
    let runtime = Runtime::new(RuntimeConfig {
        codex_path: script,
        proxy_path: PathBuf::from(env!("CARGO_BIN_EXE_topics-mcp-proxy")),
        app_data: temp.path().join("app-data"),
    });
    let req = RunRequest {
        root_id: id(),
        run_id: id(),
        scope_id: id(),
        notes_root: notes,
        prompt: "SYNTHETIC_SELECTED_BYTES".into(),
    };
    (
        temp,
        runtime,
        req,
        Arc::new(Host {
            calls: Mutex::new(0),
            stages: Mutex::new(0),
        }),
    )
}
#[test]
fn unexpected_tools_block_real_run_before_selected_bytes() {
    let (temp, rt, request, host) = fixture("rogue");
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        request,
        host.clone(),
        Arc::new(move |e| {
            tx.send(e).unwrap();
        }),
    )
    .unwrap();
    let e = rx.recv_timeout(Duration::from_secs(15)).unwrap();
    assert!(
        matches!(e.event,RunEvent::Error{error:ApiError{code,..}} if code=="runtime_unavailable")
    );
    assert!(!temp.path().join("model-sent.marker").exists());
    assert_eq!(*host.calls.lock().unwrap(), 0);
    assert_eq!(*host.stages.lock().unwrap(), 0);
}
#[test]
fn cancellation_after_buffer_and_late_completed_cannot_stage() {
    let (_temp, rt, request, host) = fixture("cancel");
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        request.clone(),
        host.clone(),
        Arc::new(move |e| {
            tx.send(e).unwrap();
        }),
    )
    .unwrap();
    let first = rx.recv_timeout(Duration::from_secs(15)).unwrap();
    assert!(
        matches!(first.event, RunEvent::TextDelta { .. }),
        "{first:?}"
    );
    assert!(!rt.cancel("wrong-root", &request.run_id));
    assert!(rt.cancel(&request.root_id, &request.run_id));
    let terminal = rx.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(
        matches!(terminal.event, RunEvent::Cancelled),
        "{terminal:?}"
    );
    assert_eq!(*host.stages.lock().unwrap(), 0);
    assert_eq!(*host.calls.lock().unwrap(), 1);
    assert!(rx.recv_timeout(Duration::from_millis(100)).is_err());
}

#[test]
fn malformed_unknown_status_failed_exit_wrong_turn_discard_buffered_candidates() {
    for mode in [
        "invalid_after_buffer",
        "malformed",
        "unknown",
        "failed",
        "exit",
        "wrong_turn",
    ] {
        let (_temp, rt, request, host) = fixture(mode);
        let (tx, rx) = mpsc::channel();
        rt.analysis_start(
            request,
            host.clone(),
            Arc::new(move |e| {
                tx.send(e).unwrap();
            }),
        )
        .unwrap();
        assert!(matches!(
            rx.recv_timeout(Duration::from_secs(15)).unwrap().event,
            RunEvent::TextDelta { .. }
        ));
        let terminal = rx.recv_timeout(Duration::from_secs(15)).unwrap();
        assert!(
            matches!(terminal.event, RunEvent::Error { .. }),
            "{mode}: {terminal:?}"
        );
        if let RunEvent::Error { error } = &terminal.event {
            assert_eq!(
                error.details,
                (mode == "failed").then(|| json!({
                    "phase":"turn", "reason":"model_terminal_failed"
                })),
                "unknown or other failures must not be guessed from broad error codes: {mode}"
            );
        }
        assert_eq!(*host.stages.lock().unwrap(), 0, "{mode}");
        assert!(rx.recv_timeout(Duration::from_millis(30)).is_err());
    }
}
#[test]
fn duplicate_terminal_stages_once_and_closes_run() {
    let (_temp, rt, request, host) = fixture("complete");
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        request.clone(),
        host.clone(),
        Arc::new(move |e| {
            tx.send(e).unwrap();
        }),
    )
    .unwrap();
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(15)).unwrap().event,
        RunEvent::TextDelta { .. }
    ));
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(15)).unwrap().event,
        RunEvent::Proposal { .. }
    ));
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(15)).unwrap().event,
        RunEvent::Completed
    ));
    assert_eq!(*host.stages.lock().unwrap(), 1);
    assert!(!rt.cancel(&request.root_id, &request.run_id));
    assert!(rx.recv_timeout(Duration::from_millis(50)).is_err());
}

#[test]
fn wrong_pinned_version_is_actionable_and_sends_no_bytes() {
    let (temp, rt, request, host) = fixture("rogue");
    let script = temp.path().join("codex-fixture");
    let text = std::fs::read_to_string(&script)
        .unwrap()
        .replace("codex-cli 0.166.0", "codex-cli 0.165.0");
    std::fs::write(script, text).unwrap();
    let status = rt.status();
    assert!(!status.available);
    assert!(status.message.contains("0.166.0"));
    assert!(status.message.contains("0.165.0"));
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        request,
        host.clone(),
        Arc::new(move |e| {
            tx.send(e).unwrap();
        }),
    )
    .unwrap();
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(10)).unwrap().event,
        RunEvent::Error { .. }
    ));
    assert_eq!(*host.calls.lock().unwrap(), 0);
    assert!(!temp.path().join("model-sent.marker").exists());
}

#[test]
fn review_overlapping_root_and_appdata_never_create_directories() {
    let (temp, rt, mut request, host) = fixture("rogue");
    // Reuse this fixture's executable with deliberately overlapping configuration.
    let config = RuntimeConfig {
        codex_path: temp.path().join("codex-fixture"),
        proxy_path: PathBuf::from(env!("CARGO_BIN_EXE_topics-mcp-proxy")),
        app_data: request.notes_root.join("must-not-create"),
    };
    let bad = Runtime::new(config);
    assert!(
        bad.analysis_start(request.clone(), host.clone(), Arc::new(|_| {}))
            .is_err()
    );
    assert!(!request.notes_root.join("must-not-create").exists());
    let app = temp.path().join("app-data");
    std::fs::create_dir_all(app.join("runtime")).unwrap();
    request.notes_root = app.join("runtime");
    assert!(rt.analysis_start(request, host, Arc::new(|_| {})).is_err());
}

struct SlowHost {
    started: Arc<std::sync::atomic::AtomicBool>,
    base: Arc<Host>,
}
impl HostTools for SlowHost {
    fn call(&self, tool: &str, args: Value) -> WorkspaceResult<Value> {
        self.started
            .store(true, std::sync::atomic::Ordering::SeqCst);
        std::thread::sleep(Duration::from_secs(8));
        self.base.call(tool, args)
    }
    fn commit_candidates(
        &self,
        candidates: Vec<Candidate>,
        permit: CommitPermit,
    ) -> WorkspaceResult<()> {
        self.base.commit_candidates(candidates, permit)
    }
}
struct PanicHost {
    base: Arc<Host>,
}
impl HostTools for PanicHost {
    fn call(&self, tool: &str, args: Value) -> WorkspaceResult<Value> {
        self.base.call(tool, args)
    }
    fn commit_candidates(&self, _: Vec<Candidate>, _: CommitPermit) -> WorkspaceResult<()> {
        panic!("synthetic staging panic")
    }
}
#[test]
fn review_cancel_stalled_host_has_bounded_terminal_and_releases_run_slot() {
    let (_temp, rt, request, base) = fixture("cancel");
    let started = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let host = Arc::new(SlowHost {
        started: started.clone(),
        base: base.clone(),
    });
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        request.clone(),
        host,
        Arc::new(move |e| {
            let _ = tx.send(e);
        }),
    )
    .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !started.load(std::sync::atomic::Ordering::SeqCst) {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    let time = std::time::Instant::now();
    assert!(rt.cancel(&request.root_id, &request.run_id));
    let terminal = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(terminal.event, RunEvent::Cancelled));
    assert!(time.elapsed() < Duration::from_secs(5));
    assert_eq!(*base.stages.lock().unwrap(), 0);
    let mut next = request;
    next.run_id = id();
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        next.clone(),
        base,
        Arc::new(move |e| {
            let _ = tx.send(e);
        }),
    )
    .unwrap();
    assert!(rt.cancel(&next.root_id, &next.run_id));
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(5)).unwrap().event,
        RunEvent::Cancelled
    ));
}
#[test]
fn review_staging_panic_emits_one_error_and_runtime_remains_usable() {
    let (_temp, rt, request, base) = fixture("complete");
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        request.clone(),
        Arc::new(PanicHost { base: base.clone() }),
        Arc::new(move |e| {
            let _ = tx.send(e);
        }),
    )
    .unwrap();
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(10)).unwrap().event,
        RunEvent::TextDelta { .. }
    ));
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(10)).unwrap().event,
        RunEvent::Error { .. }
    ));
    assert!(!rt.cancel(&request.root_id, &request.run_id));
    assert!(rx.recv_timeout(Duration::from_millis(30)).is_err());
    let mut next = request;
    next.run_id = id();
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        next,
        base.clone(),
        Arc::new(move |e| {
            let _ = tx.send(e);
        }),
    )
    .unwrap();
    loop {
        if matches!(
            rx.recv_timeout(Duration::from_secs(10)).unwrap().event,
            RunEvent::Completed
        ) {
            break;
        }
    }
    assert_eq!(*base.stages.lock().unwrap(), 1);
}

struct GuardedHost {
    base: Arc<Host>,
    entered: mpsc::Sender<()>,
    release: Mutex<mpsc::Receiver<()>>,
    done: mpsc::Sender<bool>,
    publish_first: bool,
}
impl HostTools for GuardedHost {
    fn call(&self, tool: &str, args: Value) -> WorkspaceResult<Value> {
        self.base.call(tool, args)
    }
    fn commit_candidates(
        &self,
        candidates: Vec<Candidate>,
        permit: CommitPermit,
    ) -> WorkspaceResult<()> {
        if self.publish_first {
            self.base
                .commit_candidates(candidates.clone(), permit.clone())?;
        }
        let _ = self.entered.send(());
        let _ = self
            .release
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(10));
        let result = if self.publish_first {
            Err(ApiError::new(
                "internal",
                "synthetic host error after successful publication",
            ))
        } else {
            self.base.commit_candidates(candidates, permit)
        };
        let _ = self.done.send(result.is_err());
        result
    }
}
#[test]
fn review_cancel_and_timeout_revoke_late_prepared_publication() {
    for cancel in [true, false] {
        let (_temp, rt, request, base) = fixture("complete");
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let host = Arc::new(GuardedHost {
            base: base.clone(),
            entered: entered_tx,
            release: Mutex::new(release_rx),
            done: done_tx,
            publish_first: false,
        });
        let (tx, rx) = mpsc::channel();
        rt.analysis_start(
            request.clone(),
            host,
            Arc::new(move |e| {
                let _ = tx.send(e);
            }),
        )
        .unwrap();
        assert!(matches!(
            rx.recv_timeout(Duration::from_secs(10)).unwrap().event,
            RunEvent::TextDelta { .. }
        ));
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let start = std::time::Instant::now();
        if cancel {
            assert!(rt.cancel(&request.root_id, &request.run_id));
            assert!(start.elapsed() < Duration::from_millis(200));
        }
        let terminal = rx.recv_timeout(Duration::from_secs(6)).unwrap();
        assert!(if cancel {
            matches!(terminal.event, RunEvent::Cancelled)
        } else {
            matches!(terminal.event, RunEvent::Error { .. })
        });
        assert_eq!(*base.stages.lock().unwrap(), 0);
        release_tx.send(()).unwrap();
        assert!(done_rx.recv_timeout(Duration::from_secs(1)).unwrap());
        assert_eq!(*base.stages.lock().unwrap(), 0);
        assert!(rx.recv_timeout(Duration::from_millis(20)).is_err());
        let mut next = request;
        next.run_id = id();
        let (tx, rx) = mpsc::channel();
        rt.analysis_start(
            next.clone(),
            base,
            Arc::new(move |e| {
                let _ = tx.send(e);
            }),
        )
        .unwrap();
        assert!(rt.cancel(&next.root_id, &next.run_id));
        assert!(matches!(
            rx.recv_timeout(Duration::from_secs(5)).unwrap().event,
            RunEvent::Cancelled
        ));
    }
}
#[test]
fn review_published_outcome_survives_host_stall_and_error() {
    let (_temp, rt, request, base) = fixture("complete");
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let host = Arc::new(GuardedHost {
        base: base.clone(),
        entered: entered_tx,
        release: Mutex::new(release_rx),
        done: done_tx,
        publish_first: true,
    });
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        request.clone(),
        host,
        Arc::new(move |e| {
            let _ = tx.send(e);
        }),
    )
    .unwrap();
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(10)).unwrap().event,
        RunEvent::TextDelta { .. }
    ));
    entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(!rt.cancel(&request.root_id, &request.run_id));
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(2)).unwrap().event,
        RunEvent::Proposal { .. }
    ));
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(2)).unwrap().event,
        RunEvent::Completed
    ));
    assert_eq!(*base.stages.lock().unwrap(), 1);
    release_tx.send(()).unwrap();
    assert!(done_rx.recv_timeout(Duration::from_secs(1)).unwrap());
    assert!(rx.recv_timeout(Duration::from_millis(40)).is_err());
}

#[test]
fn review_child_environment_allowlist() {
    if std::env::var_os("TOPICS_ENV_TEST_CHILD").is_some() {
        let (_temp, rt, request, host) = fixture("complete");
        let (tx, rx) = mpsc::channel();
        rt.analysis_start(
            request,
            host.clone(),
            Arc::new(move |e| {
                let _ = tx.send(e);
            }),
        )
        .unwrap();
        loop {
            let e = rx.recv_timeout(Duration::from_secs(15)).unwrap();
            match e.event {
                RunEvent::Completed => break,
                RunEvent::Error { error } => {
                    panic!("filtered child environment must support runtime: {error}")
                }
                _ => {}
            }
        }
        assert_eq!(*host.stages.lock().unwrap(), 1);
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let output = rt.block_on(async {
        let mut cmd = tokio::process::Command::new(std::env::current_exe().unwrap());
        cmd.args([
            "--exact",
            "review_child_environment_allowlist",
            "--nocapture",
        ])
        .env("TOPICS_ENV_TEST_CHILD", "1")
        .env("CODEX_ROLLOUT_TRACE_ROOT", temp.path().join("trace"))
        .env("CODEX_CA_CERTIFICATE", temp.path().join("untrusted-ca"))
        .env("SSLKEYLOGFILE", temp.path().join("keys"))
        .env("CODEX_ACCESS_TOKEN", "synthetic-not-a-token")
        .env("CODEX_APP_SERVER_CHATGPT_BASE_URL", "http://127.0.0.1:1")
        .env("CODEX_AUTHAPI_BASE_URL", "http://127.0.0.1:1")
        .env("OPENAI_ORGANIZATION", "synthetic")
        .env("OPENAI_PROJECT", "synthetic")
        .kill_on_drop(true);
        tokio::time::timeout(Duration::from_secs(20), cmd.output())
            .await
            .unwrap()
            .unwrap()
    });
    assert!(
        output.status.success(),
        "hostile inherited trace/CA/endpoint environment must be removed"
    );
    assert!(!temp.path().join("trace").exists());
    assert!(!temp.path().join("keys").exists());
}

/// Setup opts in because even an empty extraction needs a durable checkpoint.
struct EmptyCheckpointHost {
    base: Arc<Host>,
    entered: Option<mpsc::Sender<()>>,
    release: Mutex<Option<mpsc::Receiver<()>>>,
    finished: Option<mpsc::Sender<bool>>,
}
impl HostTools for EmptyCheckpointHost {
    fn call(&self, tool: &str, args: Value) -> WorkspaceResult<Value> {
        self.base.call(tool, args)
    }
    fn requires_empty_commit(&self) -> bool {
        true
    }
    fn commit_candidates(
        &self,
        candidates: Vec<Candidate>,
        permit: CommitPermit,
    ) -> WorkspaceResult<()> {
        assert!(
            candidates.is_empty(),
            "empty turn cannot fabricate proposals"
        );
        if let Some(entered) = &self.entered {
            entered.send(()).unwrap();
        }
        if let Some(release) = self.release.lock().unwrap().take() {
            release.recv_timeout(Duration::from_secs(10)).unwrap();
        }
        let result = self.base.commit_candidates(candidates, permit);
        if let Some(finished) = &self.finished {
            let _ = finished.send(result.is_err());
        }
        result
    }
}
fn empty_host(base: Arc<Host>) -> Arc<EmptyCheckpointHost> {
    Arc::new(EmptyCheckpointHost {
        base,
        entered: None,
        release: Mutex::new(None),
        finished: None,
    })
}
#[test]
fn setup_empty_success_publishes_checkpoint_once_through_real_permit() {
    let (_temp, rt, request, base) = fixture("empty_complete");
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        request,
        empty_host(base.clone()),
        Arc::new(move |e| {
            let _ = tx.send(e);
        }),
    )
    .unwrap();
    let mut publications = 0;
    loop {
        match rx.recv_timeout(Duration::from_secs(15)).unwrap().event {
            RunEvent::Proposal { .. } => publications += 1,
            RunEvent::Completed => break,
            RunEvent::Error { error } => panic!("empty setup success failed: {error}"),
            _ => {}
        }
    }
    assert_eq!(
        *base.stages.lock().unwrap(),
        1,
        "successful empty extraction must checkpoint"
    );
    assert_eq!(publications, 1);
    assert_eq!(*base.calls.lock().unwrap(), 0);
}
#[test]
fn setup_empty_revocation_denies_prepared_checkpoint() {
    let (_temp, rt, request, base) = fixture("empty_complete");
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let host = Arc::new(EmptyCheckpointHost {
        base: base.clone(),
        entered: Some(entered_tx),
        release: Mutex::new(Some(release_rx)),
        finished: Some(finished_tx),
    });
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        request.clone(),
        host,
        Arc::new(move |e| {
            let _ = tx.send(e);
        }),
    )
    .unwrap();
    entered_rx
        .recv_timeout(Duration::from_secs(15))
        .expect("empty checkpoint preparation must reach real fence");
    assert!(rt.cancel(&request.root_id, &request.run_id));
    release_tx.send(()).unwrap();
    assert!(finished_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    loop {
        match rx.recv_timeout(Duration::from_secs(5)).unwrap().event {
            RunEvent::Cancelled => break,
            RunEvent::TextDelta { .. } => {}
            event => panic!("revoked empty completion emitted {event:?}"),
        }
    }
    assert_eq!(*base.stages.lock().unwrap(), 0);
}
#[test]
fn setup_empty_failed_and_cancelled_turns_never_checkpoint() {
    for mode in ["empty_failed", "empty_cancel"] {
        let (_temp, rt, request, base) = fixture(mode);
        let (tx, rx) = mpsc::channel();
        rt.analysis_start(
            request.clone(),
            empty_host(base.clone()),
            Arc::new(move |e| {
                let _ = tx.send(e);
            }),
        )
        .unwrap();
        assert!(matches!(
            rx.recv_timeout(Duration::from_secs(15)).unwrap().event,
            RunEvent::TextDelta { .. }
        ));
        if mode == "empty_cancel" {
            assert!(rt.cancel(&request.root_id, &request.run_id));
        }
        let event = rx.recv_timeout(Duration::from_secs(10)).unwrap().event;
        assert!(if mode == "empty_cancel" {
            matches!(event, RunEvent::Cancelled)
        } else {
            matches!(event, RunEvent::Error { .. })
        });
        assert_eq!(*base.stages.lock().unwrap(), 0);
    }
}
#[test]
fn setup_empty_default_host_keeps_existing_no_publication_behavior() {
    let (_temp, rt, request, base) = fixture("empty_complete");
    let (tx, rx) = mpsc::channel();
    rt.analysis_start(
        request,
        base.clone(),
        Arc::new(move |e| {
            let _ = tx.send(e);
        }),
    )
    .unwrap();
    loop {
        match rx.recv_timeout(Duration::from_secs(15)).unwrap().event {
            RunEvent::Completed => break,
            RunEvent::TextDelta { .. } => {}
            event => panic!("default empty behavior changed: {event:?}"),
        }
    }
    assert_eq!(*base.stages.lock().unwrap(), 0);
}
