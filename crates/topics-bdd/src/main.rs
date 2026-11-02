use cucumber::{World, given, then, when};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};
use topics_store::{SearchResult, TopicsService, WatchHandle};

type Snapshot = BTreeMap<PathBuf, (Vec<u8>, SystemTime)>;
#[derive(World)]
#[world(init = Self::new)]
struct Notes {
    temp: tempfile::TempDir,
    service: TopicsService,
    root: PathBuf,
    before: Snapshot,
    results: Vec<SearchResult>,
    error: String,
    watcher: Option<WatchHandle>,
}
impl std::fmt::Debug for Notes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Synthetic notes world")
    }
}
impl Notes {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let service = TopicsService::open(temp.path().join("app-data")).unwrap();
        let root = temp.path().join("notes");
        Self {
            temp,
            service,
            root,
            before: BTreeMap::new(),
            results: vec![],
            error: String::new(),
            watcher: None,
        }
    }
    fn write(&self, path: &str, bytes: impl AsRef<[u8]>) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    fn snapshot(&self) -> Snapshot {
        fn collect(path: &std::path::Path, out: &mut Snapshot) {
            for entry in fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    collect(&path, out);
                } else {
                    out.insert(
                        path.clone(),
                        (
                            fs::read(&path).unwrap(),
                            fs::metadata(path).unwrap().modified().unwrap(),
                        ),
                    );
                }
            }
        }
        let mut out = BTreeMap::new();
        collect(&self.root, &mut out);
        out
    }
    fn files(&mut self, name: &str, count: usize) {
        self.root = self.temp.path().join(name);
        self.write(
            "memory/projects/ledger-migration.md",
            "# Ledger migration\nLedger work",
        );
        for i in 1..count {
            self.write(&format!("document-{i}.md"), format!("# {name} {i}"));
        }
        self.before = self.snapshot();
    }
}
fn quoted(text: &str) -> Vec<&str> {
    text.split('"').skip(1).step_by(2).collect()
}
#[given(regex = ".*")]
#[when(regex = ".*")]
#[then(regex = ".*")]
fn execute_step(w: &mut Notes, step: &cucumber::gherkin::Step) {
    let s = step.value.as_str();
    let q = quoted(s);
    if s == "a notes root with these files:" {
        for row in step.table.as_ref().unwrap().rows.iter().skip(1) {
            w.write(
                &row[0],
                row[1..].join("|").replace("\\|", "|").replace("\\n", "\n"),
            );
        }
        w.before = w.snapshot();
        w.service.use_notes_root(&w.root).unwrap();
    } else if s.starts_with("a folder ") || s.starts_with("the notes root is ") {
        let count = s
            .split_whitespace()
            .find_map(|x| x.parse::<usize>().ok())
            .unwrap();
        let previous = w.root.clone();
        w.files(q[0], count);
        if s.starts_with("the notes root is") {
            w.service.use_notes_root(&w.root).unwrap();
        } else if w.service.notes_root().unwrap().is_some() {
            w.root = previous;
        }
    } else if s.starts_with("a notes root with 3") {
        w.files("notes", 3);
        w.service.use_notes_root(&w.root).unwrap();
    } else if s.starts_with("the folder ") && s.ends_with("does not exist") {
        w.root = w.temp.path().join(q[0]);
        assert!(!w.root.exists());
    } else if s.starts_with("I set the notes root") {
        w.root = w.temp.path().join(q[0]);
        w.service.use_notes_root(&w.root).unwrap();
    } else if s.starts_with("I initialise") {
        let result = w.service.init_notes_root(w.temp.path().join(q[0]));
        w.error = result.err().map(|e| e.to_string()).unwrap_or_default();
    } else if s.starts_with("the notes root setting") {
        assert_eq!(
            w.service.notes_root().unwrap().unwrap(),
            fs::canonicalize(w.temp.path().join(q[0])).unwrap()
        );
    } else if s.contains("contains the folders") {
        for folder in &q[1..] {
            assert!(w.temp.path().join(q[0]).join(folder).is_dir());
        }
    } else if s == "I am told the folder is not empty and to use it instead" {
        assert!(
            w.error.contains("not empty") && w.error.contains("use"),
            "{}",
            w.error
        );
    } else if s.starts_with("no file in") || s.starts_with("every file in") {
        assert_eq!(w.snapshot(), w.before);
    } else if s == "the index is built" {
        w.service.index().unwrap();
    } else if s.starts_with("document ") {
        let docs = w.service.documents().unwrap();
        let d = docs.iter().find(|d| d.path == q[0]).unwrap();
        assert_eq!(d.kind, q[1]);
        if q.len() > 2 {
            assert_eq!(d.date.as_deref(), Some(q[2]));
        }
    } else if s.starts_with("there is a person") {
        let people = w.service.people().unwrap();
        let p = people.iter().find(|p| p.name == q[0]).unwrap();
        assert_eq!(p.known_as.as_deref(), Some(q[1]));
        assert_eq!(p.pronouns.as_deref(), Some(q[2]));
    } else if s.contains("has the alias") {
        assert!(
            w.service
                .people()
                .unwrap()
                .iter()
                .find(|p| p.name == q[0])
                .unwrap()
                .aliases
                .contains(&q[1].to_string())
        );
    } else if s.contains("has role") {
        let people = w.service.people().unwrap();
        let p = people.iter().find(|p| p.name == q[0]).unwrap();
        assert_eq!(p.role.as_deref(), Some(q[1]));
        assert_eq!(p.team.as_deref(), Some(q[2]));
    } else if s.starts_with("I search for") {
        w.results = w.service.search(q[0]).unwrap();
    } else if s.starts_with("the results include") {
        assert!(w.results.iter().any(|r| r.path == q[0]));
    } else if s.starts_with("each result shows") {
        assert!(!w.results.is_empty());
        for r in &w.results {
            assert!(
                r.snippet
                    .to_lowercase()
                    .contains(&format!("<mark>{}</mark>", q[0]))
            );
        }
    } else if s.starts_with("the file ") && s.ends_with("invalid UTF-8") {
        w.write(q[0], [0xff, 0xfe]);
    } else if s.starts_with("there is 1 warning") {
        let report = w.service.index().unwrap();
        assert_eq!(report.warnings.len(), 1);
        assert!(report.warnings[0].contains(q[0]));
    } else if s.starts_with("no document from") {
        assert!(w.service.search(q[0]).unwrap().is_empty());
    } else if s == "the index database is deleted" {
        fs::remove_file(w.temp.path().join("app-data/index.sqlite3")).unwrap();
    } else if s == "the app starts" {
        w.service = TopicsService::open(w.temp.path().join("app-data")).unwrap();
    } else if s == "the app is stopped" {
        w.watcher = None;
    } else if s == "the app is running and watching the notes root" {
        w.watcher = Some(w.service.watch().unwrap());
    } else if s.starts_with("another program appends") {
        use std::io::Write;
        let mut f = fs::OpenOptions::new()
            .append(true)
            .open(w.root.join(q[1]))
            .unwrap();
        writeln!(f, "{}", q[0]).unwrap();
    } else if s.starts_with("another program creates") {
        w.write(q[0], q[1]);
    } else if s.starts_with("another program renames") {
        fs::rename(w.root.join(q[0]), w.root.join(q[1])).unwrap();
    } else if s.starts_with("another program deletes") {
        fs::remove_file(w.root.join(q[0])).unwrap();
    } else if s.contains("the index contains") {
        let count: usize = s
            .split("contains ")
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if w.service.documents().unwrap().len() == count {
                break;
            }
            assert!(
                s.starts_with("within") && Instant::now() < deadline,
                "wrong document count"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    } else if s.contains("searching for") {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if w.service
                .search(q[0])
                .unwrap()
                .iter()
                .any(|r| r.path == q[1])
            {
                break;
            }
            assert!(
                (s.starts_with("within") || w.watcher.is_some()) && Instant::now() < deadline,
                "missing search result"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    } else if s.starts_with("no result points") {
        assert!(
            w.service
                .search("ledger")
                .unwrap()
                .iter()
                .all(|r| r.path != q[0])
        );
    } else {
        panic!("Unimplemented step: {s}");
    }
}
#[tokio::main]
async fn main() {
    Notes::cucumber()
        .fail_on_skipped()
        .run_and_exit("features/phase-0")
        .await;
}
