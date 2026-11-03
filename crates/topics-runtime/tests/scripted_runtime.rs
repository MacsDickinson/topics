//! Full runtime boundary scenarios using a fake process, not fake model success.
#![cfg(target_os = "macos")]
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};
use topics_core::workspace::*;
use topics_runtime::{HostTools, RunRequest, Runtime, RuntimeConfig};
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
    fn commit_candidates(&self, _: Vec<Candidate>) -> WorkspaceResult<Vec<String>> {
        *self.stages.lock().unwrap() += 1;
        Ok(vec![id()])
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
  if 'TOPICS_MCP_TOKEN' in os.environ:servers['topics']={{'enabled':True}}
  result(i,{{'config':{{'web_search':'disabled','features':{{f:False for f in flags}},'mcp_servers':servers}}}})
 elif m=='account/read':result(i,{{'account':{{'type':'chatgpt'}},'requiresOpenaiAuth':True}})
 elif m=='thread/start':result(i,{{'thread':{{'id':thread}},'cwd':p['cwd'],'approvalPolicy':'never','sandbox':{{'type':'readOnly','networkAccess':False}},'instructionSources':[],'modelProvider':'openai'}})
 elif m=='mcpServerStatus/list':result(i,{{'data':[{{'name':'topics','runtimeStatus':'connected','tools':tools,'resources':[],'resourceTemplates':[],'toolsError':None}},{{'name':'rogue','runtimeStatus':'connected' if mode=='rogue' else 'disabled','tools':{{}},'resources':[],'resourceTemplates':[]}}],'nextCursor':None}})
 elif m=='turn/start':
  open({marker:?},'w').write('turn_started')
  result(i,{{'turn':{{'id':turn}}}})
  proxy=subprocess.Popen([{proxy:?},'--topics-mcp-proxy'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  f=proxy.stdout
  def send(v):proxy.stdin.write((json.dumps(v)+'\n').encode());proxy.stdin.flush()
  send({{'jsonrpc':'2.0','id':1,'method':'initialize','params':{{'protocolVersion':'2025-06-18','capabilities':{{}},'clientInfo':{{'name':'scripted','version':'1'}}}}}});f.readline()
  send({{'jsonrpc':'2.0','method':'notifications/initialized'}})
  send({{'jsonrpc':'2.0','id':2,'method':'tools/call','params':{{'name':'propose_changes','arguments':{{'candidates':[{{'kind':'topic','topic':{{'id':'10000000-0000-4000-8000-000000000001','topic_kind':'person','name':'Synthetic Alex','aliases':[],'sensitivity':'cloud_allowed'}}}}]}}}}}});reply=json.loads(f.readline());assert 'error' not in reply
  emit({{'method':'item/agentMessage/delta','params':{{'threadId':thread,'turnId':turn,'itemId':'message','delta':'buffered'}}}})
 elif m=='turn/interrupt':
  result(i,{{}})
  send({{'jsonrpc':'2.0','id':3,'method':'tools/call','params':{{'name':'propose_changes','arguments':{{'candidates':[{{'kind':'topic','topic':{{'id':'10000000-0000-4000-8000-000000000002','topic_kind':'person','name':'Late','aliases':[],'sensitivity':'cloud_allowed'}}}}]}}}}}});reply=json.loads(f.readline());assert 'error' in reply
  emit({{'method':'turn/completed','params':{{'threadId':thread,'turn':{{'id':turn,'status':'completed'}}}}}})
"#,
        proxy = env!("CARGO_BIN_EXE_topics-mcp-proxy"),
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
