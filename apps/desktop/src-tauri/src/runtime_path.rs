use std::path::{Path, PathBuf};
/// Resolve executable metadata only. Credentials and runtime configuration remain Codex-owned.
pub fn resolve(
    override_path: Option<&Path>,
    home: Option<&Path>,
    is_file: impl Fn(&Path) -> bool,
) -> Result<PathBuf, String> {
    resolve_for_platform(
        override_path,
        home,
        std::env::consts::OS,
        std::env::consts::ARCH,
        is_file,
    )
}

fn resolve_for_platform(
    override_path: Option<&Path>,
    home: Option<&Path>,
    os: &str,
    arch: &str,
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
        if os == "macos" && matches!(arch, "aarch64" | "x86_64") {
            candidates.push(home.join(format!(
                ".codex/packages/standalone/releases/{}-{arch}-apple-darwin/bin/codex",
                topics_runtime::CODEX_VERSION,
            )));
        }
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
    #[cfg(all(
        target_os = "macos",
        any(target_arch = "aarch64", target_arch = "x86_64")
    ))]
    fn pinned_retained_executable_precedes_local_latest() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path();
        let pinned = home.join(format!(
            ".codex/packages/standalone/releases/{}-{}-apple-darwin/bin/codex",
            topics_runtime::CODEX_VERSION,
            std::env::consts::ARCH,
        ));
        let latest = home.join(".local/bin/codex");
        for path in [&pinned, &latest] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"synthetic executable metadata").unwrap();
        }
        assert_eq!(resolve(None, Some(home), Path::is_file).unwrap(), pinned);
    }
    #[test]
    fn discovery_checks_only_exact_known_paths_in_order_for_each_platform() {
        use std::cell::RefCell;
        let home = Path::new("/synthetic/home");
        for arch in ["aarch64", "x86_64"] {
            let pinned = home.join(format!(
                ".codex/packages/standalone/releases/{}-{arch}-apple-darwin/bin/codex",
                topics_runtime::CODEX_VERSION,
            ));
            let local = home.join(".local/bin/codex");
            let checked = RefCell::new(Vec::new());
            let result = resolve_for_platform(None, Some(home), "macos", arch, |path| {
                checked.borrow_mut().push(path.to_owned());
                path == pinned || path == local
            })
            .unwrap();
            assert_eq!(result, pinned);
            assert_eq!(*checked.borrow(), vec![pinned.clone()]);
            checked.borrow_mut().clear();
            let result = resolve_for_platform(None, Some(home), "macos", arch, |path| {
                checked.borrow_mut().push(path.to_owned());
                path == local
            })
            .unwrap();
            assert_eq!(result, local);
            assert_eq!(*checked.borrow(), vec![pinned.clone(), local.clone()]);
            checked.borrow_mut().clear();
            assert_eq!(
                resolve_for_platform(None, Some(home), "macos", arch, |path| {
                    checked.borrow_mut().push(path.to_owned());
                    path == Path::new("/usr/local/bin/codex")
                })
                .unwrap(),
                PathBuf::from("/usr/local/bin/codex")
            );
            assert_eq!(
                *checked.borrow(),
                vec![
                    pinned,
                    local,
                    PathBuf::from("/opt/homebrew/bin/codex"),
                    PathBuf::from("/usr/local/bin/codex")
                ]
            );
        }
        let checked = RefCell::new(Vec::new());
        assert_eq!(
            resolve_for_platform(None, Some(home), "macos", "unsupported", |path| {
                checked.borrow_mut().push(path.to_owned());
                false
            })
            .unwrap(),
            PathBuf::from("codex")
        );
        assert_eq!(
            *checked.borrow(),
            vec![
                home.join(".local/bin/codex"),
                PathBuf::from("/opt/homebrew/bin/codex"),
                PathBuf::from("/usr/local/bin/codex")
            ]
        );
        assert_eq!(
            resolve_for_platform(None, None, "macos", "aarch64", |path| path
                == Path::new("/opt/homebrew/bin/codex"))
            .unwrap(),
            PathBuf::from("/opt/homebrew/bin/codex")
        );
        assert_eq!(
            resolve_for_platform(None, Some(home), "linux", "x86_64", |path| path
                == home.join(".local/bin/codex"))
            .unwrap(),
            home.join(".local/bin/codex")
        );
    }
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
            resolve(
                Some(Path::new("/synthetic/bin/codex")),
                Some(home),
                |_| panic!("explicit override needs no default discovery")
            )
            .unwrap(),
            PathBuf::from("/synthetic/bin/codex")
        );
        assert!(
            resolve(Some(Path::new("relative")), Some(home), |_| panic!(
                "invalid override must not discover defaults"
            ))
            .is_err()
        );
    }
}
