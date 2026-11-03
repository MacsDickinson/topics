//! Synthetic timing evidence only: no model calls, auth or selected note reads.
#[cfg(target_os = "macos")]
fn main() {
    use std::{
        process::{Child, Command, Stdio},
        time::Instant,
    };
    struct Load(Vec<Child>);
    impl Drop for Load {
        fn drop(&mut self) {
            for child in &mut self.0 {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("synthetic ' ü");
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let mut samples = vec![];
    for loaded in [false, true] {
        let mut load = Load(vec![]);
        if loaded {
            for _ in 0..2 {
                load.0.push(
                    Command::new("/usr/bin/yes")
                        .env_clear()
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .unwrap(),
                );
            }
        }
        for iteration in 0..8 {
            for python in [false, true] {
                let executable = if python {
                    "/usr/bin/python3"
                } else {
                    "/usr/bin/true"
                };
                let mut cmd =
                    topics_runtime::isolation::command(&root, std::path::Path::new(executable));
                if python {
                    cmd.args(["-I", "-c", "import ctypes,json,os,sys;print('ready')"]);
                }
                let started = Instant::now();
                let output = topics_runtime::isolation::bounded_probe(cmd, b"");
                samples.push(serde_json::json!({"loaded":loaded,"iteration":iteration,"helper":if python {"sandbox_python_imports"}else{"sandbox_true"},"elapsed_ms":started.elapsed().as_secs_f64()*1000.0,"success":output.as_ref().is_ok_and(|v|v.status.success()),"error":output.err()}));
            }
        }
        let started = Instant::now();
        let result = topics_runtime::isolation::preflight(&root);
        samples.push(serde_json::json!({"loaded":loaded,"helper":"complete_preflight","elapsed_ms":started.elapsed().as_secs_f64()*1000.0,"success":result.is_ok(),"error":result.err()}));
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"platform":"macos_local","cold_definition":"first invocation in this measurement process; OS caches were not purged","load_definition":"two bounded owned yes processes with discarded output, killed and reaped after each loaded phase","synthetic_only":true,"samples":samples})).unwrap());
}
#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("Native timing evidence requires macOS");
}
