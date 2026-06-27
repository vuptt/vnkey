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


struct TrieNode {
  bool isWord = false;
  TrieNode* children[26] = {nullptr};
  
  ~TrieNode() {
    for (int i = 0; i < 26; ++i) {
      delete children[i];
    }
  }
};

class Trie {
public:
  TrieNode* root;
  Trie() {
    root = new TrieNode();
  }
  ~Trie() {
    delete root;
  }
  
  void insert(const std::string& word) {
    TrieNode* curr = root;
    for (char c : word) {
      int idx = c - 'a';
      if (idx < 0 || idx >= 26) return;
      if (!curr->children[idx]) {
        curr->children[idx] = new TrieNode();
      }
      curr = curr->children[idx];
    }
    curr->isWord = true;
  }
  
  bool search(const std::string& word) const {
    TrieNode* curr = root;
    for (char c : word) {
      int idx = c - 'a';
      if (idx < 0 || idx >= 26) return false;
      if (!curr->children[idx]) return false;
      curr = curr->children[idx];
    }
    return curr->isWord;
  }
  
  bool startsWith(const std::string& prefix) const {
    if (prefix.empty()) return false;
    TrieNode* curr = root;
    for (char c : prefix) {
      int idx = c - 'a';
      if (idx < 0 || idx >= 26) return false;
      if (!curr->children[idx]) return false;
      curr = curr->children[idx];
    }
    return true;
  }
};

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

std::shared_ptr<const Trie> gCustomEnglishWords = []() {
  auto customTrie = std::make_shared<Trie>();
  std::stringstream ss(getDefaultEnglishWords());
  std::string word;
  while (ss >> word) {
    if (word.empty()) continue;
    std::string normalized = lowerAscii(word);
    if (!normalized.empty()) {
      customTrie->insert(normalized);
    }
  }
  return customTrie;
}();

} // namespace

bool isProtectedEnglishWord(const std::string &word) {
  const std::string normalized = lowerAscii(word);
  if (normalized.empty()) {
    return false;
  }
  const auto customTrie = std::atomic_load(&gCustomEnglishWords);
  return customTrie->search(normalized);
}

bool hasProtectedEnglishPrefix(const std::string& prefix) {
  const std::string normalized = lowerAscii(prefix);
  if (normalized.empty()) {
    return false;
  }
  const auto customTrie = std::atomic_load(&gCustomEnglishWords);
  return customTrie->startsWith(normalized);
}

void setCustomEnglishWords(const std::string& content) {
  auto customTrie = std::make_shared<Trie>();
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
      customTrie->insert(normalized);
    }
  }
  std::atomic_store(
      &gCustomEnglishWords,
      std::static_pointer_cast<const Trie>(customTrie));
}


