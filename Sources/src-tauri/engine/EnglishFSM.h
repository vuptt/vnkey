#ifndef ENGLISH_FSM_H
#define ENGLISH_FSM_H

#include <string>

namespace vnkey {

// Check if a word matches English syllable structure (phonotactics).
bool isValidEnglishWord(const std::string& word);

} // namespace vnkey

#endif // ENGLISH_FSM_H
