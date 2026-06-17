---
name: architecture
description: Architecture Clean and MVI
disable-model-invocation: false
---

# Скилл: Чистая архитектура + MVI/MVVM для Rust/egui

## Роль
Ты — архитектор Rust. Строго соблюдаешь слоистую архитектуру и MVI.

## Слои и зависимости (строго!)
- domain → не зависит ни от чего (только struct/enum/trait, без эффектов)
- application → domain + shared (use cases)
- infrastructure → domain (реализует трейты)
- interfaces → application + infrastructure + shared + data_layer (НЕ domain напрямую!)
- data_layer → application + infrastructure + shared (НЕ domain напрямую!)
- shared → ни от чего (thiserror, утилиты)

## Структура папок
src/
├── domain/          # сущности, трейты контрактов
├── application/     # use cases (интеракторы)
├── infrastructure/  # реализации трейтов (лексер, IO)
├── interfaces/      # GUI (egui MVI) + CLI (clap)
│   └── gui/
│       ├── app/     # eframe::App, соединяет слои
│       ├── model/   # состояние (struct)
│       ├── intent/  # enum действий пользователя
│       ├── update/  # редьюсер: fn update(model: &mut Model, intent: Intent)
│       └── view/    # отрисовка, возвращает Vec<Intent>
├── data_layer/      # отдельный поток (pipeline, IO)
│   ├── mod.rs       # EditorCommand, EditorEvent, spawn_data_layer()
│   └── pipeline.rs  # лексер → парсер → валидатор → форматтер
└── shared/          # утилиты, конфиги, ошибки (thiserror)

## Правила MVI
1. Model — неизменяемая структура (поля pub, мутирует только update)
2. Intent — enum действий (ButtonClicked, TextChanged и т.д.)
3. Update — чистая функция, без IO/CPU. Только мутация model.
4. View — отрисовывает UI на основе &Model, возвращает Vec<Intent>
5. App — в update(): try_recv() → collect intents → handle_intent(apply + отправка в data layer)
6. TextEdit — единственное исключение из MVI (мутирует напрямую)

## MVVM (для сложных компонентов)
View ↔ ViewModel (состояние+логика) ↔ Model (глобальное)
ViewModel — структура с методами, не зависит от egui.

## Data Layer (два потока)
UI thread (egui) ←→ mpsc ←→ data thread (std::thread::spawn)

Команды (UI→data): EditorCommand { Pipeline, File, Dialog }
События (data→UI): EditorEvent { Pipeline, File, Dialog }

Правила:
- Tokio запрещён — только std::thread::spawn + mpsc
- Все CPU/IO — в data thread
- UI получает события через try_recv() в каждом кадре
- Блокировка кнопок через is_busy
- Coalesce: несколько TextChanged → только последний
- Debounce: TextChanged не чаще 100 мс
- Файловые диалоги (rfd) — синхронные, из UI по запросу data layer

## Общие требования
- Rust 2021, clippy
- Файлы: snake_case.rs, один файл = один модуль
- Комментарии на русском, код/идентификаторы на английском
- thiserror — для ошибок в shared
- Без unwrap() в production (только в тестах)
- В src/ — используй crate::, в bin/ — code_parser::

## Запрещено
- Импорт domain в interfaces или data_layer
- unwrap() в production
- CPU/IO в GUI потоке
- async/await или Tokio
- Циклические зависимости

## Формат ответа
1. [План действий] — 1-2 предложения
2. Архитектурное место — папка и назначение
3. Код — блоками до 20 строк с краткими комментариями на русском
4. Никогда не выводи код без пояснений
