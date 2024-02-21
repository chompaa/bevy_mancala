use bevy::{prelude::*, transform::helper};

use crate::ui::UiAssets;

use super::{animation::bezier_blend, helpers, GameState, Winner};

pub struct GameOverPlugin;

const ALPHA_END: f32 = 0.7;
const ALPHA_SPEED: f32 = 5.0;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameOverState>()
            .init_resource::<GameOverAlpha>()
            .add_systems(OnEnter(GameState::Over), setup)
            .add_systems(OnExit(GameState::Over), despawn::<GameOverScreen>)
            .add_systems(
                Update,
                show.run_if(in_state(GameState::Over))
                    .run_if(in_state(GameOverState::Hidden)),
            );
    }
}

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
struct GameOverContainer;

#[derive(Component)]
struct GameOverText;

#[derive(Resource, Default)]
struct GameOverAlpha {
    value: f32,
    elapsed: f32,
}

#[derive(States, SystemSet, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum GameOverState {
    #[default]
    Hidden,
    Visible,
}

fn setup(
    mut commands: Commands,
    winner: Res<Winner>,
    ui_assets: Res<UiAssets>,
    mut alpha: ResMut<GameOverAlpha>,
    mut state: ResMut<NextState<GameOverState>>,
) {
    let screen = helpers::get_screen(&mut commands);

    commands.entity(screen).insert(GameOverScreen);

    let value = winner.0.as_ref().map_or_else(
        || "Draw!".to_string(),
        |player| format!("{} wins!", player.to_string()),
    );

    let container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.),
                    flex_grow: 1.,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.0).into(),
                ..default()
            },
            GameOverContainer,
        ))
        .id();

    let text = commands
        .spawn((
            TextBundle {
                text: Text::from_section(
                    value,
                    TextStyle {
                        font: ui_assets.font.clone(),
                        font_size: 40.0,
                        color: Color::WHITE.with_a(0.),
                    },
                ),
                ..default()
            },
            GameOverText,
        ))
        .id();

    commands.entity(container).add_child(text);
    commands.entity(screen).add_child(container);

    alpha.value = 0.;
    alpha.elapsed = 0.;

    state.set(GameOverState::Hidden);
}

fn show(
    mut text_query: Query<&mut Text, With<GameOverText>>,
    mut background_color_query: Query<&mut BackgroundColor, With<GameOverContainer>>,
    time: Res<Time>,
    mut alpha: ResMut<GameOverAlpha>,
    mut state: ResMut<NextState<GameOverState>>,
) {
    if alpha.value >= ALPHA_END {
        let mut text = text_query.single_mut();

        text.sections[0].style.color = text.sections[0].style.color.with_a(1.);

        state.set(GameOverState::Visible);
    } else {
        alpha.elapsed += time.delta_seconds();
        alpha.value = ALPHA_SPEED * 0.5 * bezier_blend(alpha.elapsed);

        let mut color = background_color_query.single_mut();

        color.0 = color.0.with_a(alpha.value);
    }
}

fn despawn<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
