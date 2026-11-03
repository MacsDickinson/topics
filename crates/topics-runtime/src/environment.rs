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
    command.env("HOME", directory(home, "HOME")?);
    if let Some(home) = get("CODEX_HOME") {
        command.env("CODEX_HOME", directory(home, "CODEX_HOME")?);
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
