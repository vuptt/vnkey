# VnKey Project Index

## Overview

**VnKey** is a cross-platform Vietnamese input method editor (IME) built with a **C++ input engine** wrapped in a **Tauri 2** (Rust + SvelteKit) desktop application. It supports macOS, Windows, and Linux. The original VNKey was a macOS-only app; the Tauri rewrite makes it cross-platform.

- **Original author:** Tuyen Mai
- **Current maintainer:** theodore
- **License:** GPL
- **Identifier:** `com.theodore.vnkey`

---

## Table of Contents

1. [Project Structure](#project-structure)
2. [Architecture Overview](#architecture-overview)
3. [Core Engine (C++)](#core-engine-c)
4. [Rust/Tauri Backend](#rusttauri-backend)
5. [SvelteKit Frontend](#sveltekit-frontend)
6. [Build System](#build-system)
7. [Testing](#testing)
8. [Configuration & Data Flow](#configuration--data-flow)
9. [Features](#features)
10. [Key Technical Details](#key-technical-details)

---

## Project Structure

```
vnkey/
├── .gitignore
├── CHANGELOG.md
├── LICENSE
├── README.md
├── version.json                  # Version info for update checking
├── Sources/                      # Main source tree
│   ├── package.json              # Node.js dependencies (SvelteKit + Tauri)
│   ├── svelte.config.js          # SvelteKit config (static adapter)
│   ├── tsconfig.json             # TypeScript config
│   ├── vite.config.js            # Vite config (port 1420, Tauri integration)
│   ├── print_dict.rs             # Utility: decrypt & print English dictionary from DB
│   ├── src/                      # SvelteKit frontend
│   │   ├── app.html              # HTML shell
│   │   ├── lib/
│   │   │   └── tooltip.ts        # Reusable tooltip action (Svelte use:)
│   │   └── routes/
│   │       ├── +layout.ts        # Root layout (SSR disabled for SPA mode)
│   │       ├── +page.svelte      # Main control panel (~3573 lines)
│   │       ├── clipboard/
│   │       │   └── +page.svelte  # Clipboard history window picker
│   │       └── onboarding/
│   │           └── +page.svelte  # Accessibility permission onboarding
│   └── src-tauri/                # Tauri backend (Rust + C++)
│       ├── Cargo.toml            # Rust dependencies
│       ├── tauri.conf.json       # Tauri app configuration
│       ├── build.rs              # Build script (compiles C++ engine)
│       ├── capabilities/
│       │   └── default.json      # Tauri capability permissions
│       ├── icons/                # App icons (all platforms)
│       ├── engine/               # C++ input engine core
│       │   ├── Engine.h / .cpp   # Main engine API, event handling
│       │   ├── DataType.h        # Core type definitions, enums, masks
│       │   ├── Vietnamese.h/.cpp # Vietnamese character tables, input rules
│       │   ├── Macro.h/.cpp      # Macro (text expansion) system
│       │   ├── SmartSwitchKey.h/.cpp  # Per-app input method memory
│       │   ├── ConvertTool.h/.cpp     # Text encoding/code conversion
│       │   ├── EnglishDictionary.h/.cpp # Protected English words lexicon
│       │   └── platforms/        # Platform-specific key definitions
│       │       ├── mac.h
│       │       ├── linux.h
│       │       └── win32.h
│       └── src/                  # Rust source
│           ├── main.rs           # Entry point
│           ├── lib.rs            # App setup, tray, commands (~1995 lines)
│           ├── engine.rs         # FFI bindings to C++ engine
│           ├── engine_wrapper.cpp# C++ wrapper exposing engine globals to Rust
│           ├── db.rs             # SQLite database (English dict, macros, clipboard, KV store)
│           ├── cloud_sync.rs     # Cloud sync via S3-compatible (R2)
│           ├── google_sync.rs    # Cloud sync via Google Drive
│           ├── macos_apps.mm     # macOS: get running app info (Objective-C++)
│           └── tauri_event_tap.mm# macOS: CGEventTap keyboard hook (Objective-C++)
├── scripts/
│   ├── build_all.sh              # Detect OS and run appropriate build
│   ├── build_macos.sh            # macOS: build, DMG, install, uninstall
│   ├── build_linux.sh            # Linux: build (.deb, .AppImage)
│   ├── build_windows.bat         # Windows: build (.msi, .exe)
│   └── README.md                 # Build script documentation
└── Tests/
    ├── EngineCorpusTest.cpp      # Corpus-based engine correctness & latency test
    ├── run_engine_corpus_test.sh # Shell script to compile and run corpus test
    └── README.md                 # Test documentation
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                Tauri Desktop Shell (Rust)                │
│  ┌──────────────────────────────────────────────────┐   │
│  │            macOS CGEventTap (tauri_event_tap.mm) │   │
│  │  CGEventTapCreate → VNKeyCallback()              │   │
│  │  Intercepts keyboard events → sends to C++ Engine│   │
│  └──────────────┬───────────────────────────────────┘   │
│                 │                                        │
│                 ▼                                        │
│  ┌──────────────────────────────────────────────────┐   │
│  │           C++ Input Engine (engine/)              │   │
│  │  vKeyInit() → vKeyHandleEvent() → output codes    │   │
│  │  Vietnamese + Telex/VNI + Macro + Convert Tool    │   │
│  └──────────────┬───────────────────────────────────┘   │
│                 │                                        │
│                 ▼                                        │
│  ┌──────────────────────────────────────────────────┐   │
│  │  Rust Backend (lib.rs)                           │   │
│  │  - Tauri commands (IPC)                          │   │
│  │  - Tray icon + menu                              │   │
│  │  - Clipboard history manager                     │   │
│  │  - Cloud sync (R2 + Google Drive)                │   │
│  │  - SQLite DB (db.rs)                             │   │
│  │  - Per-app config                                │   │
│  └──────────────┬───────────────────────────────────┘   │
│                 │                                        │
│                 ▼                                        │
│  ┌──────────────────────────────────────────────────┐   │
│  │       SvelteKit Frontend (WebView)                │   │
│  │  Main control panel (+page.svelte)                │   │
│  │  Clipboard picker (clipboard/+page.svelte)        │   │
│  │  Onboarding (onboarding/+page.svelte)             │   │
│  │  Communicates via @tauri-apps/api (invoke)        │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Data Flow (Key Press)

1. **OS** generates keyboard event
2. **CGEventTap** (`tauri_event_tap.mm`) intercepts it on macOS
3. **VNKeyCallback()** checks hotkeys, then sends to `vKeyHandleEvent()`
4. **C++ Engine** processes the key → produces output instructions (backspace count + new characters or restore)
5. **Callback** posts synthetic Unicode/keyboard events to the system
6. The **frontmost application** receives the typed Vietnamese characters

### Data Flow (Frontend ↔ Backend)

- Frontend calls `invoke("command_name", args)` → Rust handler via `#[tauri::command]`
- Engine state globals (`vLanguage`, `vInputType`, etc.) are read/written via FFI
- Settings are persisted to `settings.json` and mirrored to the C++ engine globals
- Events are emitted from Rust to frontend via `handle.emit("event-name", payload)`

---

## Core Engine (C++)

### Key Files

| File | Purpose |
|------|---------|
| `Engine.h` | Main API: `vKeyInit()`, `vKeyHandleEvent()`, global variable declarations |
| `Engine.cpp` | Implementation of `vKeyHandleEvent()` - the core input processing logic |
| `Vietnamese.h/.cpp` | Vietnamese character tables, Telex/VNI/SimpleTelex rules, `keyCodeToCharacter()` |
| `DataType.h` | Core types: `vKeyHookState`, `HoolCodeState`, enums, bit masks for marks/tones/caps |
| `Macro.h/.cpp` | Text expansion system: add/delete/find/save/load macros |
| `SmartSwitchKey.h/.cpp` | Per-app memory for input method selection |
| `ConvertTool.h/.cpp` | Text conversion between code tables (Unicode ↔ TCVN3 ↔ VNI Windows ↔ CP1258) |
| `EnglishDictionary.h/.cpp` | Protected English words that Telex shouldn't modify |

### Global Variables (declared in Engine.h, defined in engine_wrapper.cpp)

These are the shared mutable state between C++ and Rust via FFI:

| Variable | Type | Default | Purpose |
|----------|------|---------|---------|
| `vLanguage` | `int` | 1 (Vietnamese) | 0=English, 1=Vietnamese |
| `vInputType` | `int` | 0 (Telex) | 0=Telex, 1=VNI, 2=SimpleTelex1, 3=SimpleTelex2 |
| `vFreeMark` | `int` | 0 | Free mark position |
| `vCodeTable` | `int` | 0 | 0=Unicode, 1=TCVN3, 2=VNI-Win, 3=Unicode Compound, 4=CP1258 |
| `vSwitchKeyStatus` | `int` | `0x20000C31` | Packed hotkey: Cmd+Shift+Space |
| `vCheckSpelling` | `int` | 1 | Spelling check on/off |
| `vUseModernOrthography` | `int` | 0 | oà vs òa |
| `vQuickTelex` | `int` | 0 | cc=ch, gg=gi, etc. |
| `vRestoreIfWrongSpelling` | `int` | 0 | Restore key if invalid word |
| `vUseEnglishDictionary` | `int` | 1 | Protect English words |
| `vFixRecommendBrowser` | `int` | 1 | Browser autocomplete fix |
| `vUseMacro` | `int` | 1 | Macro expansion on/off |
| `vUseMacroInEnglishMode` | `int` | 0 | Macros in English mode |
| `vAutoCapsMacro` | `int` | 1 | Auto-capitalize macro output |
| `vUseSmartSwitchKey` | `int` | 1 | Per-app input method memory |
| `vUpperCaseFirstChar` | `int` | 0 | Auto-capitalize first letter |
| `vTempOffSpelling` | `int` | 0 | Ctrl key temporarily disables spelling |
| `vAllowConsonantZFWJ` | `int` | 0 | Allow consonants Z, F, W, J |
| `vQuickStartConsonant` | `int` | 0 | f→ph, j→gi, w→qu |
| `vQuickEndConsonant` | `int` | 0 | g→ng, h→nh, k→ch |
| `vRememberCode` | `int` | 1 | Remember code table per app |
| `vOtherLanguage` | `int` | 1 | Turn off VNKey for non-English input sources |
| `vTempOffVNKey` | `int` | 0 | Cmd key temporarily disables VNKey |
| `vSendKeyStepByStep` | `int` | 0 | Legacy key-by-key sending mode |
| `vFixChromiumBrowser` | `int` | 0 | Chromium-specific workaround |
| `vPerformLayoutCompat` | `int` | 0 | Keyboard layout compatibility mode |
| `vDisableHotkeys` | `int` | 0 | Temporarily disable hotkey recording |

### Core Engine Algorithm

The engine uses a **Backspace-based technique**:
1. Each keystroke is evaluated by `vKeyHandleEvent()`
2. The engine maintains a **typing buffer** of the current word
3. When a Vietnamese-modifying key is pressed (e.g., 's' for sắc tone in Telex):
   - If valid → engine sets `backspaceCount` and populates `charData[]` with replacement characters
   - If invalid → engine either restores the original or passes through
4. The event tap callback posts: N backspaces + replacement characters

### `vKeyHookState` Structure

```cpp
struct vKeyHookState {
    Byte code;           // vDoNothing, vWillProcess, vBreakWord, vRestore, vReplaceMaro
    Byte backspaceCount; // Number of backspace key events to send
    Byte newCharCount;   // Number of replacement characters
    Byte extCode;        // Extended code (word break, delete, normal key, empty)
    Uint32 charData[MAX_BUFF]; // New characters to send (in reverse order)
    vector<Uint32> macroKey;   // Macro key sequence
    vector<Uint32> macroData;  // Macro replacement data
};
```

### Input Method Types

| Value | Name | Special Keys |
|-------|------|-------------|
| 0 | Telex | aw, aa, ee, oo, ow, uw, + s/f/r/x/j for tones |
| 1 | VNI | 1-9 for marks, 0 for toggle |
| 2 | Simple Telex 1 | Simplified Telex rules |
| 3 | Simple Telex 2 | Alternative simplified rules |

### Code Tables

| Value | Name | Description |
|-------|------|-------------|
| 0 | Unicode dựng sẵn | Pre-composed Unicode (Normalization Form C) |
| 1 | TCVN3 (ABC) | Legacy Vietnamese standard (VSCII) |
| 2 | VNI Windows | Legacy VNI encoding |
| 3 | Unicode tổ hợp | Unicode combining characters (NFD) |
| 4 | Vietnamese Locale CP1258 | Windows code page 1258 |

---

## Rust/Tauri Backend

### Key Files

| File | Lines | Purpose |
|------|-------|---------|
| `main.rs` | 6 | Entry point: calls `vnkey_lib::run()` |
| `lib.rs` | 1995 | App setup, tray menu, all Tauri commands, clipboard manager, cloud sync orchestration |
| `engine.rs` | 381 | FFI bindings: extern C functions linking to C++ engine |
| `engine_wrapper.cpp` | 190 | C++ file compiled with engine: defines global variables, exposes wrapper functions |
| `db.rs` | 303 | SQLite database: English dict, macros, clipboard history, KV store (AES-256 encrypted) |
| `cloud_sync.rs` | 247 | Cloud sync via S3-compatible API (Cloudflare R2) |
| `google_sync.rs` | 276 | Cloud sync via Google Drive API (OAuth2 device flow) |
| `macos_apps.mm` | 116 | macOS: get running applications, app info, app icons (Obj-C++) |
| `tauri_event_tap.mm` | 1390 | macOS: CGEventTap keyboard hook, status icon rendering, clipboard operations (Obj-C++) |

### Tauri Commands

The following commands are registered via `generate_handler![]` in `lib.rs`:

#### Settings
- `get_settings` → `Settings` struct
- `update_settings(settings)` → saves to C++ globals + `settings.json`
- `reset_settings` → factory reset settings + English dictionary
- `disable_hotkeys(disable: bool)` → sets `vDisableHotkeys`

#### Macros
- `list_macros` → `Vec<MacroEntry>`
- `upsert_macro(shortcut, content)` → add or update macro
- `remove_macro(shortcut)` → delete macro

#### Convert Tool
- `convert_text(request: ConvertRequest)` → converted string
- `trigger_quick_convert` → converts current clipboard content

#### English Dictionary
- `get_english_dictionary` → `EnglishDictionary { custom_words }`
- `save_custom_english_words(words: String)` → save custom words

#### Accessibility
- `check_accessibility` → `bool` (AXIsProcessTrusted)
- `request_accessibility` → opens system preference prompt

#### Applications (macOS)
- `get_running_applications` → JSON list of running apps with bundle IDs, names, icons
- `get_application_info_by_path(path: String)` → app info from `.app` path
- `get_application_info_by_bundle_id(bundle_id: String)` → app info from bundle ID
- `get_application_info_by_name(name: String)` → app info from app name

#### Per-App Config
- `get_app_configs` → `HashMap<String, AppConfig>`
- `save_app_config(bundle_id, config)` → save per-app settings
- `remove_app_config(bundle_id)` → delete per-app settings

#### Clipboard History
- `get_clipboard_items` → `Vec<ClipboardItem>`
- `remove_clipboard_item(id)` → delete single item
- `clear_clipboard_history` → delete all items
- `strip_clipboard_formatting(id)` → remove HTML from text item
- `paste_clipboard_item(id, prev_pid)` → paste item to previous app
- `toggle_clipboard_picker_window` → show/hide clipboard window
- `hide_clipboard_picker_window` → hide clipboard window

#### Cloud Sync
- `sync_to_cloud(credentials)` → upload to R2
- `sync_from_cloud(credentials)` → download from R2
- `start_google_auth` → OAuth2 device auth initiation
- `poll_google_auth(device_code)` → poll for auth completion
- `sync_to_gdrive(password)` → upload to Google Drive
- `sync_from_gdrive(password)` → download from Google Drive

#### KV Store
- `get_kv(key)` → generic key-value get
- `set_kv(key, value)` → generic key-value set

#### App Lifecycle
- `quit` → stop event tap and exit

### C++ FFI Functions (in engine.rs → engine_wrapper.cpp)

All C++ functions are linked via `extern "C"` in `engine.rs`:
- `vKeyInit()` - Initialize engine
- `startNewSession()` - Start new word session
- `start_event_tap()` / `stop_event_tap()` - macOS event tap
- `do_quick_convert()` - Convert clipboard text
- `is_accessibility_granted()` - Check AX permission
- `request_accessibility_permission()` - Request AX permission
- `vnkey_macro_count()` / `vnkey_macro_text_at()` / `vnkey_macro_content_at()` - Macro query
- `vnkey_add_macro()` / `vnkey_delete_macro()` - Macro CRUD
- `vnkey_set_custom_english_words()` - Set protected words
- `vnkey_convert_text()` - Text conversion
- `vnkey_load_macros()` - Load macros from file
- Getter/setter pairs for all convert tool settings

### Tray Icon & Menu

Built in `build_tray_menu()`:
- Toggle language (VN/EN) with switch key accelerator
- Input type submenu (Telex, VNI, Simple Telex 1/2)
- Code table submenu (Unicode, TCVN3, VNI Windows, etc.)
- Convert tool / Quick convert
- Clipboard history
- Control panel, Macro settings, About
- Quit

Icon rendering via `get_tray_icon()`:
- macOS: dynamically generated status icon (18×18 PNG) with "V" or "E"
- Gray icon mode for Dark Mode compatibility
- Other platforms: static PNG from `/icons/`

### Cargo Dependencies (Cargo.toml)

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `tauri` | 2 | Desktop app framework |
| `tauri-plugin-opener` | 2 | Open URLs |
| `tauri-plugin-dialog` | 2 | File dialogs |
| `serde` / `serde_json` | 1 | Serialization |
| `rusqlite` | 0.31.0 | SQLite database |
| `aes-gcm` | 0.10 | Encryption for local DB |
| `pbkdf2` / `sha2` | - | Key derivation |
| `lazy_static` | 1.4.0 | Lazy statics |
| `reqwest` | 0.13.4 | HTTP client (Google Drive OAuth) |
| `rust-s3` | 0.33.0 | S3-compatible (R2) cloud sync |
| `uuid` | 1 | Unique IDs for clipboard items |
| `chrono` | 0.4 | Timestamps |
| `base64` | 0.22 | Base64 encoding |
| `rand` | 0.8 | Random nonce generation |
| `tokio` | 1.52.3 | Async runtime |
| `urlencoding` | 2.1.3 | URL encoding |
| `cc` | 1.0 | Build C++ engine |

### Database Schema (SQLite via db.rs)

**Location:** `~/Library/Application Support/com.theodore.vnkey/vnkey.db`

All data is encrypted at rest using AES-256-GCM with a PBKDF2-derived key.

#### `english_dict` Table
| Column | Type | Description |
|--------|------|-------------|
| `word` | TEXT PRIMARY KEY | Encrypted English word |

#### `macros` Table
| Column | Type | Description |
|--------|------|-------------|
| `shortcut` | TEXT PRIMARY KEY | Encrypted macro shortcut |
| `content` | TEXT NOT NULL | Encrypted macro content |

#### `clipboard_history` Table
| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PRIMARY KEY | UUID v4 |
| `timestamp` | INTEGER NOT NULL | Milliseconds since epoch |
| `payload` | TEXT NOT NULL | Encrypted JSON of ClipboardItem |

#### `app_kv_store` Table
| Column | Type | Description |
|--------|------|-------------|
| `key` | TEXT PRIMARY KEY | KV key name |
| `value` | TEXT NOT NULL | Encrypted value |

---

## SvelteKit Frontend

### Pages

#### Main Control Panel (`/`)
- **File:** `Sources/src/routes/+page.svelte` (~3573 lines)
- **Tabs:**
  1. **Main Settings** - Language, input type, code table, toggles for all features
  2. **Macro** - List/add/delete text expansion entries with search
  3. **Convert Tool** - Text conversion between code tables with cap options
  4. **Clipboard** - Clipboard history settings (enabled, pin on top, auto-hide, max items, hotkey)
  5. **About** - Version info
  6. **App Config** - Per-app settings management with app selector
  7. **Cloud Sync** - R2 / Google Drive sync with sync options
- **Hotkey Recording:** Interactive key capture for switch key, convert hotkey, clipboard hotkey
- **Accessibility Check:** Polls permission, shows onboarding if not granted
- **Handles all settings fields** defined in `Settings` interface

#### Clipboard Picker (`/clipboard`)
- **File:** `Sources/src/routes/clipboard/+page.svelte` (~1111 lines)
- Displays clipboard history with search, keyboard navigation
- Supports: text (with HTML formatting), images (thumbnails), files (with icon stacks)
- Show action buttons: strip formatting, delete, paste
- Keyboard shortcuts: ↑↓ navigate, Enter/Tab paste, 1-9 quick paste, Esc close
- Pin-on-top and auto-hide toggle buttons
- Window blur → auto-hide (configurable)

#### Onboarding (`/onboarding`)
- **File:** `Sources/src/routes/onboarding/+page.svelte` (~290 lines)
- Simple page prompting user to grant Accessibility permission
- Animated logo, step-by-step guide
- "Grant Accessibility" and "Quit" buttons

### Key Frontend Details

- **SSR disabled** (`+layout.ts`): SPA mode via `@sveltejs/adapter-static`
- **Theme:** Auto-detects dark/light mode via `prefers-color-scheme` media query
- **Tooltip:** Custom floating tooltip action (`$lib/tooltip.ts`)
- **Communication:** Uses `@tauri-apps/api/core` for `invoke()` and `listen()`
- **Icons:** Inline SVG components throughout

---

## Build System

### Prerequisites
- Xcode Command Line Tools (macOS)
- Node.js + npm
- Rust stable + Cargo
- C++ compiler (clang/g++)

### Build Scripts

All scripts are in `/scripts/`:

#### `build_all.sh`
Auto-detects OS and dispatches to the appropriate build script.

#### `build_macos.sh [action]`
| Action | Description |
|--------|-------------|
| `build` (default) | Build `.app` bundle |
| `build-installer` (or `dmg`) | Build `.app` + `.dmg` |
| `install` | Build, install to `/Applications/`, replace running app |
| `clean` | Remove `target/`, `node_modules/`, `.svelte-kit/`, `build/` |
| `uninstall` | Remove `/Applications/VNKey.app` |

Build output goes to `Sources/src-tauri/target/release/bundle/` and is also copied to `/.build/`.

#### `build_linux.sh`
Builds `.deb` and `.AppImage` packages via Tauri.

#### `build_windows.bat`
Builds `.msi` and `.exe` installers via Tauri.

### Build Process
1. `npm ci` - Install Node dependencies
2. `npm run check` - Run SvelteKit type check
3. `npm run tauri build` - Build frontend + Rust/C++ → native app
4. macOS: Code signs with ad-hoc signature (`-`)

### Rust Dependencies (Build)
- `tauri-build` v2
- `cc` v1.0 (to compile `engine_wrapper.cpp` + C++ engine files)

### Frontend Dependencies (package.json)
| Package | Version | Purpose |
|---------|---------|---------|
| `@tauri-apps/api` | ^2 | Tauri IPC |
| `@tauri-apps/plugin-dialog` | ^2.7.1 | File/dialog plugin |
| `@tauri-apps/plugin-opener` | ^2 | Open URLs plugin |
| `@sveltejs/adapter-static` | ^3.0.10 | Static site adapter |
| `@sveltejs/kit` | ^2.66.0 | SvelteKit framework |
| `svelte` | ^5.0.0 | Svelte 5 |
| `@tauri-apps/cli` | ^2 | Tauri CLI |

---

## Testing

### Engine Corpus Test (`Tests/EngineCorpusTest.cpp`)
A C++ test that validates the input engine against a 10,000-word Vietnamese corpus.

**Compilation:** `Tests/run_engine_corpus_test.sh` (uses clang++)

**Features:**
- `--interactive` mode: diagnose what Telex does to any input word
- `--benchmark` mode: measure lookup + engine latency (median, P95, P99, max)
- Default mode: evaluates 4 policies:
  - **Baseline** (no restore) - measures raw engine accuracy
  - **Restore** (auto-restore, no dictionary)
  - **Structural** (auto-restore + structural English hints)
  - **Protected Lexicon** (auto-restore + protected word dictionary)
- Tracks: unique failed words, failed occurrences, false restores
- Ambiguous word detection (e.g., "Docs" → "Dóc" vs keep as "Docs")
- Exit code 1 if performance degrades across policies

### Rust Unit Tests
In `engine.rs`:
- `conversion_preserves_case_by_default` - verifies identity conversion
- `macro_can_be_added_listed_and_removed` - verifies macro CRUD

---

## Configuration & Data Flow

### Settings Persistence
```
settings.json (disk, JSON)
     ↕
Rust `Settings` struct (lib.rs)
     ↕  (via FFI)
C++ global variables (engine_wrapper.cpp)
     ↕  (via callbacks)
macOS event tap (tauri_event_tap.mm)
```

### Per-App Settings
```
app_settings.json (disk, JSON)
    ↕
Rust `AppConfig` struct (lib.rs)
    ↕  (polling thread, 250ms interval)
On app switch → apply_app_config_by_bundle_id()
```

### Cloud Sync Flow
```
User data → encrypt with sync password (PBKDF2 + AES-256) → upload to R2 / Google Drive
Download → decrypt → apply to local DB & engine
Auto-sync: debounced 5s after any macro/dict/clipboard change
```

### Clipboard History Flow
```
macOS NSPasteboard polling (250ms)
    ↕
changeCount detection → deduplication → save to SQLite
    ↕
Rust Vec<ClipboardItem> (in-memory cache)
    ↕  (emit "clipboard-changed")
Frontend clipboard picker
```

---

## Features

### Vietnamese Input
- **Input methods:** Telex, VNI, Simple Telex 1/2
- **Code tables:** Unicode, TCVN3 (ABC), VNI Windows, Unicode Compound, CP1258
- **Modern orthography:** `oà` / `uý` vs `òa` / `úy`
- **Quick Telex:** cc=ch, gg=gi, kk=kh, nn=ng, qq=qu, pp=ph, tt=th
- **Quick consonants:** f→ph, j→gi, w→qu (initial) / g→ng, h→nh, k→ch (final)
- **Spelling check** with auto-restore for invalid words
- **English dictionary** protection for common English words
- **Auto-capitalize first letter** of sentences

### Smart Switching
- **Smart Switch Key:** Per-app input method memory
- **Other Language Detection:** Auto-disable for non-Vietnamese input sources
- **Temporary disable:** Ctrl key (spelling) / Cmd key (VNKey) / Hotkey

### Macro System
- Text expansion (unlimited length)
- Case-preserving (AutoCaps)
- Works in English mode (optional)
- Persisted in SQLite DB

### Convert Tool
- Convert text between code tables
- Options: all caps, all non-caps, first letter caps, each word caps, remove marks
- Quick convert from clipboard via hotkey

### Clipboard History
- History: text, HTML, images (PNG), files
- Deduplication by content
- Search + keyboard navigation
- Pin on top, auto-hide on blur
- Max items configurable (5-200)
- Thumbnails for images/files
- Strip HTML formatting

### Cloud Sync
- S3-compatible (Cloudflare R2)
- Google Drive (OAuth2 device flow)
- Selective sync: settings, English dict, macros, clipboard, app configs
- AES-256 encryption with password-derived key
- Auto-sync on changes (debounced 5s)

### Per-App Configuration
- Override input method, code table, spelling, etc. per application
- Browse running apps or select from Finder
- Smart apply on app switch

### Accessibility
- macOS Accessibility permission required for event tap
- Onboarding window for first-time setup
- Background polling for permission grant

---

## Key Technical Details

### Hotkey Encoding
Hotkeys are packed into a 32-bit integer:
- Bits 0-7: Key code
- Bit 8: Control
- Bit 9: Option/Alt
- Bit 10: Command
- Bit 11: Shift
- Bit 15: Beep
- Bits 24-31: Character code (for display)
- `0xFE0000FE` = EMPTY_HOTKEY (no hotkey set)

### Default Hotkeys
| Function | Value | Display |
|----------|-------|---------|
| Switch Language | `0x20000C31` | ⌘⇧Space |
| Convert Tool | `0xFE0000FE` | Not set |
| Clipboard | `0x76000109` | ⌃V |

### macOS-Specific Architecture

#### Event Tap (`tauri_event_tap.mm`)
- `CGEventTapCreate()` at session level with `kCGHeadInsertEventTap`
- Intercepts: keyDown, keyUp, flagsChanged, mouseDown/Dragged
- Own `CGEventSource` (`kCGEventSourceStatePrivate`) for synthesized events
- Filters own events via `kCGEventSourceStateID` check
- Unicode string posting via `CGEventKeyboardSetUnicodeString()`
- Keyboard layout compatibility: `charactersIgnoringModifiers` → key code mapping

#### App Detection (`macos_apps.mm`)
- `NSWorkspace` APIs for frontmost app, running apps
- 64×64 PNG icon generation via `NSBitmapImageRep`
- Base64-encoded icons for display in frontend

#### Clipboard Operations
- `NSPasteboard` for reading/writing text, HTML, file URLs
- Image reading via `NSPasteboardTypePNG` / `NSPasteboardTypeTIFF`
- `CGWindowListCopyWindowInfo` for Spotlight detection

### Encryption (db.rs)
- **Local DB:** AES-256-GCM with static PBKDF2-derived key (`vnkey_local_storage_secret` + `vnkey_salt_123456`)
- **Cloud Sync:** AES-256-GCM with user-supplied password via PBKDF2 (`vnkey_cloud_salt`, 100,000 iterations)
- All sensitive data encrypted before SQLite storage

### Versioning
- `version.json`: `{ latestVersion: { versionName: "1.0", versionCode: 1 }, latestWinVersion: { ... } }`
- `Cargo.toml` version: `1.0.0`
- `package.json` version: `1.0.0`
- `tauri.conf.json` productName: `VNKey` version: `1.0.0`

---

## File Summary

| Language | Files | Purpose |
|----------|-------|---------|
| **C++** | 12 | Input engine core, macros, conversion, dictionary |
| **Rust** | 8 | Tauri backend, DB, cloud sync, FFI bridge |
| **Objective-C++** | 2 | macOS event tap, app detection, clipboard |
| **TypeScript** | 4 | SvelteKit frontend pages + layout |
| **JavaScript/Config** | 5 | Vite, SvelteKit, TypeScript, package configs |
| **Shell** | 4 | Build scripts (macOS, Linux, all, test runner) |
| **Batch** | 1 | Windows build script |
| **Markdown** | 4 | README, CHANGELOG, script README, test README |
| **JSON** | 4 | Config, capabilities, version |

---

## Git

- **Repository:** `https://github.com/hoquangthaiholy/vnkey.git` (fork)
- **Original:** `https://github.com/hoquangthaiholy/vnkey.git`
- **Latest Commit:** `98ec1edf27947d9c178b87bdc425150f801549f3`