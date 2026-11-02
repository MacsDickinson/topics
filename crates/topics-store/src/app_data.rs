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
    resolve(std::env::var_os("TOPICS_APP_DATA_DIR"), || {
        directories::BaseDirs::new().map(|dirs| dirs.data_dir().to_path_buf())
    })
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
}
