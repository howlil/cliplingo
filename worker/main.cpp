#include <windows.h>

#include <ctranslate2/translator.h>
#include <sentencepiece_processor.h>

#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <memory>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

namespace {

constexpr wchar_t kPipeName[] = LR"(\\.\pipe\cliplingo-worker-v1)";
constexpr std::array<std::uint8_t, 4> kMagic = {'C', 'L', 'N', 'G'};
constexpr std::uint8_t kProtocolVersion = 1;
constexpr std::uint8_t kTranslateRequest = 0x01;
constexpr std::uint8_t kTranslateResponse = 0x02;
constexpr std::uint8_t kErrorResponse = 0x03;
constexpr std::uint8_t kUnsupportedRequest = 0x02;
constexpr std::uint8_t kTranslationFailed = 0x03;
constexpr std::size_t kHeaderLen = 18;
constexpr std::uint32_t kMaxPayloadLen = 1024U * 1024U;
constexpr std::array<std::uint8_t, 7> kFakePrefix = {'[', 'F', 'A', 'K', 'E', ']', ' '};

struct Header {
  std::uint8_t message_type;
  std::uint64_t request_id;
  std::uint32_t payload_len;
};

enum class ReadStatus {
  Ok,
  Closed,
  Error,
};

class TranslationStage {
 public:
  explicit TranslationStage(const std::filesystem::path& directory) {
    const auto source_model = directory / "source.spm";
    const auto target_model = directory / "target.spm";

    if (!source_tokenizer_.Load(source_model.string()).ok() ||
        !target_tokenizer_.Load(target_model.string()).ok()) {
      throw std::runtime_error("failed to load SentencePiece model");
    }

    ctranslate2::models::ModelLoader model_loader(directory.string());
    translator_ = std::make_unique<ctranslate2::Translator>(model_loader);
  }

  std::string translate(const std::string& input) {
    if (input.empty()) {
      return {};
    }

    std::vector<std::string> source_tokens;
    if (!source_tokenizer_.Encode(input, &source_tokens).ok() || source_tokens.empty()) {
      throw std::runtime_error("failed to tokenize translation input");
    }

    std::vector<std::string_view> source_token_views;
    source_token_views.reserve(source_tokens.size());
    for (const auto& token : source_tokens) {
      source_token_views.emplace_back(token);
    }

    const auto results = translator_->translate_batch({source_token_views});
    if (results.size() != 1) {
      throw std::runtime_error("translation returned an unexpected batch size");
    }

    std::string output;
    if (!target_tokenizer_.Decode(results[0].output(), &output).ok()) {
      throw std::runtime_error("failed to decode translation output");
    }
    return output;
  }

 private:
  sentencepiece::SentencePieceProcessor source_tokenizer_;
  sentencepiece::SentencePieceProcessor target_tokenizer_;
  std::unique_ptr<ctranslate2::Translator> translator_;
};

class OpusRuntime {
 public:
  explicit OpusRuntime(const std::filesystem::path& pack_directory)
      : ja_en_(pack_directory / "stages" / "ja-en"),
        en_id_(pack_directory / "stages" / "en-id") {}

  bool translate(const std::string& input, std::string& output) {
    try {
      const std::string english = ja_en_.translate(input);
      output = en_id_.translate(english);
      return true;
    } catch (const std::exception&) {
      return false;
    }
  }

 private:
  TranslationStage ja_en_;
  TranslationStage en_id_;
};

std::uint64_t read_u64_le(const std::uint8_t* bytes) {
  std::uint64_t value = 0;
  for (std::size_t index = 0; index < 8; ++index) {
    value |= static_cast<std::uint64_t>(bytes[index]) << (index * 8U);
  }
  return value;
}

std::uint32_t read_u32_le(const std::uint8_t* bytes) {
  std::uint32_t value = 0;
  for (std::size_t index = 0; index < 4; ++index) {
    value |= static_cast<std::uint32_t>(bytes[index]) << (index * 8U);
  }
  return value;
}

void write_u64_le(std::uint8_t* bytes, std::uint64_t value) {
  for (std::size_t index = 0; index < 8; ++index) {
    bytes[index] = static_cast<std::uint8_t>((value >> (index * 8U)) & 0xffU);
  }
}

void write_u32_le(std::uint8_t* bytes, std::uint32_t value) {
  for (std::size_t index = 0; index < 4; ++index) {
    bytes[index] = static_cast<std::uint8_t>((value >> (index * 8U)) & 0xffU);
  }
}

ReadStatus read_exact(HANDLE pipe, std::uint8_t* output, std::size_t length) {
  std::size_t offset = 0;
  while (offset < length) {
    DWORD bytes_read = 0;
    const auto remaining = static_cast<DWORD>(length - offset);
    if (!ReadFile(pipe, output + offset, remaining, &bytes_read, nullptr)) {
      const DWORD error = GetLastError();
      if (offset == 0 && (error == ERROR_BROKEN_PIPE || error == ERROR_NO_DATA)) {
        return ReadStatus::Closed;
      }
      return ReadStatus::Error;
    }
    if (bytes_read == 0) {
      return offset == 0 ? ReadStatus::Closed : ReadStatus::Error;
    }
    offset += bytes_read;
  }
  return ReadStatus::Ok;
}

bool write_exact(HANDLE pipe, const std::uint8_t* input, std::size_t length) {
  std::size_t offset = 0;
  while (offset < length) {
    DWORD bytes_written = 0;
    const auto remaining = static_cast<DWORD>(length - offset);
    if (!WriteFile(pipe, input + offset, remaining, &bytes_written, nullptr) || bytes_written == 0) {
      return false;
    }
    offset += bytes_written;
  }
  return true;
}

bool parse_header(const std::array<std::uint8_t, kHeaderLen>& bytes, Header& header) {
  for (std::size_t index = 0; index < kMagic.size(); ++index) {
    if (bytes[index] != kMagic[index]) {
      return false;
    }
  }
  if (bytes[4] != kProtocolVersion) {
    return false;
  }

  header.message_type = bytes[5];
  header.request_id = read_u64_le(bytes.data() + 6);
  header.payload_len = read_u32_le(bytes.data() + 14);
  return header.payload_len <= kMaxPayloadLen;
}

bool write_frame(
    HANDLE pipe,
    std::uint8_t message_type,
    std::uint64_t request_id,
    const std::uint8_t* payload,
    std::uint32_t payload_len) {
  std::array<std::uint8_t, kHeaderLen> header{};
  for (std::size_t index = 0; index < kMagic.size(); ++index) {
    header[index] = kMagic[index];
  }
  header[4] = kProtocolVersion;
  header[5] = message_type;
  write_u64_le(header.data() + 6, request_id);
  write_u32_le(header.data() + 14, payload_len);

  if (!write_exact(pipe, header.data(), header.size())) {
    return false;
  }
  return payload_len == 0 || write_exact(pipe, payload, payload_len);
}

bool write_error(HANDLE pipe, std::uint64_t request_id, std::uint8_t code) {
  return write_frame(pipe, kErrorResponse, request_id, &code, 1);
}

bool deterministic_test_mode() {
  const char* value = std::getenv("CLIPLINGO_WORKER_TEST_MODE");
  return value != nullptr && std::strcmp(value, "deterministic") == 0;
}

std::unique_ptr<OpusRuntime> load_runtime() {
  const char* pack_directory = std::getenv("CLIPLINGO_MODEL_PACK");
  if (pack_directory == nullptr || pack_directory[0] == '\0') {
    return nullptr;
  }

  try {
    return std::make_unique<OpusRuntime>(std::filesystem::path(pack_directory));
  } catch (const std::exception&) {
    return nullptr;
  }
}

int serve_connection(HANDLE pipe, OpusRuntime* runtime, bool deterministic) {
  while (true) {
    std::array<std::uint8_t, kHeaderLen> header_bytes{};
    const ReadStatus header_status = read_exact(pipe, header_bytes.data(), header_bytes.size());
    if (header_status == ReadStatus::Closed) {
      return 0;
    }
    if (header_status == ReadStatus::Error) {
      return 10;
    }

    Header header{};
    if (!parse_header(header_bytes, header)) {
      return 11;
    }

    std::vector<std::uint8_t> payload(header.payload_len);
    if (header.payload_len > 0) {
      const ReadStatus payload_status = read_exact(pipe, payload.data(), payload.size());
      if (payload_status != ReadStatus::Ok) {
        return 12;
      }
    }

    if (header.message_type != kTranslateRequest) {
      if (!write_error(pipe, header.request_id, kUnsupportedRequest)) {
        return 13;
      }
      continue;
    }

    std::vector<std::uint8_t> translated;
    if (deterministic) {
      if (payload.size() > kMaxPayloadLen - kFakePrefix.size()) {
        if (!write_error(pipe, header.request_id, kTranslationFailed)) {
          return 14;
        }
        continue;
      }
      translated.reserve(kFakePrefix.size() + payload.size());
      translated.insert(translated.end(), kFakePrefix.begin(), kFakePrefix.end());
      translated.insert(translated.end(), payload.begin(), payload.end());
    } else {
      if (runtime == nullptr) {
        if (!write_error(pipe, header.request_id, kTranslationFailed)) {
          return 14;
        }
        continue;
      }

      const std::string input(payload.begin(), payload.end());
      std::string output;
      if (!runtime->translate(input, output) || output.size() > kMaxPayloadLen) {
        if (!write_error(pipe, header.request_id, kTranslationFailed)) {
          return 14;
        }
        continue;
      }
      translated.assign(output.begin(), output.end());
    }

    if (!write_frame(
            pipe,
            kTranslateResponse,
            header.request_id,
            translated.data(),
            static_cast<std::uint32_t>(translated.size()))) {
      return 15;
    }
  }
}

}  // namespace

int wmain() {
  HANDLE pipe = CreateNamedPipeW(
      kPipeName,
      PIPE_ACCESS_DUPLEX,
      PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
      1,
      64U * 1024U,
      64U * 1024U,
      0,
      nullptr);
  if (pipe == INVALID_HANDLE_VALUE) {
    return 2;
  }

  const bool deterministic = deterministic_test_mode();
  std::unique_ptr<OpusRuntime> runtime;
  if (!deterministic) {
    runtime = load_runtime();
    if (!runtime) {
      CloseHandle(pipe);
      return 4;
    }
  }

  const BOOL connected = ConnectNamedPipe(pipe, nullptr);
  if (!connected && GetLastError() != ERROR_PIPE_CONNECTED) {
    CloseHandle(pipe);
    return 3;
  }

  const int result = serve_connection(pipe, runtime.get(), deterministic);
  DisconnectNamedPipe(pipe);
  CloseHandle(pipe);
  return result;
}
