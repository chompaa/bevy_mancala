use crate::game::GameOverEvent;

use super::{helpers, AnimationWaitEvent, UiAssets};

use bevy::prelude::*;

pub fn draw_game_over_screen(
    mut commands: Commands,
    mut animation_wait_events: EventReader<AnimationWaitEvent>,
    mut game_over_events: EventReader<GameOverEvent>,
    ui_assets: Res<UiAssets>,
) {
    if animation_wait_events.read().count() > 0 {
        return;
    }

    for event in game_over_events.read() {
        let screen = helpers::get_screen(&mut commands);

        let value = {
            if let Some(winner) = &event.0 {
                format!("{} wins!", winner.to_string())
            } else {
                "Draw!".to_string()
            }
        };

        let container = commands
            .spawn(NodeBundle {
                style: Style {
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.),
                    flex_grow: 1.,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.6).into(),
                ..default()
            })
            .id();

        let text = commands
            .spawn(TextBundle {
                text: Text::from_section(
                    value,
                    TextStyle {
                        font: ui_assets.font.clone(),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            })
            .id();

        commands.entity(container).add_child(text);
        commands.entity(screen).add_child(container);
    }
}
