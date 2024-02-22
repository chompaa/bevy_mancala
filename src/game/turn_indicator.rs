use super::{
    animation::AnimationState,
    board::{BOARD_HEIGHT, BOARD_WIDTH},
    helpers,
};
use crate::{
    game::{CurrentPlayer, Player},
    menu::Selected,
    profile::Profiles,
    states::AppState,
    ui::{ReloadUiEvent, UiAssets},
};
use bevy::prelude::*;
use bevy_persistent::Persistent;

pub struct TurnIndicatorPlugin;

impl Plugin for TurnIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_labels.run_if(on_event::<ReloadUiEvent>()),
                update_labels.run_if(in_state(AppState::Game)),
            ),
        )
        .add_systems(
            OnExit(AppState::Game),
            helpers::despawn::<TurnIndicatorScreen>,
        );
    }
}

#[derive(Component)]
struct TurnIndicatorScreen;

#[derive(Component)]
pub struct TurnIndicatorLabel {
    pub name: String,
    pub player: Player,
}

pub fn draw_labels(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    current_player: Res<CurrentPlayer>,
    selected: Res<Selected>,
    profiles: Res<Persistent<Profiles>>,
) {
    let screen = helpers::get_screen(&mut commands);

    commands.entity(screen).insert(TurnIndicatorScreen);

    let containers: Vec<Entity> = Player::iter()
        .map(|player| {
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

            // TODO: better way of handling this?
            let name = profiles.0[selected.get(player)].name.clone();

            let value = {
                if player == current_player.0 {
                    format!("> {} <", name)
                } else {
                    name.clone()
                }
            };

            let label = helpers::get_text(&mut commands, ui_assets.as_ref(), &value);

            commands.entity(label).insert(TurnIndicatorLabel {
                name: name.clone(),
                player,
            });
            commands.entity(container).push_children(&[label]);

            container
        })
        .collect();

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
    mut text_query: Query<(&mut Text, &TurnIndicatorLabel)>,
    current_player: Res<CurrentPlayer>,
    animation_state: Res<State<AnimationState>>,
) {
    if !animation_state.is_changed() || *animation_state.get() == AnimationState::Animating {
        return;
    }

    for (mut text, label) in &mut text_query {
        if label.player == current_player.0 {
            text.sections[0].value = format!("> {} <", label.name.clone());
        } else {
            text.sections[0].value = label.name.clone();
        }
    }
}
