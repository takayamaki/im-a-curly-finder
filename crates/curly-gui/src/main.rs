#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use curly_finder::search::Found;
use curly_gui::exts::{EXT_GROUPS, ExtGroup, request_exts};
use curly_gui::root;
use curly_gui::session::Session;
use curly_gui::settings::Settings;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

/// How many rows the list shows at most.
const LIMIT: usize = 300;

/// Why the folder chosen at startup could not be opened, if it could not.
#[derive(Default)]
struct StartupError(Mutex<Option<String>>);

#[derive(Serialize)]
struct CurrentFolder {
    root: Option<PathBuf>,
    error: Option<String>,
    /// Folders under the root that could not be read
    skipped: Vec<String>,
}

fn settings_file(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

#[tauri::command]
fn current_folder(session: State<Arc<Session>>, startup: State<StartupError>) -> CurrentFolder {
    CurrentFolder {
        root: session.root(),
        error: startup.0.lock().unwrap().clone(),
        skipped: session.skipped(),
    }
}

#[tauri::command]
fn query(
    session: State<Arc<Session>>,
    expr: String,
    chosen: Vec<String>,
    exts: Vec<String>,
) -> Result<Found, String> {
    let chosen: Vec<&str> = chosen.iter().map(String::as_str).collect();
    let exts = request_exts(&exts);
    let exts: Vec<&str> = exts.iter().map(String::as_str).collect();
    session.search(&expr, &chosen, &exts, LIMIT)
}

#[derive(Serialize)]
struct ExtChoices {
    groups: &'static [ExtGroup],
    /// The checked extensions: those saved, or every one (any file) the first time
    checked: Vec<String>,
}

#[tauri::command]
fn ext_choices(app: AppHandle) -> ExtChoices {
    let saved = settings_file(&app)
        .ok()
        .and_then(|file| Settings::load(&file).exts);
    let every = || {
        EXT_GROUPS
            .iter()
            .flat_map(|g| g.exts)
            .map(|e| e.to_string())
            .collect()
    };
    ExtChoices {
        groups: EXT_GROUPS,
        checked: saved.unwrap_or_else(every),
    }
}

#[tauri::command]
fn save_exts(app: AppHandle, exts: Vec<String>) -> Result<(), String> {
    let file = settings_file(&app)?;
    let mut settings = Settings::load(&file);
    settings.exts = Some(exts);
    settings
        .save(&file)
        .map_err(|e| format!("Could not remember the extensions: {e}"))
}

#[tauri::command]
async fn rescan(session: State<'_, Arc<Session>>) -> Result<(), String> {
    let session = Arc::clone(&session);
    tauri::async_runtime::spawn_blocking(move || session.rescan())
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn open_folder(
    app: AppHandle,
    session: State<'_, Arc<Session>>,
    startup: State<'_, StartupError>,
    dir: PathBuf,
) -> Result<(), String> {
    let opened = Arc::clone(&session);
    let target = dir.clone();
    tauri::async_runtime::spawn_blocking(move || opened.open(&target))
        .await
        .map_err(|e| e.to_string())??;
    *startup.0.lock().unwrap() = None;
    let file = settings_file(&app)?;
    let mut settings = Settings::load(&file);
    settings.root = Some(dir);
    settings
        .save(&file)
        .map_err(|e| format!("Opened, but could not remember the folder: {e}"))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_drag::init())
        .manage(Arc::new(Session::new()))
        .manage(StartupError::default())
        .setup(|app| {
            let saved = settings_file(app.handle())
                .ok()
                .and_then(|file| Settings::load(&file).root);
            let args = std::env::args_os().nth(1);
            if let Some(dir) = root::resolve(args, std::env::var_os("CURLY_ROOT"), saved)
                && let Err(e) = app.state::<Arc<Session>>().open(&dir)
            {
                *app.state::<StartupError>().0.lock().unwrap() = Some(e);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            current_folder,
            query,
            rescan,
            open_folder,
            ext_choices,
            save_exts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
