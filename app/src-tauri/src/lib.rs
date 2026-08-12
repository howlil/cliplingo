pub mod application;
pub mod core;
pub mod platform;
pub mod presentation;

use std::sync::Arc;

use application::{FakeTranslator, InteractionCoordinator};
use core::{CaptureError, PopupViewModel, Selection, SelectionProvider};
use platform::windows::cursor_anchor;
use presentation::TauriPopupPort;
use tauri::{Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

struct NoSelectionProvider;

impl SelectionProvider for NoSelectionProvider {
    fn capture(&mut self) -> Result<Selection, CaptureError> {
        Err(CaptureError::Unsupported)
    }
}

#[tauri::command]
fn get_popup_state(coordinator: State<'_, Arc<InteractionCoordinator>>) -> PopupViewModel {
    coordinator.snapshot()
}

#[tauri::command]
fn dismiss_popup(coordinator: State<'_, Arc<InteractionCoordinator>>) {
    coordinator.dismiss();
}

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let coordinator = app.state::<Arc<InteractionCoordinator>>();
                    if coordinator.is_visible() {
                        coordinator.dismiss();
                        return;
                    }

                    match cursor_anchor() {
                        Ok(context) => {
                            coordinator.trigger_at(&context.anchor, &context.work_area);
                        }
                        Err(error) => {
                            eprintln!("event=cursor_anchor status=error error={error:?}");
                            coordinator.trigger();
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![get_popup_state, dismiss_popup])
        .setup(|app| {
            let popup = Arc::new(TauriPopupPort::new(app.handle().clone()));
            let coordinator = InteractionCoordinator::start(
                Box::new(NoSelectionProvider),
                Box::new(FakeTranslator),
                popup,
            );
            app.manage(Arc::clone(&coordinator));
            app.global_shortcut().register("Ctrl+Alt+T")?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ClipLingo");
}
