use bevy::prelude::*;

mod actions_ui;
pub use actions_ui::*;
mod workstation_ui;
pub use workstation_ui::*;

pub fn init_font(asset_server: Res<AssetServer>, mut query: Query<&mut TextFont, Added<TextFont>>) {
    for mut font in &mut query {
        font.font = asset_server.load("alagard.ttf");
    }
}

#[derive(Component)]
#[require(
    Button,
    Node(|| Node {
        padding: UiRect::all(Val::Px(5.0)),
        border: UiRect::all(Val::Px(4.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: Val::Px(5.0),
        ..default()
    }),
    BorderColor(|| BorderColor(Color::BLACK)),
    BackgroundColor(|| BackgroundColor(BG_PRIMARY))
)]
pub struct UiButton;

pub const BG_PRIMARY: Color = Color::srgb(0.15, 0.15, 0.15);
pub const BG_SECONDARY: Color = Color::srgb(0.25, 0.25, 0.25);
pub const BG_TERTIARY: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
#[require(
    Node(|| Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(10.)),
        ..default()
    }),
    BackgroundColor(|| Color::srgba(0.1, 0.1, 0.1, 0.5))
)]
pub struct UiWindow;

pub fn update_ui_buttons(
    mut interaction_query: Query<
        (&Interaction, Ref<Interaction>, &mut BackgroundColor),
        With<UiButton>,
    >,
) {
    for (interaction, interaction_changed, mut color) in &mut interaction_query {
        if interaction_changed.is_changed() {
            match *interaction {
                Interaction::Pressed => {
                    *color = BG_TERTIARY.into();
                }
                Interaction::Hovered => {
                    *color = BG_SECONDARY.into();
                    continue;
                }
                Interaction::None => {
                    *color = BG_PRIMARY.into();
                }
            }
        }

        if color.0 != BG_SECONDARY {
            *color = BG_PRIMARY.into();
        }
    }
}
