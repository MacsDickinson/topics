//! Path-scoped macOS denial. This does not protect hardlinks/copies outside the root.
use std::{path::{Path, PathBuf}, process::{Command, Output}};
pub const PROFILE: &str = "(version 1)(allow default)(deny file-read-data(subpath(param \"TOPICS_ROOT\")))(deny file-write*(subpath(param \"TOPICS_ROOT\")))";
#[cfg(target_os="macos")]
pub fn command(root: &Path, executable: &Path) -> Command {
    let mut cmd = Command::new("/usr/bin/sandbox-exec");
    // Parameter is a separate argv; paths never become Scheme code.
    cmd.arg("-D").arg(format!("TOPICS_ROOT={}", root.display())).arg("-p").arg(PROFILE).arg(executable); cmd
}
#[cfg(not(target_os="macos"))]
pub fn command(_: &Path, executable: &Path) -> Command { Command::new(executable) }
#[cfg(target_os="macos")]
fn probe(root: &Path, exe: &str, args: &[&std::ffi::OsStr]) -> Result<Output,String> {
    command(root, Path::new(exe)).args(args).output().map_err(|_| "macOS sandbox-exec unavailable; install supported macOS runtime".into())
}
/// Tests the exact profile on an app-owned synthetic fixture, never live root bytes.
#[cfg(target_os="macos")]
pub fn preflight(root: &Path) -> Result<PathBuf,String> {
    use std::os::unix::fs::symlink;
    let canonical = root.canonicalize().map_err(|_| "Selected notes root is unavailable; select it again".to_string())?;
    if !canonical.is_dir() { return Err("Selected root must be a directory".into()); }
    let base = std::env::temp_dir().join(format!("topics-confinement-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&base).map_err(|_| "Cannot create sandbox fixture".to_string())?;
    let result = (|| {
        let protected = base.join("root ' ü"); std::fs::create_dir(&protected).map_err(|_| "fixture".to_string())?;
        let protected = protected.canonicalize().map_err(|_| "fixture".to_string())?;
        let sentinel = protected.join("sentinel"); std::fs::write(&sentinel,b"synthetic sentinel").map_err(|_| "fixture".to_string())?;
        let alias = base.join("alias"); symlink(&protected,&alias).map_err(|_| "fixture".to_string())?;
        let sibling = base.join("allowed"); std::fs::write(&sibling,b"allowed sibling").map_err(|_| "fixture".to_string())?;
        for path in [sentinel, alias.join("sentinel")] {
            if probe(&protected,"/bin/cat", &[path.as_os_str()])?.status.success() { return Err("OS read confinement failed; analysis blocked".into()); }
        }
        for path in [protected.join("new"), alias.join("new")] {
            if probe(&protected,"/usr/bin/touch", &[path.as_os_str()])?.status.success() || path.exists() { return Err("OS write confinement failed; analysis blocked".into()); }
        }
        let allowed = probe(&protected,"/bin/cat", &[sibling.as_os_str()])?;
        if !allowed.status.success() || allowed.stdout != b"allowed sibling" { return Err("OS sandbox unsupported; analysis blocked".into()); }
        if !probe(&canonical,"/usr/bin/true", &[])?.status.success() { return Err("Selected root sandbox launch failed".into()); }
        Ok(canonical.clone())
    })();
    let _ = std::fs::remove_dir_all(&base); result
}
#[cfg(not(target_os="macos"))]
pub fn preflight(_: &Path) -> Result<PathBuf,String> { Err("OS confinement requires supported macOS; browsing and manual proposals remain available".into()) }
