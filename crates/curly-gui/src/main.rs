#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use curly_finder::index::{Index, SharedIndex};
use curly_gui::root;
use curly_gui::search::{Found, search};
use tauri::State;

/// How many rows the list shows at most.
const LIMIT: usize = 300;

/// The index, or why there is none.
struct App(Result<Arc<SharedIndex>, String>);

impl App {
    fn index(&self) -> Result<&Arc<SharedIndex>, String> {
        self.0.as_ref().map_err(Clone::clone)
    }
}

#[tauri::command]
fn query(app: State<App>, expr: String) -> Result<Found, String> {
    search(&app.index()?.snapshot(), &expr, LIMIT)
}

#[tauri::command]
async fn rescan(app: State<'_, App>) -> Result<(), String> {
    let shared = Arc::clone(app.index()?);
    tauri::async_runtime::spawn_blocking(move || shared.rescan())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

fn main() {
    let app = match root::resolve(
        std::env::args_os().nth(1),
        std::env::var_os("CURLY_ROOT"),
        None,
    ) {
        None => Err("No folder to search. Pass it as the first argument or set CURLY_ROOT.".into()),
        Some(dir) => Index::scan(&dir)
            .map(|index| Arc::new(SharedIndex::new(index)))
            .map_err(|e| format!("{}: {e}", dir.display())),
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_drag::init())
        .manage(App(app))
        .invoke_handler(tauri::generate_handler![query, rescan])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
