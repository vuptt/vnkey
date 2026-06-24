//
//  EnglishDictionary.cpp
//  VNKey
//

#include "EnglishDictionary.h"

#include <algorithm>
#include <cctype>
#include <iterator>
#include <memory>
#include <sstream>
#include <unordered_set>

const std::string& getDefaultEnglishWords() {
  static const std::string words =
      "password rust issue coffee wrong software framework network browser download website search support workflow feedback dashboard deadline meeting google excel slack chrome youtube facebook instagram twitter microsoft office word powerpoint code vs notion telegram laptop email login logout upload file backend frontend button modal form checkbox dropdown github json http websocket database macos windows linux benchmark median p95 p99 space enter tab ai api cpu gpu wifi bluetooth iphone android docker kubernetes javascript typescript python swift production antivirus free write screen user";
  return words;
}

namespace {

// Keep sorted for binary_search. This is a protected lexicon, not a complete
// English dictionary: only words that are common in Vietnamese technical text
// and are vulnerable to Telex transformations belong here.


std::string lowerAscii(const std::string &word) {
  std::string normalized;
  normalized.reserve(word.size());
  for (const unsigned char character : word) {
    if (!std::isalpha(character)) {
      return std::string();
    }
    normalized.push_back(static_cast<char>(std::tolower(character)));
  }
  return normalized;
}

using EnglishWordSet = std::unordered_set<std::string>;

std::shared_ptr<const EnglishWordSet> gCustomEnglishWords = []() {
  auto customWords = std::make_shared<EnglishWordSet>();
  std::stringstream ss(getDefaultEnglishWords());
  std::string word;
  while (ss >> word) {
    if (word.empty()) continue;
    std::string normalized = lowerAscii(word);
    if (!normalized.empty()) {
      customWords->insert(normalized);
    }
  }
  return customWords;
}();

} // namespace

bool isProtectedEnglishWord(const std::string &word) {
  const std::string normalized = lowerAscii(word);
  if (normalized.empty()) {
    return false;
  }
  const auto customWords = std::atomic_load(&gCustomEnglishWords);
  return customWords->count(normalized) > 0;
}

void setCustomEnglishWords(const std::string& content) {
  auto customWords = std::make_shared<EnglishWordSet>();
  std::stringstream ss(content);
  std::string word;
  while (ss >> word) {
    if (word.empty()) continue;
    if (word[0] == '#') {
      std::string comment;
      std::getline(ss, comment);
      continue;
    }
    std::string normalized = lowerAscii(word);
    if (!normalized.empty()) {
      customWords->insert(normalized);
    }
  }
  std::atomic_store(
      &gCustomEnglishWords,
      std::static_pointer_cast<const EnglishWordSet>(customWords));
}


