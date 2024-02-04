use bevy::prelude::*;

use crate::game::{CurrentPlayer, Player};

use super::{
    animation::AnimationEndEvent,
    board::{BOARD_HEIGHT, BOARD_WIDTH},
    helpers, ReloadUiEvent, UiAssets,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_labels.run_if(on_event::<ReloadUiEvent>()),
                update_labels,
            ),
        );
    }
}

#[derive(Component)]
pub struct PlayerLabel(pub Player);

pub fn draw_labels(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    current_player: Res<CurrentPlayer>,
) {
    let screen = helpers::get_screen(&mut commands);

    let players = vec![Player::Player1, Player::Player2];
    let mut containers: Vec<Entity> = vec![];

    for player in players {
        let container = commands
            .spawn(NodeBundle {
                style: Style {
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_grow: 1.,
                    ..default()
                },
                ..default()
            })
            .id();

        let value = {
            if player == current_player.0 {
                format!("> {} <", player.to_string())
            } else {
                player.to_string()
            }
        };

        let label = helpers::get_text(&mut commands, ui_assets.as_ref(), &value);

        commands.entity(label).insert(PlayerLabel(player));
        commands.entity(container).push_children(&[label]);

        containers.push(container);
    }

    let space = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                width: Val::Px(BOARD_WIDTH),
                height: Val::Px(BOARD_HEIGHT),
                ..default()
            },
            ..default()
        })
        .id();

    commands
        .entity(screen)
        .push_children(&[containers[0], space, containers[1]]);
}

pub fn update_labels(
    mut animation_end_events: EventReader<AnimationEndEvent>,
    mut text_query: Query<(&mut Text, &PlayerLabel)>,
    current_player: Res<CurrentPlayer>,
) {
    if animation_end_events.read().count() == 0 {
        return;
    }

    for (mut text, player_label) in &mut text_query {
        if player_label.0 == current_player.0 {
            text.sections[0].value = format!("> {} <", player_label.0.to_string());
        } else {
            text.sections[0].value = player_label.0.to_string();
        }
    }
}
