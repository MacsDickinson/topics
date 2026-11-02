#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::Manager;
use topics_store::{SearchResult, TopicsService, WatchHandle, application_data_dir};

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
                return Err("No notes folder configured. Use topics use <path> first.".to_string());
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

fn main() {
    let builder = tauri::Builder::default();
    #[cfg(all(debug_assertions, feature = "native-test"))]
    let builder = builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init());
    builder
        .setup(|app| {
            // Keep resolver failures in managed state so the window remains usable.
            // A search presents the original error; never fall back to personal data.
            let app_data = application_data_dir().map_err(|error| error.to_string());
            app.manage(SearchState {
                app_data,
                session: Arc::new(Mutex::new(None)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![search_notes])
        .run(tauri::generate_context!())
        .expect("failed to run Topics desktop");
}
