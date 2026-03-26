use bevy::{platform::collections::HashMap, prelude::*};

use crate::TILE_SIZE;

/// Z-уровень визуальных оверлеев зон (над тайлами, под задачами).
const Z_OVERLAY: f32 = 1.0;

// ─── Типы ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Reflect)]
pub enum ZoneType {
    /// Копка/добыча ресурсов.
    #[default]
    Mining,
    /// Строительство.
    Construction,
    /// Склад/хранение предметов.
    Storage,
    /// Запретная зона — болванчики избегают её (TODO: Фаза 3).
    Forbidden,
}

/// Данные зоны для одного тайла.
#[derive(Clone, Copy, Debug)]
pub struct ZoneInfo {
    pub zone_type: ZoneType,
    /// Приоритет: 1 (низкий) … 5 (срочно).
    pub priority: u8,
}

// ─── Ресурсы ─────────────────────────────────────────────────────────────────

/// Карта зон: тайловая позиция → данные зоны.
/// Источник истины для системы назначения задач.
#[derive(Resource, Default)]
pub struct ZoneMap {
    pub tiles: HashMap<IVec2, ZoneInfo>,
}

impl ZoneMap {
    /// Возвращает приоритет зоны в позиции (0 = нет зоны).
    pub fn get_priority(&self, pos: IVec2) -> u8 {
        self.tiles.get(&pos).map(|z| z.priority).unwrap_or(0)
    }
}

/// Настройки рисования зон (выбранный тип + приоритет).
#[derive(Resource)]
pub struct ZoneSettings {
    pub zone_type: ZoneType,
    /// Текущий выбранный приоритет (1–5).
    pub priority: u8,
}

impl Default for ZoneSettings {
    fn default() -> Self {
        Self {
            zone_type: ZoneType::Mining,
            priority: 3,
        }
    }
}

// ─── Компоненты ───────────────────────────────────────────────────────────────

/// Маркер для спрайтов-оверлеев зон.
#[derive(Component)]
pub struct ZoneOverlay;

/// Компонент: отображение текущего приоритета зоны в UI.
#[derive(Component)]
pub struct ZonePriorityUi;

// ─── Цвета ────────────────────────────────────────────────────────────────────

fn zone_color(zone_info: &ZoneInfo) -> Color {
    // Прозрачность растёт с приоритетом: от 0.15 (П1) до 0.50 (П5)
    let alpha = 0.15 + zone_info.priority as f32 * 0.07;
    match zone_info.zone_type {
        ZoneType::Mining => Color::srgba(1.00, 0.65, 0.00, alpha),        // оранжевый
        ZoneType::Construction => Color::srgba(0.30, 0.55, 1.00, alpha),  // синий
        ZoneType::Storage => Color::srgba(0.20, 0.85, 0.30, alpha),       // зелёный
        ZoneType::Forbidden => Color::srgba(1.00, 0.15, 0.15, alpha + 0.10), // красный
    }
}

// ─── Системы ──────────────────────────────────────────────────────────────────

/// Синхронизирует спрайты оверлеев с текущей `ZoneMap`.
/// Перестраивает все оверлеи при изменении карты.
pub fn sync_zone_overlays(
    zone_map: Res<ZoneMap>,
    mut commands: Commands,
    q_overlays: Query<Entity, With<ZoneOverlay>>,
) {
    if !zone_map.is_changed() {
        return;
    }

    // Уничтожаем старые оверлеи
    for entity in &q_overlays {
        commands.entity(entity).despawn();
    }

    // Спавним новые
    for (pos, zone_info) in &zone_map.tiles {
        commands.spawn((
            ZoneOverlay,
            Sprite {
                color: zone_color(zone_info),
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
            Transform::from_xyz(
                pos.x as f32 * TILE_SIZE,
                pos.y as f32 * TILE_SIZE,
                Z_OVERLAY,
            ),
        ));
    }
}

/// Обновляет UI-метку текущего приоритета зоны.
pub fn update_zone_priority_ui(
    zone_settings: Res<ZoneSettings>,
    mut q_label: Query<&mut Text, With<ZonePriorityUi>>,
) {
    if !zone_settings.is_changed() {
        return;
    }
    for mut text in &mut q_label {
        text.0 = zone_settings.priority.to_string();
    }
}
