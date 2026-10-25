//! Tauri owns shell/tray/global shortcuts. Native tooltip owns its HWND.
//! No UIA, OCR, model inference, cloud endpoint or clipboard access exists in F0.
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};
use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::native_popup::{self, NativePopup};

struct ShellState {
    popup: Mutex<NativePopup>,
    shortcut_registered: AtomicBool,
    hotkey_count: AtomicU64,
    preview_count: AtomicU64,
}

#[derive(Serialize)]
struct ShellStatus {
    shortcut_registered: bool,
    hotkey_count: u64,
    preview_count: u64,
    last_paint_us: Option<u64>,
}

#[tauri::command]
fn get_status(state: tauri::State<'_, ShellState>) -> ShellStatus {
    ShellStatus {
        shortcut_registered: state.shortcut_registered.load(Ordering::Relaxed),
        hotkey_count: state.hotkey_count.load(Ordering::Relaxed),
        preview_count: state.preview_count.load(Ordering::Relaxed),
        last_paint_us: native_popup::last_paint_us(),
    }
}

fn open_settings(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("settings") {
        existing.show().map_err(|e| e.to_string())?;
        existing.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }
    WebviewWindowBuilder::new(
        app,
        "settings",
        WebviewUrl::App("index.html".into()),
    )
    .title("ClipLingo Settings")
    .inner_size(848.0, 624.0)
    .min_inner_size(620.0, 480.0)
    .build()
    .map(|_| ())
    .map_err(|e| e.to_string())
}

fn dispatch_preview(app: tauri::AppHandle) {
    let handle = app.clone();
    // HWND and all native GDI resources must be used from the Tauri UI thread.
    if let Err(err) = app.run_on_main_thread(move || {
        let state = handle.state::<ShellState>();
        match state.popup.lock() {
            Ok(popup) => {
                if let Err(err) = popup.show_preview() {
                    eprintln!("ClipLingo F0 native preview error: {err}");
                } else {
                    state.preview_count.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(_) => eprintln!("ClipLingo F0 native popup lock poisoned"),
        };
    }) {
        eprintln!("Unable to dispatch native preview: {err}");
    }
}

pub fn run() {
    let application = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = open_settings(app);
        }))
        .invoke_handler(tauri::generate_handler![get_status])
        .setup(|app| {
            let popup = NativePopup::create()
                .map_err(std::io::Error::other)?;
            app.manage(ShellState {
                popup: Mutex::new(popup),
                shortcut_registered: AtomicBool::new(false),
                hotkey_count: AtomicU64::new(0),
                preview_count: AtomicU64::new(0),
            });

            let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
            let preview = MenuItem::with_id(app, "preview", "Preview native tooltip (F0 only)", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings, &preview, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().ok_or("missing embedded Windows icon")?.clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        let handle = app.clone();
                        if let Err(err) = app.run_on_main_thread(move || {
                            if let Err(err) = open_settings(&handle) {
                                eprintln!("Unable to open Settings: {err}");
                            }
                        }) {
                            eprintln!("Settings dispatch failed: {err}");
                        }
                    }
                    "preview" => dispatch_preview(app.clone()),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            let hotkey = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyT);
            let preview_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyP);
            let expected_hotkey = hotkey.clone();
            let expected_preview = preview_shortcut.clone();
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, shortcut, event| {
                        if event.state() != ShortcutState::Pressed {
                            return;
                        }
                        if shortcut == &expected_hotkey {
                            // No capture provider yet: never fabricate a result popup.
                            let state = app.state::<ShellState>();
                            state.hotkey_count.fetch_add(1, Ordering::Relaxed);
                        } else if cfg!(debug_assertions) && shortcut == &expected_preview {
                            // Developer-only proof: press Ctrl+Alt+P while reading
                            // another app to test non-activation and positioning.
                            dispatch_preview(app.clone());
                        }
                    })
                    .build()
            )?;

            match app.global_shortcut().register(hotkey) {
                Ok(()) => {
                    app.state::<ShellState>().shortcut_registered.store(true, Ordering::Relaxed);
                }
                Err(err) => {
                    eprintln!("Ctrl+Alt+T already registered or unsupported: {err}");
                    // Tray and Settings remain reachable even on shortcut conflict.
                }
            }
            if cfg!(debug_assertions) {
                if let Err(err) = app.global_shortcut().register(preview_shortcut) {
                    eprintln!("Ctrl+Alt+P preview shortcut unavailable: {err}");
                }
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("unable to build ClipLingo F0 shell");

    application.run(|app, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            // Closing the lazily created Settings WebView must not kill the tray.
            api.prevent_exit();
        }
        tauri::RunEvent::Exit => {
            if let Some(state) = app.try_state::<ShellState>() {
                if let Ok(popup) = state.popup.lock() {
                    popup.destroy();
                }
            }
        }
        _ => {}
    });
}
