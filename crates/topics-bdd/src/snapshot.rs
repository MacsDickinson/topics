//! Read-only contract: include the root, empty directories, files and symlinks.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug, PartialEq, Eq)]
enum EntryContents {
    Directory,
    File(Vec<u8>),
    Symlink(PathBuf),
    Other,
}

#[derive(Debug, PartialEq, Eq)]
struct Entry {
    modified: SystemTime,
    contents: EntryContents,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Snapshot(BTreeMap<PathBuf, Entry>);

impl Snapshot {
    pub(crate) fn capture(root: &Path) -> Self {
        fn collect(root: &Path, path: &Path, entries: &mut BTreeMap<PathBuf, Entry>) {
            // symlink_metadata inspects the link itself; never traverse its target.
            let metadata = fs::symlink_metadata(path).unwrap();
            let kind = metadata.file_type();
            let contents = if kind.is_symlink() {
                EntryContents::Symlink(fs::read_link(path).unwrap())
            } else if kind.is_dir() {
                EntryContents::Directory
            } else if kind.is_file() {
                EntryContents::File(fs::read(path).unwrap())
            } else {
                EntryContents::Other
            };
            entries.insert(
                path.strip_prefix(root).unwrap().to_owned(),
                Entry {
                    modified: metadata.modified().unwrap(),
                    contents,
                },
            );
            if kind.is_dir() {
                for entry in fs::read_dir(path).unwrap() {
                    collect(root, &entry.unwrap().path(), entries);
                }
            }
        }
        let mut entries = BTreeMap::new();
        collect(root, root, &mut entries);
        Self(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extra_empty_directory_changes_snapshot() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("note.md"), "unchanged").unwrap();
        let before = Snapshot::capture(root.path());
        fs::create_dir(root.path().join("unexpected")).unwrap();
        let after = Snapshot::capture(root.path());
        // Check the entry itself, independently of the parent's changed mtime.
        assert_eq!(
            after.0[Path::new("unexpected")].contents,
            EntryContents::Directory
        );
        assert!(!before.0.contains_key(Path::new("unexpected")));
        assert_ne!(before, after);
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_recorded_without_following_them() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("link")).unwrap();
        let before = Snapshot::capture(root.path());
        fs::write(outside.path().join("outside.md"), "outside fixture").unwrap();
        assert_eq!(before, Snapshot::capture(root.path()));
        assert_eq!(before.0.len(), 2); // Root plus the link, no target entries.
    }
}
