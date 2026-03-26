use bevy::prelude::*;

use crate::{
    ActionKind, Dweller, DwellersSelected, TaskKind, TaskNeeds, UiButton,
    actions::CurrentAction,
    data::BUILD_RECIPES,
    extract_ok,
    locale::Locale,
    tasks::BuildResult,
    zones::{ZoneSettings, ZoneType, ZonePriorityUi},
};

#[derive(Component)]
pub struct DwellersSelectedUi;

#[derive(Component)]
pub struct CoordinatesUi;

pub fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>, locale: Res<Locale>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                row_gap: Val::Px(10.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|c| {
            // Информация о выбранных болванчиках
            c.spawn((
                DwellersSelectedUi,
                Text::new(""),
                BackgroundColor(Color::BLACK.with_alpha(0.5)),
            ));

            // Координаты курсора
            c.spawn((
                CoordinatesUi,
                Text::new(""),
                BackgroundColor(Color::BLACK.with_alpha(0.5)),
            ));

            // ── Ряд 1: Кнопки строительства ─────────────────────────────────
            c.spawn(Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|c| {
                for (result, cost) in BUILD_RECIPES {
                    let label = build_result_label(result, &locale);
                    c.spawn(UiButton)
                        .with_child(Text::new(label))
                        .with_child(ImageNode::new(asset_server.load(result.sprite_path())))
                        .observe(get_observer_action_button(ActionKind::TaskWithNeeds(
                            TaskKind::Build { result: *result },
                            TaskNeeds::Objects(cost.to_vec()),
                        )));
                }
            });

            // ── Ряд 2: Кнопки задач ─────────────────────────────────────────
            c.spawn(Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|c| {
                for task_kind in [
                    TaskKind::Dig,
                    TaskKind::Harvest,
                    TaskKind::Flood,
                    TaskKind::Scoop,
                    TaskKind::Attack,
                    TaskKind::Fish,
                    TaskKind::Pickup,
                    TaskKind::Stockpile,
                    TaskKind::Smoothen,
                    TaskKind::Walk,
                ] {
                    let label = locale.t(&format!("task.{}", task_kind.id()));
                    c.spawn(UiButton)
                        .with_child(Text::new(label))
                        .with_child(ImageNode::new(asset_server.load(task_kind.sprite_path())))
                        .observe(get_observer_action_button(ActionKind::Task(task_kind)));
                }

                c.spawn(UiButton)
                    .with_child(Text::new(locale.t("ui.cancel")))
                    .observe(get_observer_action_button(ActionKind::Cancel));
            });

            // ── Ряд 3: Зоны ─────────────────────────────────────────────────
            c.spawn(Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|c| {
                // Метка + текущий приоритет
                c.spawn((
                    Text::new(format!("{}: ", locale.t("zone.priority"))),
                    BackgroundColor(Color::BLACK.with_alpha(0.4)),
                ));
                c.spawn((
                    ZonePriorityUi,
                    Text::new("3"),
                    BackgroundColor(Color::BLACK.with_alpha(0.4)),
                ));

                // Кнопки приоритета П1…П5
                for priority in 1u8..=5 {
                    c.spawn(UiButton)
                        .with_child(Text::new(format!("П{priority}")))
                        .observe(
                            move |_: On<Pointer<Click>>, mut zone_settings: ResMut<ZoneSettings>| {
                                zone_settings.priority = priority;
                            },
                        );
                }

                // Разделитель
                c.spawn(Text::new(" | "));

                // Кнопки типов зон
                for (zone_type, key) in [
                    (ZoneType::Mining, "zone.mining"),
                    (ZoneType::Construction, "zone.construction"),
                    (ZoneType::Storage, "zone.storage"),
                    (ZoneType::Forbidden, "zone.forbidden"),
                ] {
                    let label = locale.t(key);
                    c.spawn(UiButton)
                        .with_child(Text::new(label))
                        .observe(
                            move |_: On<Pointer<Click>>,
                                  mut commands: Commands,
                                  zone_settings: Res<ZoneSettings>,
                                  mut q_borders: Query<
                                &mut BorderColor,
                                With<UiButton>,
                            >| {
                                commands.insert_resource(CurrentAction::new(
                                    ActionKind::DrawZone(zone_type),
                                ));
                                // Сбрасываем все рамки
                                for mut border in &mut q_borders {
                                    *border = Color::BLACK.into();
                                }
                                let _ = zone_settings; // используется для чтения типа
                            },
                        );
                }

                // Кнопка очистки зоны
                c.spawn(UiButton)
                    .with_child(Text::new(locale.t("zone.clear")))
                    .observe(get_observer_action_button(ActionKind::ClearZone));
            });
        });
}

fn build_result_label(result: &BuildResult, locale: &Locale) -> String {
    let key = match result {
        BuildResult::Object(obj) => format!("object.{obj:?}"),
        BuildResult::Tile(tile) => format!("tile.{tile:?}"),
    };
    locale.t(&key)
}

pub fn get_observer_action_button(
    action: ActionKind,
) -> impl FnMut(On<Pointer<Click>>, Commands, Res<CurrentAction>, Query<&mut BorderColor, With<UiButton>>)
{
    move |pointer_click: _, mut commands: _, current_action: _, mut q_borders: _| {
        if current_action.kind == action {
            commands.insert_resource(CurrentAction::default());
        } else {
            commands.insert_resource(CurrentAction::new(action.clone()));
        }

        info!("Current action: {:?}", current_action.kind);

        for mut border in &mut q_borders {
            *border = Color::BLACK.into();
        }

        if let Ok(mut border) = q_borders.get_mut(pointer_click.entity) {
            *border = bevy::color::palettes::css::RED.into();
        }
    }
}

pub fn update_dwellers_selected(
    dwellers_selected: Res<DwellersSelected>,
    q_dwellers: Query<&Dweller>,
    mut q_dwellers_selected_ui: Query<&mut Text, With<DwellersSelectedUi>>,
) {
    if dwellers_selected.is_changed() {
        let mut dwellers_selected_ui = extract_ok!(q_dwellers_selected_ui.single_mut());

        dwellers_selected_ui.0 = dwellers_selected
            .list()
            .iter()
            .filter_map(|e| q_dwellers.get(*e).ok().map(|d| d.name.clone()))
            .collect::<Vec<String>>()
            .join(", ");
    }
}
