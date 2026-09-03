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
use tauri::{Manager, State};
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
    application::install_model_pack(app).await
}

#[tauri::command]
async fn remove_model_pack(app: tauri::AppHandle) -> Result<ModelPackStatus, String> {
    application::remove_model_pack(app).await
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
            remove_model_pack
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ClipLingo");
}
