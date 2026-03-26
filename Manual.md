# Dwellers-AI — Руководство разработчика

> Живой документ. Обновляй при каждом значимом изменении архитектуры.

---

## Фаза 0 — Ориентация в коде

### Структура проекта

```
dwellers-AI/
├── Cargo.toml               # bevy 0.18, rand, uuid, pathfinding, dashmap, noise, bitcode
├── assets/
│   ├── alagard.ttf          # единственный шрифт
│   ├── tasks/               # спрайты задач (dig.png, harvest.png и т.д.)
│   ├── tiles/
│   │   ├── floors/          # спрайты полов
│   │   ├── walls/           # спрайты стен
│   │   └── objects/         # спрайты предметов
│   ├── sprites/             # спрайты болванчиков и мобов
│   ├── saves/               # директория сохранений (создаётся при сохранении)
│   └── locale/              # ← НОВОЕ: файлы локализации
│       ├── ru.ron           # русские строки
│       └── en.ron           # английские строки (запасной)
└── src/
    ├── main.rs              # точка входа, регистрация систем
    ├── actions.rs           # обработка действий пользователя (ActionKind)
    ├── camera.rs            # управление камерой
    ├── dwellers.rs          # ИИ болванчиков, движение, назначение задач
    ├── dwellers_needs.rs    # потребности (еда, сон, здоровье)
    ├── locale.rs            # ← НОВОЕ: система локализации
    ├── mobs.rs              # мобы и враждебное поведение
    ├── random_text.rs       # процедурная генерация имён
    ├── save_load.rs         # сохранение/загрузка игры
    ├── state.rs             # GameState: Running / Paused
    ├── tasks.rs             # ядро системы задач (996 строк — главный файл)
    ├── utils.rs             # pascal_case_to_title_case, transform_to_pos
    ├── data/
    │   ├── mod.rs           # ObjectId, TileId, MobId, BUILD_RECIPES, WORKSTATIONS
    │   ├── macros.rs        # enum_map!, structure_ascii!
    │   └── structures.rs    # описания структур мира
    ├── sprites/             # анимации, загрузка спрайтов
    ├── tilemap/             # тайлкарта, чанки, терейн, погода
    └── ui/
        ├── mod.rs           # UiButton, UiWindow, init_font, update_ui_buttons
        ├── actions_ui.rs    # кнопки задач и строительства (главный UI)
        ├── cheats_ui.rs     # дебаг-панель (клавиша C)
        ├── save_load_ui.rs  # сохранение/загрузка (клавиша M)
        └── workstation_ui.rs # UI верстака (клик по верстаку)
```

---

## Три ключевые точки

### 1. Задачи болванчиков — `src/tasks.rs:20`

```rust
pub enum TaskKind {
    Dig, Smoothen, Harvest, Pickup, Attack, Fish, Stockpile,
    Build { result: BuildResult },
    Workstation { amount: u32 },
    Walk, Eat, Sleep, Flood, Scoop,
}
```

**Полезные методы:**
- `task_kind.id()` → `"dig"`, `"build"`, `"harvest"` и т.д. (lowercase snake)
- `task_kind.priority()` → `-1..=2` (Attack=2, Eat/Sleep=1, прочее=0, Stockpile/Walk=-1)
- `task_kind.sprite_path()` → `"tasks/dig.png"`

**Структура задачи** (`tasks.rs:197`):
```rust
pub struct Task {
    pub kind: TaskKind,
    pub pos: IVec2,           // позиция на тайлкарте
    pub dweller_id: Option<Uuid>, // назначенный болванчик
    pub reachable_positions: Vec<IVec2>,
    pub reachable_pathfinding: bool,
    // ...
}
```

---

### 2. Выбор задачи болванчиком — `src/dwellers.rs:346`

Функция `assign_tasks_to_dwellers()` — **главный ИИ файл**. Запускается каждые 200 мс.

**Алгоритм:**
1. Собирает всех незанятых болванчиков и все неназначенные задачи
2. Строит `BinaryHeap` пар (болванчик, задача) с весом `(приоритет, -расстояние)`
3. Извлекает пары по убыванию веса
4. Для каждой пары проверяет `dweller.can_do(task_kind, task_needs)`
5. Запускает A* pathfinding через `task.pathfind(dweller_pos, &tilemap_data)`
6. При успехе — назначает задачу

**Вспомогательная логика** (`dwellers.rs:272`):
- `update_dwellers()` — движение по очереди, выполнение задачи в точке, реакция на врагов

---

### 3. Ресурсы и предметы — `src/data/mod.rs`

#### ObjectId (38 вариантов)
| ID | Ключ спрайта | Свойства |
|----|-------------|----------|
| Wood | "wood" | переносимый |
| Rock | "rock" | переносимый |
| CopperOre | "copper_ore" | переносимый |
| Tree | "tree" | непроходимый, непереносимый |
| Furnace | "furnace" | непроходимый, верстак |
| Sword | "sword" | инструмент (урон=2) |
| Armor | "armor" | броня (хп=3) |
| ... | | |

#### TileId (13 вариантов)
| ID | Ключ спрайта | Тип |
|----|-------------|-----|
| GrassFloor | "grass" | пол |
| StoneWall | "stone" | стена |
| Water | "water" | стена (прозрачная) |
| Lava | "lava" | стена (прозрачная) |
| WoodWall | "wood" | стена |
| ... | | |

#### Верстаки (WORKSTATIONS)
| Верстак | Вход | Выход |
|---------|------|-------|
| Furnace | Wheat + Wood | Bread |
| Forge | CopperOre×2 | CopperIngot |
| Grindstone | CopperIngot×2 | Sword |
| Anvil | CopperIngot×3 | Armor |
| MeadVat | Honeycomb + WaterBucket | Hydromel |

#### Рецепты строительства (BUILD_RECIPES, 18 рецептов)
Хранятся как `&[(BuildResult, &[ObjectId])]` в `data/mod.rs:82`.

---

## Система локализации (Фаза 1)

### Архитектура

Файлы строк хранятся в `assets/locale/`. Формат: `ключ = значение` (по одной строке).

```
# assets/locale/ru.ron
task.dig = Копать
object.Wood = Дерево
```

Ресурс `Locale` загружается синхронно при старте через `std::fs::read_to_string`.

### Использование в системах

```rust
// В startup-системе:
pub fn spawn_ui(locale: Res<Locale>, ...) {
    Text::new(t!(locale, "ui.cancel"))
    // или: locale.t("ui.cancel")
}
```

### Схема ключей

| Префикс | Источник ключа | Пример |
|---------|---------------|--------|
| `task.*` | `task_kind.id()` | `task.dig` |
| `object.*` | `format!("{obj:?}")` | `object.Wood` |
| `tile.*` | `format!("{tile:?}")` | `tile.WoodWall` |
| `mob.*` | `format!("{mob:?}")` | `mob.Sheep` |
| `ui.*` | вручную | `ui.cancel` |

---

## Система игровых тиков

| Интервал | Системы |
|---------|---------|
| каждый кадр | движение болванчиков/мобов, анимации, камера, UI |
| 200 мс | `update_dwellers`, `update_mobs`, **`assign_tasks_to_dwellers`** |
| 600 мс | `update_dweller_needs` (еда/сон/здоровье) |
| 800 мс | `update_terrain` (генерация тайлов в чанках) |
| 1000 мс | `dwellers_load_chunks`, `update_pickups`, `update_hostile_mobs` |
| 5000 мс | `update_unreachable_pathfinding_tasks` |

---

## Управление

| Клавиша | Действие |
|---------|---------|
| Пробел | Пауза/продолжение |
| M | Меню сохранения/загрузки |
| C | Дебаг-панель (спавн объектов/мобов) |
| F | Фокус на случайном болванчике |
| ЛКМ/ПКМ | Выделение территории для задачи |

---

## Дорожная карта

### Фаза 0 ✅ — Ориентация
- Изучена структура проекта
- Найдены три ключевые точки: задачи, ИИ, ресурсы

### Фаза 1 ✅ — Русификация
- Создана инфраструктура локализации (`src/locale.rs`)
- Строки вынесены в `assets/locale/ru.ron`
- UI переведён на русский

### Фаза 2 ✅ — Система приоритетов

#### Зоны (`src/zones.rs`)

```rust
pub enum ZoneType { Mining, Construction, Storage, Forbidden }

pub struct ZoneInfo {
    pub zone_type: ZoneType,
    pub priority: u8,  // 1–5
}

pub struct ZoneMap {
    pub tiles: HashMap<IVec2, ZoneInfo>,  // тайловая позиция → зона
}
```

**Ресурсы:**
- `ZoneMap` — источник истины; содержит приоритет каждого тайла
- `ZoneSettings` — текущий тип и приоритет для рисования (дефолт: Mining, П3)

**Визуализация:** `sync_zone_overlays()` — создаёт полупрозрачные спрайты для каждого тайла. Цвет: оранжевый (Добыча), синий (Стройка), зелёный (Склад), красный (Запрет). Прозрачность растёт с приоритетом (П1=0.22α, П5=0.50α).

#### Utility Score (`src/dwellers.rs`)

Вместо `BinaryHeap<(priority, -distance)>`:

```rust
fn score_task(task_pos, dweller_pos, zone_priority, task_base_priority) -> f32 {
    zone_priority as f32 * 2.0        // зональный приоритет (0..10)
    + (task_base_priority + 2) as f32  // тип задачи (1..4)
    + 1.0 / distance.max(1.0)         // близость (0..1)
    + 0.5                             // skill placeholder (Фаза 3)
}
```

Зона П5 (score +10) beats расстояние (max +1) → болванчик бросает всё и идёт в срочную зону.

#### UI зон (Ряд 3 в нижней панели)
```
[Приоритет: 3]  [П1][П2][П3][П4][П5] | [Добыча][Стройка][Склад][Запрет][Очистить зону]
```

**ActionKind добавлены:**
- `DrawZone(ZoneType)` — рисует зону с приоритетом из `ZoneSettings`
- `ClearZone` — стирает зону с тайлов

#### Файлы изменены
| Файл | Изменения |
|------|-----------|
| `src/zones.rs` | Новый файл: ZoneType, ZoneInfo, ZoneMap, ZoneSettings, sync_zone_overlays |
| `src/actions.rs` | DrawZone/ClearZone в ActionKind, обработка в terrain_pointer_up |
| `src/dwellers.rs` | score_task(), assign_tasks заменён на sort-by-score |
| `src/ui/actions_ui.rs` | Ряд зон: приоритет П1-П5 + кнопки типов |
| `src/main.rs` | Регистрация зон, ресурсов, систем |
| `assets/locale/*.ron` | zone.* строки |

#### TODO (Фаза 3)
- `skill_score` сейчас = 0.5 (заглушка). Реализовать `Dweller.skills` как `HashMap<TaskKind, u8>`
- `ZoneType::Forbidden`: болванчики обходят тайлы с этим тегом (изменить pathfinding cost)
- Сохранение зон (добавить в save_load.rs сериализацию ZoneMap)

### Фаза 3 — Глубина (запланировано)
- Дерево технологий
- Профессии и навыки болванчиков
- Биомы, сезоны и голод

### Фаза 3 — Глубина (запланировано)
- Дерево технологий
- Биомы
- Сезоны и голод

---

## Заметки по разработке

- **Pathfinding** запускается синхронно в `assign_tasks_to_dwellers`. При большом числе задач может стать узким местом. Вынос в `AsyncComputeTaskPool` — первое, что нужно сделать при оптимизации.
- **Чанки** размером 16×16 тайлов загружаются/выгружаются динамически. `TilemapData` — это `DashMap` чанков в памяти.
- **Сохранения** используют `bitcode` (бинарный формат), хранятся в `assets/saves/<имя>/`.
- `enum_map!` — самописный макрос в `data/macros.rs`. Генерирует enum + `data()` метод + `ALL` константу.
