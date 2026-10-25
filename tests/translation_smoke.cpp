#include "core.h"
// CI-only translation link substitution. This executable is not a working translator.
void unload_translation_model() {}
TranslationResult translate_en_id(const std::wstring&, const std::filesystem::path&) {
  return {Failure::ModelMissing, {}};
}
std::wstring failure_label(Failure failure) {
  switch (failure) {
    case Failure::None: return L"Ready";
    case Failure::NoSelection: return L"No selection";
    case Failure::Unsupported: return L"Unsupported";
    case Failure::TooLong: return L"Selection too long";
    case Failure::CaptureFailed: return L"Capture failed";
    case Failure::ModelMissing: return L"Inference stub only";
    case Failure::ModelInvalid: return L"Model invalid";
    case Failure::TranslationFailed: return L"Translation failed";
    case Failure::ExcludedSource: return L"Capture denied by privacy policy";
  }
  return L"Unknown";
}