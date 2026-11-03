//! Path denial with literal ancestor-rename protection. No outside-copy guarantee.
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};
pub const PROFILE: &str = "(version 1)(allow default)(deny file-read-data(subpath(param \"TOPICS_ROOT\")))(deny file-write*(subpath(param \"TOPICS_ROOT\")))";
#[cfg(target_os = "macos")]
pub fn command(root: &Path, executable: &Path) -> Command {
    let mut command = Command::new("/usr/bin/sandbox-exec");
    command
        .arg("-D")
        .arg(format!("TOPICS_ROOT={}", root.display()));
    let mut profile = PROFILE.to_owned();
    for (index, ancestor) in root.ancestors().skip(1).enumerate() {
        let key = format!("TOPICS_ANCESTOR_{index}");
        command
            .arg("-D")
            .arg(format!("{key}={}", ancestor.display()));
        profile.push_str(&format!(
            "(deny file-write-unlink(literal(param \"{key}\")))(deny file-clone(literal(param \"{key}\")))"
        ));
    }
    command.arg("-p").arg(profile).arg(executable);
    command
}
#[cfg(not(target_os = "macos"))]
pub fn command(_: &Path, executable: &Path) -> Command {
    Command::new(executable)
}
/// Bound process lifetime AND captured output. Never retain native error paths.
pub fn bounded_probe(mut command: Command, input: &[u8]) -> Result<Output, String> {
    crate::environment::apply(&mut command).map_err(|error| error.message)?;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|_| "Cannot launch native confinement probe".to_owned())?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || {
        let mut bytes = vec![];
        let _ = stdout.take(8192).read_to_end(&mut bytes);
        bytes
    });
    let err = std::thread::spawn(move || {
        let mut bytes = vec![];
        let _ = stderr.take(8192).read_to_end(&mut bytes);
        bytes
    });
    if input.len() > 4096 {
        let _ = child.kill();
        let _ = child.wait();
        return Err("Native probe input too large".into());
    }
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input);
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(5)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break Err("Native confinement probe exceeded deadline".to_owned());
            }
        }
    };
    let stdout = out.join().unwrap_or_default();
    let stderr = err.join().unwrap_or_default();
    Ok(Output {
        status: status?,
        stdout,
        stderr,
    })
}
#[cfg(target_os = "macos")]
fn probe(
    root: &Path,
    exe: &str,
    args: &[&std::ffi::OsStr],
    input: &[u8],
) -> Result<Output, String> {
    let mut cmd = command(root, Path::new(exe));
    cmd.args(args);
    bounded_probe(cmd, input)
}

#[cfg(any(target_os = "macos", all(test, unix)))]
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SyscallResult {
    result: i32,
    errno: i32,
}
#[cfg(any(target_os = "macos", all(test, unix)))]
fn syscall_result(output: Output) -> Result<SyscallResult, String> {
    if !output.status.success() {
        return Err("Native syscall helper did not execute successfully; install a working macOS Python 3 helper and retry".into());
    }
    serde_json::from_slice(&output.stdout).map_err(|_| {
        "Native syscall helper returned an invalid result; repair the helper and retry".into()
    })
}
#[cfg(any(target_os = "macos", all(test, unix)))]
fn require_success(output: Output) -> Result<(), String> {
    let value = syscall_result(output)?;
    if value.result != 0 {
        return Err(
            "Native positive control failed; this filesystem or helper cannot prove confinement"
                .into(),
        );
    }
    Ok(())
}
#[cfg(any(target_os = "macos", all(test, unix)))]
fn require_denial(output: Output) -> Result<(), String> {
    let value = syscall_result(output)?;
    if value.result != -1 || ![libc::EPERM, libc::EACCES].contains(&value.errno) {
        return Err(
            "Native syscall was not denied by permissions; analysis remains blocked".into(),
        );
    }
    Ok(())
}
#[cfg(target_os = "macos")]
const SYSCALL_PROBE: &str = r#"import ctypes,json,os,sys
lib=ctypes.CDLL(None,use_errno=True)
ctypes.set_errno(0)
mode=sys.argv[1]
if mode=='swap':
 lib.renameatx_np.argtypes=[ctypes.c_int,ctypes.c_char_p,ctypes.c_int,ctypes.c_char_p,ctypes.c_uint32]
 r=lib.renameatx_np(-2,os.fsencode(sys.argv[2]),-2,os.fsencode(sys.argv[3]),2)
elif mode=='clonefile':
 lib.clonefile.argtypes=[ctypes.c_char_p,ctypes.c_char_p,ctypes.c_uint32]
 r=lib.clonefile(os.fsencode(sys.argv[2]),os.fsencode(sys.argv[3]),0)
else:
 lib.fclonefileat.argtypes=[ctypes.c_int,ctypes.c_int,ctypes.c_char_p,ctypes.c_uint32]
 fd=os.open(sys.argv[2],os.O_RDONLY)
 r=lib.fclonefileat(fd,-2,os.fsencode(sys.argv[3]),0)
 os.close(fd)
print(json.dumps({'result':r,'errno':ctypes.get_errno()}))
"#;
#[cfg(target_os = "macos")]
fn syscall_probe(
    root: Option<&Path>,
    mode: &str,
    source: &Path,
    destination: &Path,
) -> Result<Output, String> {
    let mut command = if let Some(root) = root {
        command(root, Path::new("/usr/bin/python3"))
    } else {
        Command::new("/usr/bin/python3")
    };
    command
        .args(["-I", "-c", SYSCALL_PROBE, mode])
        .arg(source)
        .arg(destination);
    bounded_probe(command, b"")
}
#[cfg(target_os = "macos")]
pub fn preflight(root: &Path) -> Result<PathBuf, String> {
    use std::os::unix::{fs::MetadataExt, fs::symlink};
    let canonical = root
        .canonicalize()
        .map_err(|_| "Selected root is unavailable".to_owned())?;
    if canonical.to_str().is_none() || !canonical.is_dir() {
        return Err("Selected root requires a valid UTF-8 directory".into());
    }
    let temp = std::env::temp_dir()
        .canonicalize()
        .map_err(|_| "Synthetic probe directory unavailable".to_owned())?;
    if std::fs::metadata(&temp)
        .map_err(|_| "Probe filesystem unavailable")?
        .dev()
        != std::fs::metadata(&canonical)
            .map_err(|_| "Selected filesystem unavailable")?
            .dev()
    {
        return Err("Confinement has not been verified for the selected filesystem; choose a supported local volume".into());
    }
    let base = temp.join(format!("topics-confinement-{}", uuid::Uuid::new_v4()));
    if base.starts_with(&canonical) {
        return Err("Synthetic preflight directory overlaps selected notes root".into());
    }
    std::fs::create_dir(&base).map_err(|_| "Cannot create synthetic fixture")?;
    let result = (|| {
        let parent = base.join("ancestor ' \\\" ü");
        let grandparent = parent.join("nested");
        let protected = grandparent.join("root");
        std::fs::create_dir_all(&protected).map_err(|_| "fixture")?;
        let protected = protected.canonicalize().map_err(|_| "fixture")?;
        let sentinel = protected.join("sentinel");
        std::fs::write(&sentinel, b"synthetic sentinel").map_err(|_| "fixture")?;
        let alias = base.join("alias");
        symlink(&protected, &alias).map_err(|_| "fixture")?;
        for path in [sentinel.clone(), alias.join("sentinel")] {
            if probe(&protected, "/bin/cat", &[path.as_os_str()], b"")?
                .status
                .success()
            {
                return Err("OS read confinement failed".into());
            }
            if probe(
                &protected,
                "/usr/bin/tee",
                &[path.as_os_str()],
                b"forbidden overwrite",
            )?
            .status
            .success()
                || std::fs::read(&sentinel).map_err(|_| "fixture")? != b"synthetic sentinel"
            {
                return Err("OS overwrite confinement failed".into());
            }
        }
        for path in [protected.join("new"), alias.join("new")] {
            if probe(&protected, "/usr/bin/touch", &[path.as_os_str()], b"")?
                .status
                .success()
                || path.exists()
            {
                return Err("OS creation confinement failed".into());
            }
        }
        let positive_a = base.join("swap-positive-a");
        let positive_b = base.join("swap-positive-b");
        std::fs::create_dir(&positive_a).map_err(|_| "fixture")?;
        std::fs::create_dir(&positive_b).map_err(|_| "fixture")?;
        std::fs::write(positive_a.join("marker"), b"positive swap").map_err(|_| "fixture")?;
        require_success(syscall_probe(None, "swap", &positive_a, &positive_b)?)?;
        if !positive_b.join("marker").is_file() {
            return Err(
                "Native swap positive control did not perform the operation; analysis blocked"
                    .into(),
            );
        }
        let clone_source = base.join("clone-positive-source");
        std::fs::create_dir(&clone_source).map_err(|_| "fixture")?;
        std::fs::write(clone_source.join("marker"), b"positive clone").map_err(|_| "fixture")?;
        for mode in ["clonefile", "fclonefileat"] {
            let positive = base.join(format!("positive-{mode}"));
            require_success(syscall_probe(None, mode, &clone_source, &positive)?)?;
            if std::fs::read(positive.join("marker"))
                .map_err(|_| "Clone positive control did not create expected bytes")?
                != b"positive clone"
            {
                return Err("Clone positive control mismatch; analysis blocked".into());
            }
            let allowed = base.join(format!("sibling-{mode}"));
            require_success(syscall_probe(
                Some(&protected),
                mode,
                &clone_source,
                &allowed,
            )?)?;
            for (index, ancestor) in [&parent, &grandparent].into_iter().enumerate() {
                let destination = base.join(format!("denied-{mode}-{index}"));
                require_denial(syscall_probe(
                    Some(&protected),
                    mode,
                    ancestor,
                    &destination,
                )?)?;
                if destination.exists() {
                    return Err("Ancestor clone created a copy; analysis blocked".into());
                }
            }
        }
        for ancestor in [&parent, &grandparent] {
            let moved = base.join("renamed");
            if probe(
                &protected,
                "/bin/mv",
                &[ancestor.as_os_str(), moved.as_os_str()],
                b"",
            )?
            .status
            .success()
                || !sentinel.exists()
            {
                return Err("OS ancestor rename confinement failed".into());
            }
            let swap = base.join("swap");
            std::fs::create_dir(&swap).map_err(|_| "fixture")?;
            require_denial(syscall_probe(Some(&protected), "swap", ancestor, &swap)?)?;
            if !sentinel.exists() {
                return Err("Ancestor swap changed the protected fixture; analysis blocked".into());
            }
            std::fs::remove_dir(&swap).map_err(|_| "fixture")?;
        }
        let sibling = base.join("allowed");
        std::fs::write(&sibling, b"allowed sibling").map_err(|_| "fixture")?;
        let out = probe(&protected, "/bin/cat", &[sibling.as_os_str()], b"")?;
        if !out.status.success() || out.stdout != b"allowed sibling" {
            return Err("OS sibling read preflight failed".into());
        }
        let created = base.join("allowed-created");
        if !probe(&protected, "/usr/bin/touch", &[created.as_os_str()], b"")?
            .status
            .success()
            || !created.is_file()
        {
            return Err("OS sibling write preflight failed".into());
        }
        if !probe(&canonical, "/usr/bin/true", &[], b"")?
            .status
            .success()
        {
            return Err("Selected root sandbox launch failed".into());
        }
        Ok(canonical.clone())
    })();
    let _ = std::fs::remove_dir_all(base);
    result
}
#[cfg(not(target_os = "macos"))]
pub fn preflight(_: &Path) -> Result<PathBuf, String> {
    Err(
        "Confinement requires supported macOS; browsing and manual proposals remain available"
            .into(),
    )
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    #[test]
    fn helper_failures_cannot_masquerade_as_permission_denial() {
        for (status, stdout) in [
            (1, b"".to_vec()),
            (0, b"not-json".to_vec()),
            (0, br#"{"result":-1,"errno":2}"#.to_vec()),
            (0, br#"{"result":0,"errno":0}"#.to_vec()),
        ] {
            assert!(
                require_denial(Output {
                    status: std::process::ExitStatus::from_raw(status << 8),
                    stdout,
                    stderr: vec![]
                })
                .is_err()
            );
        }
        assert!(
            require_success(Output {
                status: std::process::ExitStatus::from_raw(0),
                stdout: br#"{"result":-1,"errno":38}"#.to_vec(),
                stderr: vec![]
            })
            .is_err()
        );
    }
}
