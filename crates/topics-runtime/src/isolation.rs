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
            "(deny file-write-unlink(literal(param \"{key}\")))"
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
            let code = "import ctypes,sys;lib=ctypes.CDLL(None,use_errno=True);r=lib.renameatx_np(-2,sys.argv[1].encode(),-2,sys.argv[2].encode(),2);sys.exit(0 if r==0 else 1)";
            if probe(
                &protected,
                "/usr/bin/python3",
                &[
                    std::ffi::OsStr::new("-I"),
                    std::ffi::OsStr::new("-c"),
                    std::ffi::OsStr::new(code),
                    ancestor.as_os_str(),
                    swap.as_os_str(),
                ],
                b"",
            )?
            .status
            .success()
                || !sentinel.exists()
            {
                return Err("OS ancestor RENAME_SWAP confinement failed".into());
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
