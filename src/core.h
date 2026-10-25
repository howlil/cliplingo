#pragma once
#include <windows.h>
#include <cstdint>
#include <string>
#include <filesystem>
#include <string_view>

// Selection always takes priority; these modes are only fallback policy.
enum class CaptureScope : std::uint32_t {
  SelectionOnly = 0, Word = 1, Sentence = 2, Paragraph = 3
};
enum class Failure { None, NoSelection, Unsupported, TooLong, CaptureFailed, ModelMissing, ModelInvalid, TranslationFailed, OcrUnavailable, ExcludedSource, ClipboardUnavailable, ClipboardChanged };
struct CaptureResult {
  Failure failure{Failure::None};
  std::wstring text;
  explicit operator bool() const { return failure == Failure::None && !text.empty(); }
};
struct TranslationResult {
  Failure failure{Failure::None};
  std::wstring text;
  explicit operator bool() const { return failure == Failure::None && !text.empty(); }
};
enum class RequestKind : std::uint8_t { Text, ManualOcr, ExplicitClipboard };
struct Request {
  RequestKind kind{RequestKind::Text};
  RECT region{}; // physical virtual-screen coordinates; manual OCR only
  DWORD clipboard_sequence{0}; // explicit clipboard only
  std::uint64_t generation{0};
  HWND foreground{nullptr};
  POINT cursor{};  // physical virtual-screen pixels captured at hotkey time
};
struct Preferences {
  int max_width_dip{420};
  int max_height_dip{300};
  int theme{0}; // 0 light, 1 dark
  bool clipboard_enabled{false}; // opt-in; no automatic fallback
  std::wstring excluded_app_names; // validated semicolon-separated exe basenames
  bool hover_enabled{true};    // invoked only by the translation hotkey
  int hover_scope{0};        // 0 word, 1 sentence, 2 paragraph
  int popup_style{0}; // 0 solid, 1 tinted (glass deferred)
  std::filesystem::path model_path;
};

struct ModelInstallResult {
  bool success{false};
  std::filesystem::path path;
  std::wstring message;
};
bool verify_model_pack(const std::filesystem::path& folder);
bool capture_source_allowed(HWND foreground, std::wstring_view policy);
ModelInstallResult install_model_pack(const std::filesystem::path& source);
bool ocr_available();
CaptureResult capture_ocr_region(RECT selected);
CaptureResult capture_ocr_isolated(RECT selected);
CaptureResult capture_clipboard_text(DWORD expected_sequence);
CaptureResult capture_clipboard_isolated(DWORD expected_sequence);
CaptureResult capture_selected_text(HWND expected_foreground);
CaptureResult capture_hover_text(HWND expected_foreground, POINT cursor, CaptureScope scope);
CaptureResult capture_with_hover(HWND expected_foreground, POINT cursor, CaptureScope scope);
CaptureResult capture_isolated(HWND expected_foreground, POINT cursor, CaptureScope scope);
bool maybe_run_capture_helper(int& exit_code);
void unload_translation_model(); // call only on inference worker thread
TranslationResult translate_en_id(const std::wstring& selected, const std::filesystem::path& model_dir);
std::wstring failure_label(Failure failure);