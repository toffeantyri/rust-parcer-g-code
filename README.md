# G-Code Editor

Редактор и форматтер программ G-кода для станков с ЧПУ.

## Возможности

- **Форматирование** — расстановка пробелов между кодами, выравнивание, округление чисел
- **Перенумерация кадров** — автоматическая перенумерация N-кодов с заданным шагом
- **Валидация** — проверка G-кода на ошибки (ось без значения, пустая программа и т.д.)
- **Сохранение пустых строк** — структура исходного кода не ломается
- **Локализация** — русский и английский интерфейс
- **Горячие клавиши**: `F5` — форматировать, `F6` — проверить, `Ctrl+O` — открыть, `Ctrl+S` — сохранить
- **Графический интерфейс** на egui (immediate mode)

Программа **не изменяет**:
- Специфические машинные команды (`MODECHECK(2)`, `TRANS`, `CFTCP`, `MAMILL` и т.д.)
- Выражения вида `Z=71.304`, `X=160+10` (AxisExpr)
- Комментарии после `;`
- Координаты осей без G-кода на отдельных строках

## Установка и запуск

### 1. Установка Rust

Если Rust ещё не установлен:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

После установки перезапустите терминал или выполните:

```sh
source ~/.cargo/env
```

### 2. Клонирование репозитория

```sh
git clone https://github.com/toffeantyri/rust-parcer-g-code.git
cd rust-parcer-g-code
```

### 3. Запуск

```sh
cargo run --bin editor
```

При первом запуске Cargo скачает и соберёт все зависимости — это может занять несколько минут. Последующие запуски будут быстрыми.

## Системные требования

- **Rust** 1.81 или новее
- **Linux**, **macOS** или **Windows**
- Для Wayland (Linux) может потребоваться пакет `libxkbcommon`

## Разработка

```sh
# Сборка
cargo build

# Тесты
cargo test -- --test-threads=1

# Запуск GUI
cargo run --bin editor
```

### Архитектура проекта

```
src/
├── domain/          # Сущности и бизнес-логика (Token, Statement)
├── application/     # Use cases (Parser, Formatter, Validator)
├── infrastructure/  # Внешние зависимости (Lexer)
├── interfaces/      # GUI на egui (MVI архитектура)
├── data_layer/      # Отдельный поток для пайплайна и IO
└── shared/          # Общие типы (ошибки, i18n)
```
 сейчас у меня такое описание репозитория- перепиши на актуальный на русском и английском

## Assistant

```markdown
# G-Code Editor v0.2.1

Редактор, форматтер и валидатор программ G-кода для станков с ЧПУ с графическим интерфейсом.

## Возможности

- **Форматирование (F5)** — нумерация кадров с настраиваемым шагом, расстановка пробелов между кодами, сохранение целостности команд
- **Валидация (F6)** — проверка G-кода на ошибки: ось без значения, подозрительные двухбуквенные токены, пустая программа
- **Подсветка синтаксиса** — G-коды (зелёный), M-коды (синий), оси (жёлтый), AxisExpr (тёмно-жёлтый), комментарии (серый), управляющие конструкции (бордовый), строки с ошибками (красный фон)
- **Поиск по тексту (Ctrl+F)** — поиск без учёта регистра, навигация по вхождениям, жёлтая подсветка найденного
- **Замена текста (Ctrl+H)** — пошаговая замена с навигацией или замена всех вхождений
- **Замена осей** — поменять две оси местами (Swap) или инвертировать знак выбранной оси (Invert), включая AxisExpr с выражениями и R-параметрами
- **Поддержка управляющих конструкций** — WHILE/ENDWHILE, IF/ELSE/ENDIF, REPEAT/UNTIL, FOR/ENDFOR, LOOP/ENDLOOP с автоматическими отступами
- **Двуязычный интерфейс** — русский и английский, переключение на лету
- **Горячие клавиши** — F5, F6, Ctrl+O, Ctrl+S, Ctrl+Shift+S, Ctrl+F, Ctrl+H

## Программа не изменяет

- Специфические машинные команды (`MODECHECK`, `TRANS`, `CFTCP`, `MAMILL`, `WGTRANS`, `MSG`, `MATLCH` и др.)
- Выражения осей: `Z=71.304`, `X=160+10`, `Y=3*5/2`
- R-параметры: `R50`, `R101=R101+1`
- Комментарии после `;`
- Оси без G-кода на отдельных строках

## Установка и запуск

### 1. Установка Rust
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. Клонирование и запуск
```sh
git clone https://github.com/toffeantyri/rust-parcer-g-code.git
cd rust-parcer-g-code
cargo run --bin editor --features desktop
```

### 3. Сборка под Android
```sh
# Установить Java JDK 21+, LLVM/clang, Rust Android target
rustup target add aarch64-linux-android
cargo install xbuild

# Сборка и запуск на устройстве
# adb devices - device_id - copy
x run --features android --device adb:<device_id>
```

## Системные требования

- Rust 1.81+
- Linux, macOS или Windows

## Разработка

```sh
cargo build --bin editor --features desktop   # Сборка десктоп
cargo test                                    # Тесты (323 lib + 26 gui = 349)
cargo run --bin editor --features desktop       # Запуск GUI

# Android
x build --features android --format apk         # Сборка APK
x run --features android --device adb:<id>      # Запуск на устройстве
```

### Архитектура

```
src/
├── domain/          # Сущности, трейты (Token, Statement, Lexer)
├── application/     # Use cases (Parser, Formatter, Validator)
├── infrastructure/  # Реализации (лексер, подсветка, platform/)
│   └── platform/    # Платформенные адаптеры (desktop: rfd, android: SAF)
├── interfaces/      # GUI на egui (MVI: Model-View-Intent)
│   └── gui/
│       ├── app/     # desktop.rs (eframe) / android.rs (android-activity)
│       ├── view/    # view_desktop.rs / view_android.rs
│       ├── model/   # Общая модель (переиспользуется)
│       ├── intent/  # Общие интенты
│       └── update/  # Общая логика
├── data_layer/      # Отдельный поток (пайплайн: лексер→парсер→валидатор→форматтер)
└── shared/          # Утилиты, локализация (i18n), типы ошибок (thiserror)
```

---

# G-Code Editor v0.2.1

A G-code editor, formatter and validator for CNC machines with a graphical interface.

## Features

- **Formatting (F5)** — frame renumbering with configurable step, spacing between codes, preserving command integrity
- **Validation (F6)** — error detection: axis without value, suspicious two-letter tokens, empty program
- **Syntax highlighting** — G-codes (green), M-codes (blue), axes (yellow), AxisExpr (dark yellow), comments (gray), control structures (dark red), error lines (red background)
- **Text search (Ctrl+F)** — case-insensitive search, occurrence navigation, yellow match highlight
- **Text replace (Ctrl+H)** — step-by-step replace with navigation or replace all
- **Axis swap** — swap two axes or invert sign of selected axis, including AxisExpr with expressions and R-parameters
- **Control structures** — WHILE/ENDWHILE, IF/ELSE/ENDIF, REPEAT/UNTIL, FOR/ENDFOR, LOOP/ENDLOOP with auto-indentation
- **Bilingual UI** — Russian and English, on-the-fly switching
- **Keyboard shortcuts** — F5, F6, Ctrl+O, Ctrl+S, Ctrl+Shift+S, Ctrl+F, Ctrl+H

## Preserved as-is

- Machine-specific commands (`MODECHECK`, `TRANS`, `CFTCP`, `MAMILL`, `WGTRANS`, `MSG`, `MATLCH`, etc.)
- Axis expressions: `Z=71.304`, `X=160+10`, `Y=3*5/2`
- R-parameters: `R50`, `R101=R101+1`
- Comments after `;`
- Standalone axis lines without G-codes

## Quick Start

### 1. Install Rust
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. Clone and run
```sh
git clone https://github.com/toffeantyri/rust-parcer-g-code.git
cd rust-parcer-g-code
cargo run --bin editor --features desktop
```

### 3. Build for Android
```sh
# Install Java JDK 21+, LLVM/clang, Rust Android target
rustup target add aarch64-linux-android
cargo install xbuild

# Build and run on device
x run --features android --device adb:<device_id>
```

## Requirements

- Rust 1.81+
- Linux, macOS or Windows

## Development

```sh
cargo build --bin editor --features desktop   # Build desktop
cargo test                                    # Tests (323 lib + 26 gui = 349)
cargo run --bin editor --features desktop       # Run GUI

# Android
x build --features android --format apk         # Build APK
x run --features android --device adb:<id>      # Run on device
```

### Architecture

```
src/
├── domain/          # Entities and traits (Token, Statement, Lexer)
├── application/     # Use cases (Parser, Formatter, Validator)
├── infrastructure/  # Implementations (lexer, highlighting, platform/)
│   └── platform/    # Platform adapters (desktop: rfd, android: SAF)
├── interfaces/      # egui GUI (MVI: Model-View-Intent)
│   └── gui/
│       ├── app/     # desktop.rs (eframe) / android.rs (android-activity)
│       ├── view/    # view_desktop.rs / view_android.rs
│       ├── model/   # Shared model (reused across platforms)
│       ├── intent/  # Shared intents
│       └── update/  # Shared logic
├── data_layer/      # Separate thread (pipeline: lexer→parser→validator→formatter)
└── shared/          # Utilities, localization (i18n), error types (thiserror)
```
```
