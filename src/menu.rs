use crate::states;
use crate::ui::UiAssets;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use states::AppState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MenuState>()
            .add_systems(OnEnter(AppState::Menu), start)
            .add_systems(OnExit(AppState::Menu), despawn::<Main>)
            .add_systems(OnExit(MenuState::Start), despawn::<Hint>)
            .add_systems(OnEnter(MenuState::Mode), mode)
            .add_systems(OnExit(MenuState::Mode), despawn::<Main>)
            .add_systems(
                Update,
                (blink, listen)
                    .run_if(in_state(MenuState::Start))
                    .run_if(in_state(AppState::Menu)),
            )
            .add_systems(
                Update,
                (button_system, button_action).run_if(in_state(AppState::Menu)),
            );
    }
}

#[derive(States, SystemSet, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum MenuState {
    #[default]
    Start,
    Mode,
    Profile,
}

#[derive(Component)]
enum ButtonAction {
    Avalanche,
    Capture,
}

#[derive(Component)]
struct Main;

#[derive(Component)]
struct Hint;

#[derive(Component)]
struct Blink;

#[derive(Component)]
struct OnMode;

fn start(mut commands: Commands, ui_assets: Res<UiAssets>) {
    let screen = commands
        .spawn((
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(80.0),
                    ..default()
                },
                ..default()
            },
            Main,
        ))
        .id();

    let header = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(50.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: ui_assets.title.clone().into(),
                ..default()
            });
        })
        .id();

    let hint = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(50.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    ..default()
                },
                ..default()
            },
            Hint,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Press any key",
                        TextStyle {
                            font: ui_assets.font.clone(),
                            font_size: 40.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    ..default()
                },
                Blink,
            ));
        })
        .id();

    commands.entity(screen).push_children(&[header, hint]);
}

fn blink(mut query: Query<&mut Text, With<Blink>>, time: Res<Time>) {
    for mut text in &mut query {
        let alpha = (time.elapsed_seconds() * 2.0).sin() * 0.5 + 0.5;
        text.sections[0].style.color = text.sections[0].style.color.with_a(alpha);
    }
}

fn listen(
    mut key_evr: EventReader<KeyboardInput>,
    mut mouse_evr: EventReader<MouseButtonInput>,
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    if key_evr.read().next().is_some() || mouse_evr.read().next().is_some() {
        menu_state.set(MenuState::Mode);
    }
}

fn mode(mut commands: Commands, ui_materials: Res<UiAssets>, query: Query<Entity, With<Main>>) {
    let screen = query.single();

    let container = commands
        .spawn((NodeBundle {
            style: Style {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(50.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        },))
        .id();

    let game_modes: Vec<(&str, ButtonAction)> = vec![
        ("Avalanche", ButtonAction::Avalanche),
        ("Capture", ButtonAction::Capture),
    ];

    for mode in game_modes {
        let (text, action) = mode;

        let node = commands
            .spawn(NodeBundle {
                style: Style {
                    display: Display::Flex,
                    width: Val::Percent(25.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            })
            .id();

        let button = commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
                },
                action,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        text,
                        TextStyle {
                            font: ui_materials.font.clone(),
                            font_size: 40.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    ..default()
                });
            })
            .id();

        commands.entity(node).add_child(button);
        commands.entity(container).add_child(node);
    }

    commands.entity(screen).push_children(&[container]);
}

fn button_system(
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, children) in &interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();

        text.sections[0].value = match *interaction {
            Interaction::Hovered => format!("> {} <", text.sections[0].value),
            _ => text.sections[0].value.replace("> ", "").replace(" <", ""),
        };
    }
}

fn button_action(
    interaction_query: Query<(&Interaction, &ButtonAction)>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<states::GameMode>>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *action {
            ButtonAction::Avalanche => {
                app_state.set(AppState::Game);
                game_state.set(states::GameMode::Avalanche);
            }
            ButtonAction::Capture => {}
        }
    }
}

fn despawn<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &mut query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
