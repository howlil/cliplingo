#pragma once
#include <windows.h>
#include <cstdint>
#include <string>
#include <filesystem>

enum class Failure { None, NoSelection, Unsupported, TooLong, CaptureFailed, ModelMissing, ModelInvalid, TranslationFailed };
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
struct Request {
  std::uint64_t generation{0};
  HWND foreground{nullptr};
  POINT cursor{};  // physical virtual-screen pixels captured at hotkey time
};
struct Preferences {
  int max_width_dip{420};
  int max_height_dip{300};
  int theme{0}; // 0 light, 1 dark
  int popup_style{0}; // 0 solid, 1 tinted (glass deferred)
  std::filesystem::path model_path;
};

struct ModelInstallResult {
  bool success{false};
  std::filesystem::path path;
  std::wstring message;
};
bool verify_model_pack(const std::filesystem::path& folder);
ModelInstallResult install_model_pack(const std::filesystem::path& source);
CaptureResult capture_selected_text(HWND expected_foreground);
CaptureResult capture_isolated(HWND expected_foreground);
bool maybe_run_capture_helper(int& exit_code);
void unload_translation_model(); // call only on inference worker thread
TranslationResult translate_en_id(const std::wstring& selected, const std::filesystem::path& model_dir);
std::wstring failure_label(Failure failure);