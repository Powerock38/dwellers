use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{
    data::ObjectId,
    production::{EventLog, ResourceInventory, ResourceLimits, ResourceLimit},
};

// ─── Константы ────────────────────────────────────────────────────────────────

/// Все ресурсы, для которых можно задать лимит (в порядке отображения).
const LIMITABLE_RESOURCES: &[ObjectId] = &[
    ObjectId::Wood,
    ObjectId::Rock,
    ObjectId::Fish,
    ObjectId::Wheat,
    ObjectId::Berries,
    ObjectId::Honeycomb,
    ObjectId::CopperOre,
    ObjectId::Hide,
    ObjectId::Seeds,
    ObjectId::Bread,
    ObjectId::CopperIngot,
    ObjectId::Sword,
    ObjectId::Armor,
    ObjectId::Hydromel,
];

// ─── Системы ──────────────────────────────────────────────────────────────────

/// Показывает панель лимитов ресурсов (клавиша L) через egui.
pub fn show_limits_panel(
    mut contexts: EguiContexts,
    mut limits: ResMut<ResourceLimits>,
    inventory: Res<ResourceInventory>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        limits.panel_visible = !limits.panel_visible;
    }

    if !limits.panel_visible {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::Window::new("Лимиты ресурсов")
        .resizable(true)
        .min_width(420.0)
        .show(ctx, |ui| {
            egui::Grid::new("limits_grid")
                .num_columns(7)
                .striped(true)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    // Header
                    ui.strong("Ресурс");
                    ui.strong("Текущий");
                    ui.strong("Мин");
                    ui.strong("Макс");
                    ui.strong("П");
                    ui.strong("Вкл");
                    ui.strong("");
                    ui.end_row();

                    let mut to_remove: Option<usize> = None;

                    for (idx, limit) in limits.limits.iter_mut().enumerate() {
                        let current = inventory.get(limit.resource);
                        let low = current < limit.min_stock;

                        // Resource name
                        let name = format!("{:?}", limit.resource);
                        if low {
                            ui.colored_label(egui::Color32::from_rgb(220, 80, 80), &name);
                        } else {
                            ui.label(&name);
                        }

                        // Current stock
                        ui.label(current.to_string());

                        // Min DragValue
                        ui.add(
                            egui::DragValue::new(&mut limit.min_stock)
                                .range(0..=9999)
                                .speed(1.0),
                        );

                        // Max DragValue
                        ui.add(
                            egui::DragValue::new(&mut limit.max_stock)
                                .range(0..=9999)
                                .speed(1.0),
                        );

                        // Priority (1–5)
                        ui.add(
                            egui::DragValue::new(&mut limit.priority)
                                .range(1..=5)
                                .speed(1.0),
                        );

                        // Enabled checkbox
                        ui.checkbox(&mut limit.enabled, "");

                        // Remove button
                        if ui.small_button("✕").clicked() {
                            to_remove = Some(idx);
                        }

                        ui.end_row();
                    }

                    if let Some(idx) = to_remove {
                        limits.limits.remove(idx);
                    }
                });

            ui.separator();

            // Add new limit form
            ui.horizontal(|ui| {
                ui.label("Добавить:");

                egui::ComboBox::from_id_salt("new_resource_combo")
                    .selected_text(format!("{:?}", LIMITABLE_RESOURCES[limits.new_resource_idx]))
                    .show_ui(ui, |ui| {
                        for (i, res) in LIMITABLE_RESOURCES.iter().enumerate() {
                            ui.selectable_value(
                                &mut limits.new_resource_idx,
                                i,
                                format!("{res:?}"),
                            );
                        }
                    });

                ui.label("мин");
                ui.add(
                    egui::DragValue::new(&mut limits.new_min)
                        .range(0..=9999)
                        .speed(1.0),
                );

                ui.label("макс");
                ui.add(
                    egui::DragValue::new(&mut limits.new_max)
                        .range(0..=9999)
                        .speed(1.0),
                );

                if ui.button("Добавить").clicked() {
                    let resource = LIMITABLE_RESOURCES[limits.new_resource_idx];
                    // Don't add duplicates
                    if !limits.limits.iter().any(|l| l.resource == resource) {
                        limits.limits.push(ResourceLimit::new(
                            resource,
                            limits.new_min,
                            limits.new_max,
                        ));
                    }
                }
            });
        });
}

/// Показывает лог событий планировщика (клавиша J) через egui.
pub fn show_event_log(
    mut contexts: EguiContexts,
    mut event_log: ResMut<EventLog>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyJ) {
        event_log.visible = !event_log.visible;
    }

    if !event_log.visible {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::Window::new("Лог событий")
        .resizable(true)
        .min_width(320.0)
        .max_height(300.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in &event_log.entries {
                        let mins = (entry.time_secs / 60.0) as u32;
                        let secs = (entry.time_secs as u32) % 60;
                        ui.label(format!("[{mins:02}:{secs:02}] {}", entry.message));
                    }
                });

            ui.separator();
            if ui.button("Очистить").clicked() {
                event_log.entries.clear();
            }
        });
}
