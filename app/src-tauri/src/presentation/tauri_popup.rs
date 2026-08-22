use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};

use crate::core::{PopupPort, PopupViewModel};

pub struct TauriPopupPort {
    app: AppHandle,
}

impl TauriPopupPort {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn window(&self) -> Option<tauri::WebviewWindow> {
        self.app.get_webview_window("popup")
    }

    fn emit_state(&self, state: PopupViewModel) {
        if let Some(window) = self.window() {
            if let Err(error) = window.emit("popup-state", state) {
                eprintln!("event=popup_emit status=error error={error}");
            }
        } else {
            eprintln!("event=popup_lookup status=error reason=missing_window");
        }
    }
}

impl PopupPort for TauriPopupPort {
    fn show(&self, state: PopupViewModel) {
        self.emit_state(state);
        if let Some(window) = self.window() {
            if let Err(error) = window.show() {
                eprintln!("event=popup_show status=error error={error}");
            }
        }
    }

    fn update(&self, state: PopupViewModel) {
        self.emit_state(state);
    }

    fn move_to(&self, x: f64, y: f64) {
        if let Some(window) = self.window() {
            let position = PhysicalPosition::new(x.round() as i32, y.round() as i32);
            if let Err(error) = window.set_position(position) {
                eprintln!("event=popup_move status=error error={error}");
            }
        }
    }

    fn hide(&self) {
        if let Some(window) = self.window() {
            if let Err(error) = window.hide() {
                eprintln!("event=popup_hide status=error error={error}");
            }
        }
    }
}
