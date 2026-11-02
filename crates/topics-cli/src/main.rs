use std::path::PathBuf;
use topics_store::TopicsService;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let app_data = match std::env::var_os("TOPICS_APP_DATA") {
        Some(path) => PathBuf::from(path),
        None => directories::ProjectDirs::from("", "", "Topics")
            .ok_or("Cannot find Application Support; set TOPICS_APP_DATA")?
            .data_local_dir()
            .to_path_buf(),
    };
    let mut service = TopicsService::open(app_data)?;
    let mut args = std::env::args().skip(1);
    let command = args
        .next()
        .ok_or("Usage: topics <init PATH|use PATH|index|search QUERY|people>")?;
    let output = match command.as_str() {
        "init" | "use" => {
            let path = args.next().ok_or("A notes path is required")?;
            if args.next().is_some() {
                return Err("Unexpected arguments".into());
            }
            serde_json::to_value(if command == "init" {
                service.init_notes_root(path)?
            } else {
                service.use_notes_root(path)?
            })?
        }
        "index" => {
            if args.next().is_some() {
                return Err("Unexpected arguments".into());
            }
            serde_json::to_value(service.index()?)?
        }
        "people" => {
            if args.next().is_some() {
                return Err("Unexpected arguments".into());
            }
            serde_json::to_value(service.people()?)?
        }
        "search" => {
            let query = args.collect::<Vec<_>>().join(" ");
            if query.trim().is_empty() {
                return Err("A search query is required".into());
            }
            serde_json::to_value(service.search(&query)?)?
        }
        _ => return Err("Unknown command; use init, use, index, search or people".into()),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
