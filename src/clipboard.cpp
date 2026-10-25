#include "core.h"
#include "clipboard_text.h"
#include <windows.h>

namespace {
struct ClipboardGuard {
  bool open{false};
  ClipboardGuard() : open(OpenClipboard(nullptr) != 0) {}
  ~ClipboardGuard() { if (open) CloseClipboard(); }
  ClipboardGuard(const ClipboardGuard&) = delete;
  ClipboardGuard& operator=(const ClipboardGuard&) = delete;
};
struct GlobalLockGuard {
  HGLOBAL memory{};
  void* data{};
  explicit GlobalLockGuard(HANDLE handle)
    : memory(static_cast<HGLOBAL>(handle)), data(GlobalLock(memory)) {}
  ~GlobalLockGuard() { if (data) GlobalUnlock(memory); }
  GlobalLockGuard(const GlobalLockGuard&) = delete;
  GlobalLockGuard& operator=(const GlobalLockGuard&) = delete;
};
}

CaptureResult capture_clipboard_text(DWORD expected_sequence) {
  // Explicit, opt-in, one-shot read only. Clipboard data may have been copied
  // from ANY application: it has no reliable provenance.
  if (!expected_sequence || GetClipboardSequenceNumber() != expected_sequence)
    return {Failure::ClipboardChanged, {}};
  ClipboardGuard clipboard;
  if (!clipboard.open) return {Failure::ClipboardUnavailable, {}};
  if (GetClipboardSequenceNumber() != expected_sequence)
    return {Failure::ClipboardChanged, {}};
  if (!IsClipboardFormatAvailable(CF_UNICODETEXT))
    return {Failure::NoSelection, {}};
  HANDLE handle = GetClipboardData(CF_UNICODETEXT);
  if (!handle) return {Failure::ClipboardUnavailable, {}};
  const SIZE_T bytes = GlobalSize(static_cast<HGLOBAL>(handle));
  if (!bytes || (bytes % sizeof(wchar_t)) != 0)
    return {Failure::CaptureFailed, {}};
  if (bytes > 128u * 1024u) return {Failure::TooLong, {}};
  GlobalLockGuard locked(handle);
  if (!locked.data) return {Failure::ClipboardUnavailable, {}};
  const auto text = clipboard_text::inspect(
      static_cast<const wchar_t*>(locked.data), bytes / sizeof(wchar_t));
  switch (text.state) {
    case clipboard_text::State::Valid: return {Failure::None, text.text};
    case clipboard_text::State::Empty: return {Failure::NoSelection, {}};
    case clipboard_text::State::TooLong: return {Failure::TooLong, {}};
    case clipboard_text::State::Invalid: return {Failure::CaptureFailed, {}};
  }
  return {Failure::CaptureFailed, {}};
}
