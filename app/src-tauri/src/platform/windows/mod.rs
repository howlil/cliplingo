mod clipboard;
mod cursor;
mod hotkey;
mod provider;
mod selection;

pub use clipboard::ClipboardSelectionProvider;
pub use cursor::{cursor_anchor, work_area_for_rect, CursorContext};
pub use hotkey::TRANSLATE_SHORTCUT;
pub use provider::WindowsSelectionProvider;
pub use selection::UiAutomationSelectionProvider;
