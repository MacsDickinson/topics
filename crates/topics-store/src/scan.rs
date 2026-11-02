//! Source traversal. Only index() / changed watcher snapshots read note bodies.
use crate::{Result, StoreError, parse};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path, time::UNIX_EPOCH};
use topics_core::Document;

type Records = Vec<(Document, String)>;

fn available(root: &Path) -> Result<()> {
    fs::read_dir(root)
        .map_err(|error| StoreError::Invalid(format!("Notes root unavailable: {error}")))?;
    Ok(())
}
fn relative(path: &Path, root: &Path) -> Result<String> {
    Ok(path
        .strip_prefix(root)
        .map_err(|e| StoreError::Invalid(e.to_string()))?
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

pub(crate) fn scan(root: &Path) -> Result<(Records, Vec<String>)> {
    available(root)?;
    let mut records = Vec::new();
    let mut warnings = Vec::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                let path = error
                    .path()
                    .and_then(|p| relative(p, root).ok())
                    .unwrap_or_default();
                warnings.push(format!("{path}: cannot traverse notes directory"));
                continue;
            }
        };
        if !entry.file_type().is_file() || entry.path().extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        let path = relative(entry.path(), root)?;
        let mut bytes = Vec::new();
        let mut modified = "0".to_string();
        let mut plain = false;
        // Metadata belongs to the opened object, not a pathname that can vanish
        // after the read. Per-file errors never abort the rest of a valid root.
        match fs::File::open(entry.path()) {
            Ok(mut file) => {
                match file.metadata() {
                    Ok(metadata) => {
                        if !metadata.is_file() {
                            warnings.push(format!("{path}: no longer a regular file"));
                            continue;
                        }
                        modified = metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .unwrap_or_default()
                            .as_nanos()
                            .to_string();
                    }
                    Err(error) => {
                        warnings.push(format!("{path}: metadata unavailable: {error}"));
                        plain = true;
                    }
                }
                if let Err(error) = file.read_to_end(&mut bytes) {
                    warnings.push(format!(
                        "{path}: unreadable; indexed as a plain document: {error}"
                    ));
                    bytes.clear();
                    plain = true;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                warnings.push(format!("{path}: removed during scan"));
                continue;
            }
            Err(error) => {
                warnings.push(format!(
                    "{path}: unreadable; indexed as a plain document: {error}"
                ));
                plain = true;
            }
        }
        let body = match String::from_utf8(bytes.clone()) {
            Ok(body) => body,
            Err(_) => {
                warnings.push(format!(
                    "{path}: invalid UTF-8; indexed as a plain document"
                ));
                plain = true;
                String::from_utf8_lossy(&bytes).into_owned()
            }
        };
        let mut document = parse::document(
            &path,
            &body,
            format!("{:x}", Sha256::digest(&bytes)),
            modified,
        );
        if plain {
            document.kind = "other".into();
            document.date = None;
        }
        records.push((document, body));
    }
    // Detect a root lost or made unreadable during traversal before publishing.
    available(root)?;
    records.sort_by(|a, b| a.0.path.cmp(&b.0.path));
    Ok((records, warnings))
}

pub(crate) fn snapshot(root: &Path) -> Result<Vec<String>> {
    available(root)?;
    let mut entries = vec![root.to_string_lossy().into_owned()];
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        match entry {
            Ok(entry)
                if entry.file_type().is_file()
                    && entry.path().extension().is_some_and(|ext| ext == "md") =>
            {
                let path = relative(entry.path(), root)?;
                match entry.metadata() {
                    Ok(metadata) => {
                        let mut stamp = format!(
                            "{path}:{:?}:{}:{:?}",
                            metadata.modified().ok(),
                            metadata.len(),
                            metadata.permissions()
                        );
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::MetadataExt;
                            // Catch same-size / restored-mtime writes and inode replacements.
                            stamp.push_str(&format!(
                                ":{}:{}:{}",
                                metadata.ctime(),
                                metadata.ctime_nsec(),
                                metadata.ino()
                            ));
                        }
                        entries.push(stamp);
                    }
                    Err(_) => entries.push(format!("{path}:metadata unavailable")),
                }
            }
            Err(error) => entries.push(format!(
                "{}:traversal unavailable",
                error
                    .path()
                    .and_then(|p| relative(p, root).ok())
                    .unwrap_or_default()
            )),
            _ => {}
        }
    }
    entries.sort();
    Ok(entries)
}
