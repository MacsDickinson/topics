//! Native defence-in-depth: an ancestor cannot be cloned around the root deny.
#![cfg(target_os = "macos")]
use std::{path::Path, process::Command};
const CODE: &str = r#"import ctypes,json,os,sys
lib=ctypes.CDLL(None,use_errno=True)
lib.clonefile.argtypes=[ctypes.c_char_p,ctypes.c_char_p,ctypes.c_uint32]
lib.fclonefileat.argtypes=[ctypes.c_int,ctypes.c_int,ctypes.c_char_p,ctypes.c_uint32]
ctypes.set_errno(0)
if sys.argv[1]=='clonefile': r=lib.clonefile(os.fsencode(sys.argv[2]),os.fsencode(sys.argv[3]),0)
else:
 fd=os.open(sys.argv[2],os.O_RDONLY)
 r=lib.fclonefileat(fd,-2,os.fsencode(sys.argv[3]),0)
 os.close(fd)
print(json.dumps({'result':r,'errno':ctypes.get_errno()}))
"#;
fn invoke(root: Option<&Path>, mode: &str, source: &Path, destination: &Path) -> serde_json::Value {
    let mut cmd = if let Some(root) = root {
        topics_runtime::isolation::command(root, Path::new("/usr/bin/python3"))
    } else {
        Command::new("/usr/bin/python3")
    };
    cmd.args(["-I", "-c", CODE, mode])
        .arg(source)
        .arg(destination);
    let output = topics_runtime::isolation::bounded_probe(cmd, b"").expect("bounded native helper");
    assert!(
        output.status.success(),
        "native helper must actually execute"
    );
    serde_json::from_slice(&output.stdout)
        .expect("native helper must return structured syscall result")
}
fn ancestor_clone_is_denied(mode: &str) {
    let fixture = tempfile::tempdir().unwrap();
    let parent = fixture.path().join("ancestor ' ü");
    let root = parent.join("Notes");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("sentinel.md"), b"synthetic clone sentinel").unwrap();
    let control = fixture.path().join("positive");
    assert_eq!(
        invoke(None, mode, &parent, &control)["result"],
        0,
        "unsandboxed clone positive control"
    );
    assert_eq!(
        std::fs::read(control.join("Notes/sentinel.md")).unwrap(),
        b"synthetic clone sentinel"
    );
    let forbidden = fixture.path().join("forbidden");
    let result = invoke(
        Some(&root.canonicalize().unwrap()),
        mode,
        &parent,
        &forbidden,
    );
    assert_eq!(
        result["result"], -1,
        "ancestor clone syscall must be denied"
    );
    assert!(
        matches!(result["errno"].as_i64(), Some(1 | 13)),
        "must be a permission denial"
    );
    assert!(!forbidden.exists());
    let sibling = fixture.path().join("sibling");
    std::fs::create_dir(&sibling).unwrap();
    std::fs::write(sibling.join("safe"), b"synthetic sibling").unwrap();
    let destination = fixture.path().join("sibling-copy");
    assert_eq!(
        invoke(
            Some(&root.canonicalize().unwrap()),
            mode,
            &sibling,
            &destination
        )["result"],
        0,
        "unprotected sibling clone must remain available"
    );
}
#[test]
fn clonefile_ancestor_cannot_copy_protected_descendant() {
    ancestor_clone_is_denied("clonefile");
}
#[test]
fn fclonefileat_ancestor_cannot_copy_protected_descendant() {
    ancestor_clone_is_denied("fclonefileat");
}
