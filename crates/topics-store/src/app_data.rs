use crate::{Result, StoreError};
use std::{ffi::OsString, path::PathBuf};

const APPLICATION_ID: &str = "com.macsdickinson.topics";

/// Resolve the shared CLI/desktop directory without opening configuration or creating files.
///
/// `TOPICS_APP_DATA_DIR` takes precedence over the OS data directory. An empty or
/// relative override is an error, never a reason to fall back to personal data.
/// On macOS the default is `~/Library/Application Support/com.macsdickinson.topics`.
/// Windows uses roaming application data; other platforms use the XDG data directory.
pub fn application_data_dir() -> Result<PathBuf> {
    resolve_environment(
        std::env::var_os("TOPICS_APP_DATA_DIR"),
        std::env::var_os("TOPICS_APP_DATA"),
        std::env::var_os("HOME"),
        || directories::BaseDirs::new().map(|dirs| dirs.data_dir().to_path_buf()),
    )
}

fn resolve_environment(
    override_dir: Option<OsString>,
    obsolete: Option<OsString>,
    explicit_home: Option<OsString>,
    default_base: impl FnOnce() -> Option<PathBuf>,
) -> Result<PathBuf> {
    if obsolete.is_some() {
        return Err(StoreError::Invalid(
            "TOPICS_APP_DATA is obsolete; use TOPICS_APP_DATA_DIR".into(),
        ));
    }
    #[cfg(unix)]
    if override_dir.is_none() && explicit_home.as_ref().is_some_and(|home| home.is_empty()) {
        return Err(StoreError::Invalid(
            "HOME must not be empty; set TOPICS_APP_DATA_DIR explicitly".into(),
        ));
    }
    #[cfg(not(unix))]
    let _ = explicit_home;
    resolve(override_dir, default_base)
}

fn resolve(
    override_dir: Option<OsString>,
    default_base: impl FnOnce() -> Option<PathBuf>,
) -> Result<PathBuf> {
    if let Some(path) = override_dir {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(StoreError::Invalid(
                "TOPICS_APP_DATA_DIR must be a non-empty absolute path".into(),
            ));
        }
        return Ok(path);
    }
    default_base()
        .map(|base| base.join(APPLICATION_ID))
        .ok_or_else(|| {
            StoreError::Invalid(
                "Cannot find application data directory; set TOPICS_APP_DATA_DIR".into(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_is_exact_and_does_not_discover_default() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("synthetic app data");
        assert_eq!(
            resolve(Some(path.clone().into_os_string()), || panic!("fallback")).unwrap(),
            path
        );
        assert!(!path.exists(), "resolution must not create directories");
    }

    #[test]
    fn invalid_override_never_falls_back() {
        for path in ["", "relative/app-data"] {
            assert!(resolve(Some(path.into()), || panic!("fallback")).is_err());
        }
    }

    #[test]
    fn default_uses_tauri_identifier_and_reports_missing_base() {
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(
            resolve(None, || Some(temp.path().to_path_buf())).unwrap(),
            temp.path().join("com.macsdickinson.topics")
        );
        assert!(resolve(None, || None).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn empty_home_never_calls_os_fallback_without_override() {
        // Pure injected inputs: never invoke OS discovery with an empty HOME.
        assert!(
            resolve_environment(None, None, Some("".into()), || panic!(
                "OS fallback must not run"
            ))
            .is_err()
        );
    }
    #[test]
    fn obsolete_environment_never_falls_back_even_with_new_override() {
        let temp = tempfile::tempdir().unwrap();
        assert!(
            resolve_environment(
                Some(temp.path().join("safe").into_os_string()),
                Some("obsolete".into()),
                None,
                || panic!("fallback")
            )
            .is_err()
        );
    }
}
