use super::UiAssets;
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct PressableButton;

pub fn get_button(commands: &mut Commands, width: f32, height: f32) -> Entity {
    commands
        .spawn((
            PressableButton,
            ButtonBundle {
                style: Style {
                    width: Val::Px(width),
                    height: Val::Px(height),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
        ))
        .id()
}

pub fn get_node(commands: &mut Commands, width: f32, height: f32) -> Entity {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(width),
                height: Val::Px(height),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .id()
}

pub fn get_label<T: Component>(
    commands: &mut Commands,
    tag: T,
    assets: &UiAssets,
    size: f32,
    value: &str,
) -> Entity {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        value,
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 40.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    ..default()
                },
                tag,
            ));
        })
        .id()
}

pub fn random_point_in_circle(radius: Vec2) -> Vec2 {
    let mut rng = rand::thread_rng();

    let theta = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
    let rx = rng.gen_range(0.0..(radius.x));
    let ry = rng.gen_range(0.0..(radius.y));

    Vec2::new(rx * theta.cos(), ry * theta.sin())
}
