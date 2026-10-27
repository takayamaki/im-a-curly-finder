#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use curly_finder::search::Found;
use curly_gui::exts::{EXT_GROUPS, other_exts, request_exts};
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
    unchecked: Vec<String>,
) -> Result<Found, String> {
    let chosen: Vec<&str> = chosen.iter().map(String::as_str).collect();
    let exts = request_exts(&unchecked, &other_exts(&session.exts()));
    let exts: Vec<&str> = exts.iter().map(String::as_str).collect();
    session.search(&expr, &chosen, &exts, LIMIT)
}

/// A group of the extension drop-down as the page shows it.
#[derive(Serialize)]
struct ExtChoiceGroup {
    name: &'static str,
    exts: Vec<String>,
}

#[derive(Serialize)]
struct ExtChoices {
    /// The groups, then Others (the extensions of the open folder in no group) if there are any
    groups: Vec<ExtChoiceGroup>,
    /// The unchecked extensions, as saved
    unchecked: Vec<String>,
}

#[tauri::command]
fn ext_choices(app: AppHandle, session: State<Arc<Session>>) -> ExtChoices {
    let mut groups: Vec<ExtChoiceGroup> = EXT_GROUPS
        .iter()
        .map(|g| ExtChoiceGroup {
            name: g.name,
            exts: g.exts.iter().map(|e| e.to_string()).collect(),
        })
        .collect();
    let others = other_exts(&session.exts());
    if !others.is_empty() {
        groups.push(ExtChoiceGroup {
            name: "Others",
            exts: others,
        });
    }
    let unchecked = settings_file(&app)
        .map(|file| Settings::load(&file).unchecked_exts)
        .unwrap_or_default();
    ExtChoices { groups, unchecked }
}

#[tauri::command]
fn save_exts(app: AppHandle, unchecked: Vec<String>) -> Result<(), String> {
    let file = settings_file(&app)?;
    let mut settings = Settings::load(&file);
    settings.unchecked_exts = unchecked;
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

// ---- the context menu of a row ----
// Only the files of the list: the page could send any path.

fn listed(session: &Session, path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if session.lists(&path) {
        Ok(path)
    } else {
        Err(format!("Not a file of the list: {}", path.display()))
    }
}

/// Opens the file with the app it is associated with.
#[tauri::command]
fn open_file(session: State<Arc<Session>>, path: String) -> Result<(), String> {
    let path = listed(&session, &path)?;
    tauri_plugin_opener::open_path(&path, None::<&str>).map_err(|e| e.to_string())
}

/// Shows the Windows "Open with" dialog for the file. Not async, so it runs on the main thread,
/// modal to the window.
#[tauri::command]
fn open_file_with(
    window: tauri::WebviewWindow,
    session: State<Arc<Session>>,
    path: String,
) -> Result<(), String> {
    let path = listed(&session, &path)?;
    open_with_dialog(&window, &path)
}

#[cfg(windows)]
fn open_with_dialog(window: &tauri::WebviewWindow, path: &std::path::Path) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::{
        OAIF_ALLOW_REGISTRATION, OAIF_EXEC, OPENASINFO, SHOpenWithDialog,
    };
    use windows::core::{HSTRING, PCWSTR};

    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    let file = HSTRING::from(path.as_os_str());
    let info = OPENASINFO {
        pcszFile: PCWSTR(file.as_ptr()),
        pcszClass: PCWSTR::null(),
        // OAIF_EXEC: open the file with the app chosen. OAIF_ALLOW_REGISTRATION: the "Always use
        // this app" box, left to the user.
        oaifInFlags: OAIF_ALLOW_REGISTRATION | OAIF_EXEC,
    };
    // Tauri's HWND may come from another version of the windows crate.
    unsafe { SHOpenWithDialog(Some(HWND(hwnd.0)), &info) }.or_else(|e| {
        // Cancelling the dialog is not an error.
        if e.code() == windows::Win32::Foundation::ERROR_CANCELLED.to_hresult() {
            Ok(())
        } else {
            Err(e.to_string())
        }
    })
}

#[cfg(not(windows))]
fn open_with_dialog(_window: &tauri::WebviewWindow, _path: &std::path::Path) -> Result<(), String> {
    Err("Open with is only on Windows".to_string())
}

/// Opens the folder of the file in Explorer, with the file selected.
#[tauri::command]
fn show_in_folder(session: State<Arc<Session>>, path: String) -> Result<(), String> {
    let path = listed(&session, &path)?;
    tauri_plugin_opener::reveal_item_in_dir(&path).map_err(|e| e.to_string())
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
            save_exts,
            open_file,
            open_file_with,
            show_in_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
