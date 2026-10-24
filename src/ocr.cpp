#include "core.h"
#include "region_geometry.h"
#include <algorithm>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <memory>
#include <string>
#include <dwmapi.h>
#if defined(CLIPLINGO_WITH_TESSERACT)
#include <tesseract/baseapi.h>
#endif

namespace {
std::filesystem::path data_folder() {
  wchar_t module[32768]{};
  const DWORD count = GetModuleFileNameW(nullptr, module, 32768);
  if (!count || count >= 32768) return {};
  return std::filesystem::path(module).parent_path() / L"tessdata";
}
struct ScreenDc {
  HDC dc{GetDC(nullptr)};
  ~ScreenDc() { if (dc) ReleaseDC(nullptr, dc); }
};
struct MemoryDc {
  HDC dc{nullptr};
  explicit MemoryDc(HDC compatible) : dc(CreateCompatibleDC(compatible)) {}
  ~MemoryDc() { if (dc) DeleteDC(dc); }
};
struct Bitmap {
  HBITMAP handle{nullptr};
  ~Bitmap() { if (handle) DeleteObject(handle); }
};
} // namespace

bool ocr_available() {
#if defined(CLIPLINGO_WITH_TESSERACT)
  const auto path = data_folder();
  std::error_code ec;
  return !path.empty() &&
         std::filesystem::is_regular_file(path / L"eng.traineddata", ec);
#else
  return false;
#endif
}

// Called only from the bounded worker, after the selection overlay is hidden.
// A fresh DIB section captures exactly the selected pixels; no screen image is
// stored on disk or kept in the tray process while idle.
CaptureResult capture_ocr_region(RECT selected) {
#if !defined(CLIPLINGO_WITH_TESSERACT)
  (void)selected;
  return {Failure::OcrUnavailable, {}};
#else
  if (!ocr_available()) return {Failure::OcrUnavailable, {}};
  const region_geometry::Rect region{selected.left, selected.top,
                                     selected.right, selected.bottom};
  if (!region_geometry::valid(region)) return {Failure::CaptureFailed, {}};
  const int width = selected.right - selected.left;
  const int height = selected.bottom - selected.top;
  const auto bytes = static_cast<std::size_t>(width) * height * 4u;

  ScreenDc screen;
  if (!screen.dc) return {Failure::CaptureFailed, {}};
  MemoryDc memory(screen.dc);
  if (!memory.dc) return {Failure::CaptureFailed, {}};
  BITMAPINFO info{};
  info.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
  info.bmiHeader.biWidth = width;
  info.bmiHeader.biHeight = -height; // DIB top-down, RGB pixels
  info.bmiHeader.biPlanes = 1;
  info.bmiHeader.biBitCount = 32;
  info.bmiHeader.biCompression = BI_RGB;
  void* buffer = nullptr;
  Bitmap bitmap;
  bitmap.handle = CreateDIBSection(screen.dc, &info, DIB_RGB_COLORS, &buffer, nullptr, 0);
  if (!bitmap.handle || !buffer) return {Failure::CaptureFailed, {}};
  HGDIOBJ old = SelectObject(memory.dc, bitmap.handle);
  if (!old || old == HGDI_ERROR) return {Failure::CaptureFailed, {}};
  // Flush removal of the transient selection overlay before capturing.
  DwmFlush();
  const BOOL ok = BitBlt(memory.dc, 0, 0, width, height, screen.dc,
                         selected.left, selected.top, SRCCOPY);
  SelectObject(memory.dc, old);
  if (!ok) return {Failure::CaptureFailed, {}};

  // GDI 32-bit BI_RGB is byte-ordered BGRA; Tesseract SetImage takes RGBA.
  // Reuse the DIB buffer in place. Avoid allocating another potentially
  // 48 MB temporary buffer for the same ROI.
  auto* rgba = static_cast<unsigned char*>(buffer);
  for (std::size_t i = 0; i < bytes; i += 4) {
    std::swap(rgba[i], rgba[i+2]);
    rgba[i+3] = 255;
  }
  tesseract::TessBaseAPI engine;
  const auto u8 = data_folder().u8string();
  const std::string tessdata(u8.begin(), u8.end());
  if (engine.Init(tessdata.c_str(), "eng", tesseract::OEM_LSTM_ONLY) != 0)
    return {Failure::OcrUnavailable, {}};
  engine.SetPageSegMode(tesseract::PSM_AUTO);
  engine.SetImage(rgba, width, height, 4, width*4);
  std::unique_ptr<char[]> raw(engine.GetUTF8Text());
  if (!raw) return {Failure::CaptureFailed, {}};
  // Explicit cap: no huge OCR output may reach CTranslate2.
  const auto len = strnlen_s(raw.get(), 32769);
  if (!len || len >= 32769) return {Failure::TooLong, {}};
  const int count = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS,
                                        raw.get(), static_cast<int>(len), nullptr, 0);
  if (count <= 0 || count > 8192) return {Failure::TooLong, {}};
  std::wstring text(static_cast<std::size_t>(count), L'\0');
  if (MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, raw.get(),
                          static_cast<int>(len), text.data(), count) != count)
    return {Failure::CaptureFailed, {}};
  const auto first = text.find_first_not_of(L" \r\n\t\v\f");
  if (first == std::wstring::npos) return {Failure::NoSelection, {}};
  const auto last = text.find_last_not_of(L" \r\n\t\v\f");
  return {Failure::None, text.substr(first, last - first + 1)};
#endif
}
