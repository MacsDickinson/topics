use std::path::{Path, PathBuf};
/// Resolve executable metadata only. Credentials and runtime configuration remain Codex-owned.
pub fn resolve(
    override_path: Option<&Path>,
    home: Option<&Path>,
    is_file: impl Fn(&Path) -> bool,
) -> Result<PathBuf, String> {
    if let Some(path) = override_path {
        if !path.is_absolute() {
            return Err(
                "TOPICS_CODEX_PATH must be an absolute path to the installed Codex executable."
                    .into(),
            );
        }
        return Ok(path.to_owned());
    }
    let mut candidates = Vec::new();
    if let Some(home) = home {
        candidates.push(home.join(".local/bin/codex"));
    }
    candidates.extend([
        PathBuf::from("/opt/homebrew/bin/codex"),
        PathBuf::from("/usr/local/bin/codex"),
    ]);
    Ok(candidates
        .into_iter()
        .find(|path| is_file(path))
        .unwrap_or_else(|| PathBuf::from("codex")))
}
pub fn installed() -> Result<PathBuf, String> {
    #[cfg(all(debug_assertions, feature = "native-test"))]
    if let Some(path) = std::env::var_os("TOPICS_TEST_CODEX") {
        return Ok(PathBuf::from(path));
    }
    let override_path = std::env::var_os("TOPICS_CODEX_PATH").map(PathBuf::from);
    let home = std::env::var_os("HOME").map(PathBuf::from);
    resolve(override_path.as_deref(), home.as_deref(), Path::is_file)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finder_discovery_only_checks_known_executable_metadata() {
        let home = Path::new("/synthetic/home");
        assert_eq!(
            resolve(None, Some(home), |path| path
                == home.join(".local/bin/codex"))
            .unwrap(),
            home.join(".local/bin/codex")
        );
        assert_eq!(
            resolve(None, Some(home), |_| false).unwrap(),
            PathBuf::from("codex")
        );
        assert_eq!(
            resolve(Some(Path::new("/synthetic/bin/codex")), None, |_| panic!(
                "explicit override needs no default discovery"
            ))
            .unwrap(),
            PathBuf::from("/synthetic/bin/codex")
        );
        assert!(
            resolve(Some(Path::new("relative")), None, |_| panic!(
                "invalid override must not discover defaults"
            ))
            .is_err()
        );
    }
}
