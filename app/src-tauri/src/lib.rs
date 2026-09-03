pub mod application;
pub mod core;
pub mod platform;
pub mod presentation;

use std::sync::Arc;
use std::time::Instant;

use application::{InteractionCoordinator, ModelPackStatus, WorkerTranslator};
use core::PopupViewModel;
use platform::windows::{cursor_anchor, WindowsSelectionProvider, TRANSLATE_SHORTCUT};
use presentation::TauriPopupPort;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[tauri::command]
fn get_popup_state(coordinator: State<'_, Arc<InteractionCoordinator>>) -> PopupViewModel {
    coordinator.snapshot()
}

#[tauri::command]
fn dismiss_popup(coordinator: State<'_, Arc<InteractionCoordinator>>) {
    coordinator.dismiss();
}

#[tauri::command]
fn get_model_pack_status(app: tauri::AppHandle) -> Result<ModelPackStatus, String> {
    application::model_pack_status(&app)
}

#[tauri::command]
async fn install_model_pack(app: tauri::AppHandle) -> Result<ModelPackStatus, String> {
    let status = application::install_model_pack(app.clone()).await?;
    if let Err(error) = app.emit("model-pack-state", status.clone()) {
        eprintln!("event=model_pack_state status=emit_error error={error}");
    }
    Ok(status)
}

#[tauri::command]
async fn remove_model_pack(app: tauri::AppHandle) -> Result<ModelPackStatus, String> {
    let status = application::remove_model_pack(app.clone()).await?;
    if let Err(error) = app.emit("model-pack-state", status.clone()) {
        eprintln!("event=model_pack_state status=emit_error error={error}");
    }
    Ok(status)
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

fn show_settings(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let started_at = Instant::now();
                    let coordinator = app.state::<Arc<InteractionCoordinator>>();
                    if coordinator.is_visible() {
                        coordinator.dismiss();
                        return;
                    }

                    match cursor_anchor() {
                        Ok(context) => {
                            coordinator.trigger_at(&context.anchor, &context.work_area, started_at);
                        }
                        Err(error) => {
                            eprintln!("event=cursor_anchor status=error error={error:?}");
                            coordinator.trigger(started_at);
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_popup_state,
            dismiss_popup,
            get_model_pack_status,
            install_model_pack,
            remove_model_pack,
            quit_app
        ])
        .setup(|app| {
            let popup = Arc::new(TauriPopupPort::new(app.handle().clone()));
            let model_pack = application::model_pack_directory(app.handle())
                .map_err(std::io::Error::other)?;
            let coordinator = InteractionCoordinator::start(
                Box::new(WindowsSelectionProvider::new()),
                Box::new(WorkerTranslator::new_default(model_pack)),
                popup,
            );
            app.manage(Arc::clone(&coordinator));
            app.global_shortcut()
                .register(TRANSLATE_SHORTCUT)
                .map_err(|error| {
                    eprintln!(
                        "event=hotkey_register status=error shortcut={TRANSLATE_SHORTCUT} error={error}"
                    );
                    error
                })?;

            if let Some(settings) = app.get_webview_window("settings") {
                let settings_to_hide = settings.clone();
                settings.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = settings_to_hide.hide();
                    }
                });
            }

            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit ClipLingo", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            let mut tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => show_settings(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_settings(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ClipLingo");
}
