//! Render the synthetic app-server child for the native desktop test harness.
//! Usage: cargo run -p topics-setup --example render_fixture -- /absolute/empty-temp-dir graph_claim
//! No notes, provider, token, HOME or Codex configuration is read or changed.
#[path = "../tests/support/render.rs"]
mod fixture;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let directory = args
        .next()
        .ok_or("Usage: render_fixture /absolute/empty-temp-dir [graph_claim]")?;
    let mode = args.next().unwrap_or_else(|| "graph_claim".into());
    if args.next().is_some() {
        return Err("Usage: render_fixture /absolute/empty-temp-dir [graph_claim]".into());
    }
    let mode = mode
        .to_str()
        .ok_or("Synthetic fixture mode must be UTF-8")?;
    let paths = fixture::render(std::path::Path::new(&directory), mode)?;
    println!("{}", serde_json::to_string(&paths)?);
    Ok(())
}
