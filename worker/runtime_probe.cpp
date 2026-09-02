#include <ctranslate2/translator.h>
#include <sentencepiece_processor.h>

#include <iostream>
#include <string>
#include <string_view>
#include <vector>

namespace {

bool probe_sentencepiece(const std::string& model_path) {
  sentencepiece::SentencePieceProcessor processor;
  const auto load_status = processor.Load(model_path);
  if (!load_status.ok()) {
    std::cerr << "runtime_probe component=sentencepiece status=load_error\n";
    return false;
  }

  const std::string input = "This is a tokenizer probe.";
  std::vector<std::string> pieces;
  const auto encode_status = processor.Encode(input, &pieces);
  if (!encode_status.ok() || pieces.empty()) {
    std::cerr << "runtime_probe component=sentencepiece status=encode_error\n";
    return false;
  }

  std::string decoded;
  const auto decode_status = processor.Decode(pieces, &decoded);
  if (!decode_status.ok() || decoded.empty()) {
    std::cerr << "runtime_probe component=sentencepiece status=decode_error\n";
    return false;
  }

  return true;
}

bool probe_ctranslate2(const std::string& model_path) {
  const std::vector<std::string_view> input = {"آ", "ت", "ز", "م", "و", "ن"};
  const std::vector<std::string> expected = {"a", "t", "z", "m", "o", "n"};

  try {
    ctranslate2::models::ModelLoader model_loader(model_path);
    ctranslate2::Translator translator(model_loader);
    const auto results = translator.translate_batch({input});
    if (results.size() != 1 || results[0].output() != expected) {
      std::cerr << "runtime_probe component=ctranslate2 status=unexpected_output\n";
      return false;
    }
  } catch (const std::exception&) {
    std::cerr << "runtime_probe component=ctranslate2 status=translation_error\n";
    return false;
  }

  return true;
}

}  // namespace

int main(int argc, char* argv[]) {
  if (argc != 3) {
    std::cerr << "usage: cliplingo-worker-runtime-probe <ct2-model> <sentencepiece-model>\n";
    return 2;
  }

  if (!probe_ctranslate2(argv[1]) || !probe_sentencepiece(argv[2])) {
    return 1;
  }

  std::cout << "runtime_probe status=pass\n";
  return 0;
}
