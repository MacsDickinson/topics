use std::collections::BTreeMap;
use topics_core::{Document, Person};

pub fn document(path: &str, body: &str, hash: String, modified_ns: String) -> Document {
    let kind = if path.starts_with("memory/people/") {
        "person"
    } else if path.starts_with("memory/projects/") {
        "project"
    } else if path.starts_with("Meetings/") {
        "meeting"
    } else if path.starts_with("Log/") {
        "log"
    } else {
        "other"
    };
    let file = path
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .trim_end_matches(".md");
    let mut title = body
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .unwrap_or(file)
        .trim()
        .to_string();
    let filename_date = if kind == "meeting" || kind == "log" {
        date_prefix(file)
    } else {
        None
    };
    let date = filename_date.clone().or_else(|| {
        // Explicit date frontmatter is indexed too; unsupported dates remain undated.
        body.strip_prefix("---\n")
            .and_then(|s| s.split_once("\n---"))
            .and_then(|(front, _)| {
                front.lines().find_map(|line| {
                    let d = line.strip_prefix("date: ")?.trim();
                    (d.len() == 10).then(|| date_prefix(d)).flatten()
                })
            })
    });
    if kind == "meeting" && filename_date.is_some() {
        // Convention: YYYY-MM-DD-HH-MM-title.md; date-only names also work.
        let suffix = file.get(11..).unwrap_or("");
        let parts: Vec<_> = suffix.split('-').collect();
        let title_parts = if parts.len() > 2
            && parts[0].parse::<u8>().is_ok_and(|hour| hour < 24)
            && parts[1].parse::<u8>().is_ok_and(|minute| minute < 60)
        {
            &parts[2..]
        } else {
            &parts[..]
        };
        let from_file = title_parts.join(" ");
        if !from_file.is_empty() {
            title = from_file;
        }
    }
    Document {
        path: path.into(),
        title,
        kind: kind.into(),
        date,
        hash,
        modified_ns,
    }
}
fn date_prefix(file: &str) -> Option<String> {
    let prefix = file.get(..10)?;
    let bytes = prefix.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year: u32 = prefix[..4].parse().ok()?;
    let month: u32 = prefix[5..7].parse().ok()?;
    let day: u32 = prefix[8..].parse().ok()?;
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        _ => return None,
    };
    (year > 0 && day > 0 && day <= days).then(|| prefix.to_string())
}
pub fn aliases(body: &str) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut nicknames = false;
    for line in body.lines() {
        if line.starts_with('#') {
            nicknames = line
                .trim_start_matches('#')
                .trim()
                .eq_ignore_ascii_case("Nicknames");
            continue;
        }
        if !nicknames {
            continue;
        }
        let cells: Vec<_> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cells.len() >= 2
            && !cells[0].is_empty()
            && cells[0] != "Nickname"
            && !cells[0].starts_with('-')
        {
            out.entry(cells[1].to_string())
                .or_default()
                .push(cells[0].to_string());
        }
    }
    out
}
pub fn person(d: &Document, body: &str, aliases: &BTreeMap<String, Vec<String>>) -> Person {
    let field = |name: &str| {
        body.lines().find_map(|line| {
            let line = line.trim().trim_start_matches('-').trim();
            line.strip_prefix(&format!("**{name}:**"))
                .map(|value| value.trim().to_string())
        })
    };
    Person {
        name: d.title.clone(),
        path: d.path.clone(),
        known_as: field("Known as"),
        pronouns: field("Pronouns"),
        role: field("Role"),
        team: field("Team"),
        aliases: aliases.get(&d.title).cloned().unwrap_or_default(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn meeting_metadata_comes_from_filename() {
        let d = document(
            "Meetings/2026-10-28-16-05-alex-121.md",
            "# Different heading",
            "hash".into(),
            "0".into(),
        );
        assert_eq!(d.date.as_deref(), Some("2026-10-28"));
        assert_eq!(d.title, "alex 121");
    }
    #[test]
    fn malformed_and_unicode_dates_do_not_panic_or_become_dates() {
        for file in [
            "2026-02-30",
            "2026-13-01",
            "2026-00-01",
            "2026-01-00",
            "🦀🦀🦀.md",
            "2026-1-1",
            "0000-01-01",
        ] {
            assert_eq!(date_prefix(file), None, "{file}");
        }
        assert_eq!(date_prefix("2024-02-29-meeting"), Some("2024-02-29".into()));
    }
}

#[cfg(test)]
mod workspace_date_tests {
    use super::*;
    #[test]
    fn indexed_frontmatter_date_is_validated_without_changing_arbitrary_titles() {
        let d = document(
            "Meetings/arbitrary-meeting-name.md",
            "---\ndate: 2026-11-01\n---\n# Explicit title\n",
            "h".into(),
            "0".into(),
        );
        assert_eq!(d.date.as_deref(), Some("2026-11-01"));
        assert_eq!(d.title, "Explicit title");
        for date in ["2026-02-30", "2026-11-01 extra", "yesterday", "2026-1-1"] {
            let d = document(
                "folder/file.md",
                &format!("---\ndate: {date}\n---\n# Note\n"),
                "h".into(),
                "0".into(),
            );
            assert!(d.date.is_none());
        }
        let d = document(
            "Meetings/2026-11-03-session.md",
            "---\ndate: 2026-11-01\n---\n# Title\n",
            "h".into(),
            "0".into(),
        );
        assert_eq!(d.date.as_deref(), Some("2026-11-03"));
    }
}
