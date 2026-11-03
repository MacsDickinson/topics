use topics_store::{TopicsService, WorkspaceService, application_data_dir};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut service = TopicsService::open(application_data_dir()?)?;
    let mut args = std::env::args().skip(1);
    let command = args
        .next()
        .ok_or("Usage: topics <init PATH|use PATH|index|search QUERY|people>")?;
    if command == "workspace" {
        return workspace(args);
    }
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

/// Each workspace command accepts the same single request object used by IPC.
fn workspace(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
    let command=args.next().ok_or("Usage: topics workspace <root-get|root-set|sources|source|preview|manual|proposals|decide|topics|topic|history|undo|sensitivity> REQUEST_JSON")?;
    let input = args.next().unwrap_or_else(|| "{}".into());
    if args.next().is_some() {
        return Err("Expected one JSON request object".into());
    }
    let mut service = WorkspaceService::open(application_data_dir()?)?;
    macro_rules! call {
        ($method:ident) => {
            serde_json::to_value(service.$method(serde_json::from_str(&input)?)?)?
        };
    }
    let output = match command.as_str() {
        "root-get" => call!(root_get),
        "root-set" => call!(root_set),
        "sources" => call!(sources_list),
        "source" => call!(source_get),
        "source-repair" => call!(source_identity_repair),
        "statement" => call!(statement_get),
        "preview" => call!(scope_preview),
        "manual" => call!(manual_propose),
        "proposals" => call!(proposals_list),
        "decide" => call!(proposal_decide),
        "topics" => call!(topics_list),
        "topic" => call!(topic_get),
        "history" => call!(history_list),
        "undo" => call!(undo),
        "sensitivity" => call!(sensitivity_set),
        _ => return Err("Unknown workspace command".into()),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
