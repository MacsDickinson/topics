//! Synthetic protocol fixture construction shared by tests and the developer example.
//! Never imported by the production setup library.
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

pub fn render(directory: &Path, mode: &str) -> io::Result<Value> {
    if !directory.is_absolute()
        || directory
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Use an existing absolute empty synthetic directory",
        ));
    }
    let metadata = fs::symlink_metadata(directory)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || fs::read_dir(directory)?.next().is_some()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Fixture output must be a caller-created empty directory, not a symlink",
        ));
    }
    if ![
        "graph",
        "graph_claim",
        "rehearsal_claim",
        "empty",
        "ambiguous",
        "split_calls",
        "fail_first",
        "hold_second",
        "pronouns",
        "historic",
        "no_read",
        "candidate_schema",
        "host_denied",
        "malformed",
        "bad_callback",
        "wrong_version",
    ]
    .contains(&mode)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unknown synthetic fixture mode",
        ));
    }
    let directory = directory.canonicalize()?;
    if directory.to_str().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Fixture path must be UTF-8",
        ));
    }
    let executable = directory.join("scripted-codex");
    let mode_file = directory.join("mode");
    let log_file = directory.join("transmissions.jsonl");
    let source = include_str!("scripted_child.py")
        .replace("@SETTINGS@", &serde_json::to_string(&mode_file)?)
        .replace("@LOG@", &serde_json::to_string(&log_file)?)
        .replace(
            "@FLAGS@",
            &serde_json::to_string(topics_runtime::protocol::DISABLED)?,
        );
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o700);
    }
    options.open(&executable)?.write_all(source.as_bytes())?;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&mode_file)?
        .write_all(mode.as_bytes())?;
    Ok(json!({"executable":executable,"mode_file":mode_file,"log_file":log_file,"mode":mode}))
}
