use super::{animation::bezier_blend, helpers, GameState, Winner};
use crate::{states::AppState, ui::UiAssets};
use bevy::prelude::*;

pub struct GameOverPlugin;

const ALPHA_END: f32 = 0.7;
const ALPHA_SPEED: f32 = 5.0;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameOverState>()
            .init_resource::<GameOverAlpha>()
            .add_systems(OnEnter(GameState::Over), setup)
            .add_systems(OnExit(GameState::Over), helpers::despawn::<GameOverScreen>)
            .add_systems(
                Update,
                (fade.run_if(in_state(GameOverState::Hidden)), button_action)
                    .run_if(in_state(GameState::Over)),
            )
            .add_systems(OnEnter(GameOverState::Visible), show);
    }
}

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
struct GameOverContainer;

#[derive(Component)]
struct GameOverElement;

#[derive(Component)]
enum GameOverButtonAction {
    Menu,
}

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
        |&player| format!("{} WINS!", player.to_string()),
    );

    let container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(20.),
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
                        color: Color::WHITE,
                    },
                ),
                visibility: Visibility::Hidden,
                ..default()
            },
            GameOverElement,
        ))
        .id();

    let button = commands
        .spawn((
            ButtonBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                visibility: Visibility::Hidden,
                background_color: Color::NONE.into(),
                ..default()
            },
            GameOverElement,
            GameOverButtonAction::Menu,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "PLAY AGAIN",
                TextStyle {
                    font: ui_assets.font.clone(),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            ));
        })
        .id();

    commands.entity(container).push_children(&[text, button]);
    commands.entity(screen).add_child(container);

    alpha.value = 0.;
    alpha.elapsed = 0.;

    state.set(GameOverState::Hidden);
}

fn fade(
    mut background_color_query: Query<&mut BackgroundColor, With<GameOverContainer>>,
    time: Res<Time>,
    mut alpha: ResMut<GameOverAlpha>,
    mut state: ResMut<NextState<GameOverState>>,
) {
    if alpha.value >= ALPHA_END {
        state.set(GameOverState::Visible);
        return;
    }

    alpha.elapsed += time.delta_seconds();
    alpha.value = ALPHA_SPEED * 0.5 * bezier_blend(alpha.elapsed);

    let mut color = background_color_query.single_mut();

    color.0 = color.0.with_a(alpha.value);
}

fn show(mut visibility_query: Query<&mut Visibility, With<GameOverElement>>) {
    for mut visibility in visibility_query.iter_mut() {
        *visibility = Visibility::Visible;
    }
}

fn button_action(
    interaction_query: Query<
        (&Interaction, &GameOverButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, action) in &interaction_query {
        match interaction {
            Interaction::Pressed => match *action {
                GameOverButtonAction::Menu => {
                    app_state.set(AppState::Menu);
                    game_state.set(GameState::None);
                }
            },
            _ => {}
        }
    }
}
