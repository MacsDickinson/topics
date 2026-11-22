#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::Manager;
mod runtime_path;
mod workspace;
use topics_store::{SearchResult, TopicsService, WatchHandle, application_data_dir};
use workspace::*;

struct StoreSession {
    service: TopicsService,
    // Dropping the handle stops the watcher; retain it as long as the store.
    _watcher: WatchHandle,
}

#[derive(Clone)]
struct SearchState {
    app_data: Result<PathBuf, String>,
    session: Arc<Mutex<Option<StoreSession>>>,
}

#[tauri::command]
async fn search_notes(
    query: String,
    state: tauri::State<'_, SearchState>,
) -> Result<Vec<SearchResult>, String> {
    let state = state.inner().clone();
    // Opening, scanning and SQLite queries never block the webview event loop.
    tauri::async_runtime::spawn_blocking(move || {
        let app_data = state.app_data.as_ref().map_err(Clone::clone)?;
        let mut session = state
            .session
            .lock()
            .map_err(|_| "Search service unavailable".to_string())?;
        if session.is_none() {
            let service = TopicsService::open(app_data).map_err(|error| error.to_string())?;
            if service
                .notes_root()
                .map_err(|error| error.to_string())?
                .is_none()
            {
                return Err(
                    "No notes folder configured. Choose a notes folder in the app first."
                        .to_string(),
                );
            }
            let watcher = service.watch().map_err(|error| error.to_string())?;
            *session = Some(StoreSession {
                service,
                _watcher: watcher,
            });
        }
        session
            .as_ref()
            .expect("store initialized")
            .service
            .search(&query)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

fn proxy_invocation(mut arguments: impl Iterator<Item = std::ffi::OsString>) -> bool {
    arguments.next();
    arguments.next().as_deref() == Some(std::ffi::OsStr::new("--topics-mcp-proxy"))
        && arguments.next().is_none()
}

fn main() {
    // Minimal MCP proxy dispatch happens before GUI, state or root discovery.
    if proxy_invocation(std::env::args_os()) {
        std::process::exit(topics_runtime::run_stdio_proxy());
    }
    let builder = tauri::Builder::default();
    #[cfg(all(debug_assertions, feature = "native-test"))]
    let builder = builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init());
    builder
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Keep resolver failures in managed state so the window remains usable.
            // A search presents the original error; never fall back to personal data.
            let app_data = application_data_dir().map_err(|error| error.to_string());
            app.manage(WorkspaceState::new(app_data.clone()));
            app.manage(SearchState {
                app_data,
                session: Arc::new(Mutex::new(None)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search_notes,
            workspace_root_get,
            workspace_root_set,
            workspace_choose_root,
            workspace_scope_preview,
            workspace_sensitivity_set,
            workspace_analysis_start,
            workspace_run_cancel,
            workspace_proposals_list,
            workspace_proposal_decide,
            workspace_topics_list,
            workspace_topic_get,
            workspace_statement_get,
            workspace_user_statement,
            workspace_chat_start,
            workspace_history_list,
            workspace_undo,
            workspace_runtime_status,
            workspace_manual_propose,
            workspace_sources_list,
            workspace_source_get,
            pilot_briefing,
            pilot_snapshot_create,
            pilot_feedback_snapshot_create,
            pilot_snapshot_list,
            pilot_snapshot_get,
            pilot_feedback_list,
            pilot_feedback_save,
            pilot_week_get,
            pilot_week_save,
            setup_inventory,
            setup_manifest_preview,
            setup_job_get,
            setup_job_create,
            setup_job_start,
            setup_job_watch,
            setup_job_control,
            setup_jobs_list,
            setup_draft_edit,
            setup_draft_disposition,
            setup_save_preview,
            setup_job_reconsent,
            setup_connection_confirm_preview,
            setup_context_add,
            setup_confirm_preview,
            setup_correction_preview,
            setup_decide,
            setup_topic_review
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Topics desktop");
}

#[cfg(test)]
mod entrypoint_tests {
    use super::*;
    #[test]
    fn proxy_dispatch_requires_exact_supported_position_and_argument_count() {
        let args = |values: &[&str]| {
            values
                .iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>()
                .into_iter()
        };
        assert!(proxy_invocation(args(&["topics", "--topics-mcp-proxy"])));
        assert!(!proxy_invocation(args(&["topics"])));
        assert!(!proxy_invocation(args(&[
            "topics",
            "--open",
            "--topics-mcp-proxy"
        ])));
        assert!(!proxy_invocation(args(&[
            "topics",
            "--topics-mcp-proxy",
            "extra"
        ])));
    }
}
