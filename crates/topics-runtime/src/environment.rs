//! Only minimal runtime-owned environment crosses into the installed child.
//! Never enumerate/copy ambient values: tracing/auth/proxy/CA/provider knobs are
//! excluded by construction, including knobs added by later Codex versions.
use crate::error;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};
use topics_core::workspace::WorkspaceResult;
fn directory(value: OsString, name: &str) -> WorkspaceResult<PathBuf> {
    let path = Path::new(&value);
    if !path.is_absolute() {
        return Err(error(
            "runtime_unavailable",
            &format!(
                "{name} must name an existing absolute directory; remove the override or launch Topics from your normal account"
            ),
        ));
    }
    let canonical=path.canonicalize().map_err(|_|error("runtime_unavailable",&format!("{name} directory is unavailable; remove the override or configure an existing runtime directory")))?;
    if !canonical.is_dir() || canonical.to_str().is_none() {
        return Err(error(
            "runtime_unavailable",
            &format!(
                "{name} must be an existing UTF-8 directory; correct the launch environment and retry"
            ),
        ));
    }
    Ok(canonical)
}
fn apply_from(
    command: &mut Command,
    get: impl Fn(&str) -> Option<OsString>,
) -> WorkspaceResult<()> {
    command.env_clear();
    let home = get("HOME").ok_or_else(|| {
        error(
            "runtime_unavailable",
            "HOME is missing; launch Topics from your normal account",
        )
    })?;
    let home = directory(home, "HOME")?;
    let codex_home = get("CODEX_HOME")
        .map(|value| directory(value, "CODEX_HOME"))
        .transpose()?;
    // Codex reloads this dotenv file after env_clear. lstat rejects even dangling
    // symlinks without opening content; only genuine absence permits launch.
    // A same-user replacement after this metadata gate remains outside the boundary.
    let dotenv = codex_home
        .clone()
        .unwrap_or_else(|| home.join(".codex"))
        .join(".env");
    match std::fs::symlink_metadata(dotenv) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        _ => {
            return Err(error(
                "runtime_unavailable",
                "This runtime session does not support Codex dotenv configuration. Choose an appropriate existing CODEX_HOME without a .env entry, then retry",
            ));
        }
    }
    command.env("HOME", home);
    if let Some(home) = codex_home {
        command.env("CODEX_HOME", home);
    }
    command.env(
        "PATH",
        get("PATH").unwrap_or_else(|| "/usr/bin:/bin".into()),
    );
    if let Some(temp) = get("TMPDIR") {
        command.env("TMPDIR", directory(temp, "TMPDIR")?);
    }
    for name in ["LANG", "LC_ALL", "LC_CTYPE"] {
        if let Some(value) = get(name)
            && value.to_str().is_some_and(|s| {
                !s.is_empty()
                    && s.len() <= 80
                    && s.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-.@".contains(&b))
            })
        {
            command.env(name, value);
        }
    }
    command.env("RUST_LOG", "off");
    Ok(())
}
pub(crate) fn apply(command: &mut Command) -> WorkspaceResult<()> {
    apply_from(command, |name| std::env::var_os(name))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn apply_with_home(home: &Path, explicit: Option<&Path>) -> WorkspaceResult<()> {
        apply_from(&mut Command::new("/usr/bin/true"), |key| match key {
            "HOME" => Some(home.as_os_str().to_owned()),
            "CODEX_HOME" => explicit.map(|p| p.as_os_str().to_owned()),
            _ => None,
        })
    }
    #[test]
    fn dotenv_existing_file_or_directory_blocks_default_and_explicit_home() {
        for explicit in [false, true] {
            for is_directory in [false, true] {
                let temp = tempfile::tempdir().unwrap();
                let runtime_home = temp.path().join(".codex");
                std::fs::create_dir(&runtime_home).unwrap();
                let dotenv = runtime_home.join(".env");
                if is_directory {
                    std::fs::create_dir(&dotenv).unwrap();
                } else {
                    std::fs::write(&dotenv, b"SYNTHETIC_UNSUPPORTED_DOTENV=1\n").unwrap();
                }
                let error =
                    apply_with_home(temp.path(), explicit.then_some(runtime_home.as_path()))
                        .expect_err("any existing dotenv entry must prevent child launch");
                assert!(
                    error
                        .message
                        .contains("does not support Codex dotenv configuration")
                );
                assert!(!error.message.contains("SYNTHETIC_UNSUPPORTED"));
                assert!(std::fs::symlink_metadata(dotenv).is_ok());
            }
        }
    }
    #[cfg(unix)]
    #[test]
    fn dotenv_symlinks_including_dangling_links_block_launch() {
        for dangling in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let target = temp.path().join("target");
            if !dangling {
                std::fs::write(&target, b"synthetic\n").unwrap();
            }
            std::os::unix::fs::symlink(&target, temp.path().join(".env")).unwrap();
            assert!(apply_with_home(temp.path(), Some(temp.path())).is_err());
            assert!(std::fs::symlink_metadata(temp.path().join(".env")).is_ok());
        }
    }
    #[test]
    fn dotenv_absent_allows_default_missing_or_existing_home_and_explicit_home() {
        let temp = tempfile::tempdir().unwrap();
        assert!(apply_with_home(temp.path(), None).is_ok());
        assert!(!temp.path().join(".codex").exists());
        let runtime_home = temp.path().join(".codex");
        std::fs::create_dir(&runtime_home).unwrap();
        assert!(apply_with_home(temp.path(), None).is_ok());
        assert!(apply_with_home(temp.path(), Some(&runtime_home)).is_ok());
    }
    #[test]
    fn dotenv_metadata_error_is_not_treated_as_absence() {
        let temp = tempfile::tempdir().unwrap();
        // ENOTDIR is a deterministic metadata failure even under privileged test runners.
        std::fs::write(temp.path().join(".codex"), b"synthetic obstruction").unwrap();
        assert!(apply_with_home(temp.path(), None).is_err());
    }
    #[test]
    fn only_small_validated_allowlist_is_copied() {
        let temp = tempfile::tempdir().unwrap();
        let values = std::collections::HashMap::from([
            ("HOME", temp.path().as_os_str().to_owned()),
            ("CODEX_HOME", temp.path().as_os_str().to_owned()),
            ("PATH", OsString::from("/usr/bin:/bin")),
            ("LANG", OsString::from("en_GB.UTF-8")),
            (
                "CODEX_ROLLOUT_TRACE_ROOT",
                OsString::from("/synthetic/trace"),
            ),
            ("CODEX_CA_CERTIFICATE", OsString::from("/synthetic/ca")),
            ("SSLKEYLOGFILE", OsString::from("/synthetic/keys")),
            (
                "CODEX_APP_SERVER_CHATGPT_BASE_URL",
                OsString::from("http://127.0.0.1:1"),
            ),
            (
                "CODEX_AUTHAPI_BASE_URL",
                OsString::from("http://127.0.0.1:1"),
            ),
            ("CODEX_ACCESS_TOKEN", OsString::from("synthetic")),
            ("OPENAI_ORGANIZATION", OsString::from("synthetic")),
            ("OPENAI_PROJECT", OsString::from("synthetic")),
        ]);
        let mut cmd = Command::new("/usr/bin/true");
        cmd.env("SSLKEYLOGFILE", "/synthetic/preexisting");
        apply_from(&mut cmd, |key| {
            assert!(
                [
                    "HOME",
                    "CODEX_HOME",
                    "PATH",
                    "TMPDIR",
                    "LANG",
                    "LC_ALL",
                    "LC_CTYPE"
                ]
                .contains(&key)
            );
            values.get(key).cloned()
        })
        .unwrap();
        let keys = cmd
            .get_envs()
            .map(|(key, _)| key.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert!(keys.iter().all(|key| {
            [
                "HOME",
                "CODEX_HOME",
                "PATH",
                "TMPDIR",
                "LANG",
                "LC_ALL",
                "LC_CTYPE",
                "RUST_LOG",
            ]
            .contains(&key.as_str())
        }));
        assert_eq!(
            cmd.get_envs()
                .find(|(key, _)| *key == "RUST_LOG")
                .unwrap()
                .1,
            Some(std::ffi::OsStr::new("off"))
        );
    }
    #[test]
    fn invalid_codex_home_fails_closed_without_creating_it() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("missing");
        let mut command = Command::new("/usr/bin/true");
        assert!(
            apply_from(&mut command, |key| match key {
                "HOME" => Some(temp.path().as_os_str().to_owned()),
                "CODEX_HOME" => Some(missing.as_os_str().to_owned()),
                _ => None,
            })
            .is_err()
        );
        assert!(!missing.exists());
    }
}
