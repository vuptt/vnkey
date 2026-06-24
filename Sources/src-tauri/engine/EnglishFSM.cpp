#include "EnglishFSM.h"
#include <unordered_set>
#include <vector>
#include <cctype>

namespace vnkey {

namespace {

bool isVowel(char c) {
    return c == 'a' || c == 'e' || c == 'i' || c == 'o' || c == 'u' || c == 'y';
}

const std::unordered_set<std::string> validOnsets2 = {
    "bl", "br", "ch", "cl", "cr", "dr", "dw", "fl", "fr", "gl", "gr", "gh", "kn", "ph", "pl", "pr", "sc", "sh", "sk", "sl", "sm", "sn", "sp", "st", "sw", "th", "tr", "tw", "wh", "wr"
};

const std::unordered_set<std::string> validOnsets3 = {
    "chr", "scr", "spl", "spr", "str", "thr", "sph"
};

const std::unordered_set<std::string> validCodas2 = {
    "bs", "ch", "ck", "ct", "ds", "ff", "ft", "gg", "gh", "gs",
    "ld", "lf", "lk", "ll", "lm", "lp", "lt", "mb", "mp", "ms",
    "nd", "ng", "nk", "nl", "ns", "nt", "ny", "nx",
    "ph", "pt", "rd", "rf", "rg", "rk", "rl", "rm", "rn", "rp", "rs", "rt", "rv", "rx", "ry",
    "sh", "sk", "sp", "ss", "st", "th", "ts", "tt", "tz", "wd", "wn", "xt"
};

const std::unordered_set<std::string> validCodas3 = {
    "cts", "dge", "ght", "gth", "lch", "ldy", "lsh", "lts",
    "mps", "nch", "ndy", "ngs", "nks", "nst", "nts",
    "rch", "rds", "rgs", "rks", "rld", "rms", "rns", "rps", "rse", "rst", "rth", "rts", "rve",
    "shs", "sks", "sps", "sts", "ths", "xth"
};

const std::unordered_set<std::string> validCodas4 = {
    "ghts", "gths", "ldst", "ngth", "rths", "stts"
};

bool isValidOnset(const std::string& s) {
    if (s.empty()) return true;
    if (s.length() == 1) {
        // Any consonant except 'x' can start a word/syllable
        return s[0] != 'x';
    }
    if (s.length() == 2) {
        return validOnsets2.count(s) > 0;
    }
    if (s.length() == 3) {
        return validOnsets3.count(s) > 0;
    }
    return false;
}

bool isValidCoda(const std::string& s) {
    if (s.empty()) return true;
    if (s.length() == 1) {
        // English words don't typically end in h, j, q, v
        char c = s[0];
        return c != 'h' && c != 'j' && c != 'q' && c != 'v';
    }
    if (s.length() == 2) {
        return validCodas2.count(s) > 0;
    }
    if (s.length() == 3) {
        return validCodas3.count(s) > 0;
    }
    if (s.length() == 4) {
        return validCodas4.count(s) > 0 || (s[3] == 's' && validCodas3.count(s.substr(0, 3)) > 0);
    }
    return false;
}

// Can a sequence of consonants between two vowels be split into a valid Coda and a valid Onset?
bool canSplitConsonants(const std::string& s) {
    if (s.empty()) return true;
    for (size_t i = 0; i <= s.length(); ++i) {
        std::string left = s.substr(0, i);
        std::string right = s.substr(i);
        if (isValidCoda(left) && isValidOnset(right)) {
            return true;
        }
    }
    return false;
}

} // namespace

bool isValidEnglishWord(const std::string& word) {
    if (word.empty() || word.length() > 25) return false;
    
    // Normalize to lowercase
    std::string clean;
    for (char c : word) {
        if (!std::isalpha(static_cast<unsigned char>(c))) {
            return false;
        }
        clean.push_back(static_cast<char>(std::tolower(static_cast<unsigned char>(c))));
    }
    
    // Group into alternating Vowel (V) and Consonant (C) segments
    struct Segment {
        std::string chars;
        bool isV;
    };
    std::vector<Segment> segments;
    
    for (char c : clean) {
        bool currentV = isVowel(c);
        if (segments.empty() || segments.back().isV != currentV) {
            segments.push_back({std::string(1, c), currentV});
        } else {
            segments.back().chars.push_back(c);
        }
    }
    
    // Perform phonotactic checks on each segment
    for (size_t i = 0; i < segments.size(); ++i) {
        const Segment& seg = segments[i];
        if (seg.isV) {
            // Max 3 consecutive vowels (e.g. queue, beautiful)
            if (seg.chars.length() > 3) {
                return false;
            }
        } else {
            // Consonant segment checks
            if (i == 0) {
                // First segment is consonant -> must be valid onset
                if (!isValidOnset(seg.chars)) return false;
            } else if (i == segments.size() - 1) {
                // Last segment is consonant -> must be valid coda
                if (!isValidCoda(seg.chars)) return false;
            } else {
                // Consonants in the middle -> must be splittable into valid coda + onset
                if (!canSplitConsonants(seg.chars)) return false;
            }
        }
    }
    
    return true;
}

} // namespace vnkey
