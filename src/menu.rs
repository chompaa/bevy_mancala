use crate::GameState;
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup)
            .add_systems(OnExit(GameState::Menu), despawn::<OnMenu>)
            .add_systems(Update, (button_system, button_action));
    }
}

const NORMAL_BUTTON: Color = Color::rgb(1.0, 1.0, 1.0);
const HOVERED_BUTTON: Color = Color::rgb(0.9, 0.9, 0.9);
const PRESSED_BUTTON: Color = Color::rgb(0.3, 0.3, 0.3);
const NORMAL_TEXT: Color = Color::rgb(0.3, 0.3, 0.3);
const PRESSED_TEXT: Color = Color::rgb(1.0, 1.0, 1.0);

#[derive(Component)]
enum ButtonAction {
    Play,
}

#[derive(Component)]
struct OnMenu;

fn setup(mut commands: Commands) {
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Auto),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let text_style = TextStyle {
        font_size: 40.0,
        color: NORMAL_TEXT,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        ..default()
                    },
                    ButtonAction::Play,
                    OnMenu,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("PLAY", text_style.clone()));
                });
        });
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>, With<OnMenu>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();

        let (button_color, text_color): (BackgroundColor, Color) = {
            match *interaction {
                Interaction::Pressed => (PRESSED_BUTTON.into(), PRESSED_TEXT),
                Interaction::Hovered => (HOVERED_BUTTON.into(), NORMAL_TEXT),
                _ => (NORMAL_BUTTON.into(), NORMAL_TEXT),
            }
        };

        *color = button_color;
        text.sections[0].style.color = text_color;
    }
}

fn button_action(
    interaction_query: Query<(&Interaction, &ButtonAction)>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *action {
            ButtonAction::Play => game_state.set(GameState::Game),
        }
    }
}

fn despawn<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &mut query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
