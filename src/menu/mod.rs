use crate::{
    game::Player,
    profile::Profiles,
    states::{AppState, GameMode},
    ui::UiAssets,
};
use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput},
    prelude::*,
};
use bevy_persistent::Persistent;

const PROFILE_LIMIT: usize = 10;
const PROFILE_SIZE: f32 = 80.;
const PROFILE_SPACING: f32 = 20.;
const PROFILE_CONTAINER_WIDTH: f32 = ((PROFILE_SIZE + PROFILE_SPACING) * 5.) - PROFILE_SPACING;

// (53, 84, 89)
const PRIMARY_COLOR: Color = Color::rgb(0.20784314, 0.32941177, 0.34901962);
// (132, 213, 226)
const ACCENT_COLOR: Color = Color::rgb(0.5176471, 0.8352941, 0.8862745);
const TEXT_COLOR: Color = Color::WHITE;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .init_resource::<Selected>()
            .add_systems(OnEnter(AppState::Menu), setup_start_screen)
            .add_systems(OnExit(AppState::Menu), despawn::<Main>)
            .add_systems(OnExit(MenuState::Start), despawn::<Hint>)
            .add_systems(OnEnter(MenuState::Mode), setup_mode_screen)
            .add_systems(OnExit(MenuState::Mode), despawn::<Mode>)
            .add_systems(OnEnter(MenuState::Profile), setup_profile_screen)
            .add_systems(
                Update,
                (
                    button_action,
                    (blink, listen).run_if(in_state(MenuState::Start)),
                    (selected_changed, spawn_profiles)
                        .run_if(in_state(MenuState::Profile))
                        .after(setup_profile_screen),
                )
                    .run_if(in_state(AppState::Menu)),
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
    SelectMode(GameMode),
    SwapProfiles,
    AddProfile,
    SelectProfile(usize),
    Play,
}

#[derive(Component)]
struct Main;

#[derive(Component)]
struct Hint;

#[derive(Component)]
struct Blink;

#[derive(Component)]
struct Mode;

#[derive(Component)]
struct UiProfileContainer;

#[derive(Component)]
struct UiSelected;

#[derive(Resource)]
pub struct Selected {
    profiles: [usize; 2],
    last_selected: usize,
}

impl Default for Selected {
    fn default() -> Self {
        Self {
            profiles: [0, 1],
            last_selected: 0,
        }
    }
}

impl Selected {
    fn swap(&mut self) {
        self.profiles.swap(0, 1);
    }

    fn select(&mut self, profile_index: usize) {
        self.profiles[self.last_selected] = profile_index;
        self.last_selected = (self.last_selected + 1) % 2;
    }

    fn is_selected(&self, profile_index: usize) -> bool {
        self.profiles.contains(&profile_index)
    }

    pub fn get(&self, player: Player) -> usize {
        self.profiles[player as usize]
    }
}

fn setup_start_screen(mut commands: Commands, ui_assets: Res<UiAssets>) {
    let screen = commands
        .spawn((
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(20.),
                    ..default()
                },
                ..default()
            },
            Main,
        ))
        .id();

    let header = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(50.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexEnd,
                    margin: UiRect {
                        bottom: Val::Px(60.),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            Mode,
        ))
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
                    margin: UiRect {
                        top: Val::Px(60.),
                        ..default()
                    },
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
                        "PRESS ANY BUTTON",
                        TextStyle {
                            font: ui_assets.font.clone(),
                            font_size: 40.0,
                            color: TEXT_COLOR,
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

fn setup_mode_screen(
    mut commands: Commands,
    ui_materials: Res<UiAssets>,
    query: Query<Entity, With<Main>>,
) {
    let screen = query.single();

    let container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    width: Val::Percent(100.0),
                    height: Val::Percent(50.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Row,
                    margin: UiRect {
                        top: Val::Px(60.),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            Mode,
        ))
        .id();

    for mode in GameMode::iter() {
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
                ButtonAction::SelectMode(mode),
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        mode.to_string(),
                        TextStyle {
                            font: ui_materials.font.clone(),
                            font_size: 40.0,
                            color: TEXT_COLOR,
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

fn setup_profile_screen(
    mut commands: Commands,
    query: Query<Entity, With<Main>>,
    ui_materials: Res<UiAssets>,
) {
    let screen = query.single();

    let top_container = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                padding: UiRect::all(Val::Px(10.)),
                width: Val::Px(PROFILE_CONTAINER_WIDTH),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                border: UiRect::all(Val::Px(5.)),
                ..default()
            },
            background_color: PRIMARY_COLOR.into(),
            border_color: ACCENT_COLOR.into(),
            ..default()
        })
        .id();

    let title = commands
        .spawn(TextBundle {
            text: Text::from_section(
                "SELECTED",
                TextStyle {
                    font: ui_materials.font.clone(),
                    font_size: 40.0,
                    color: TEXT_COLOR,
                },
            ),
            ..default()
        })
        .id();

    let selected_profiles: Vec<Entity> = Player::iter()
        .map(|player| {
            let node = commands
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        height: Val::Percent(100.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.),
                        ..default()
                    },
                    ..default()
                })
                .id();

            let heading = commands
                .spawn(TextBundle {
                    text: Text::from_section(
                        player.to_string(),
                        TextStyle {
                            font: ui_materials.font.clone(),
                            font_size: 40.0,
                            color: TEXT_COLOR,
                        },
                    ),
                    ..default()
                })
                .id();

            let image = commands
                .spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(PROFILE_SIZE),
                        aspect_ratio: Some(1.0),
                        ..default()
                    },
                    image: ui_materials.profile.clone().into(),
                    ..default()
                })
                .id();

            let text = commands
                .spawn((
                    TextBundle {
                        text: Text::from_section(
                            player.to_string(),
                            TextStyle {
                                font: ui_materials.font.clone(),
                                font_size: 40.,
                                color: TEXT_COLOR,
                            },
                        ),
                        ..default()
                    },
                    UiSelected,
                ))
                .id();

            commands.entity(node).push_children(&[heading, image, text]);

            node
        })
        .collect();

    let swap = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                width: Val::Px(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
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
                    ButtonAction::SwapProfiles,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "<->",
                            TextStyle {
                                font: ui_materials.font.clone(),
                                font_size: 40.,
                                color: TEXT_COLOR,
                            },
                        ),
                        ..default()
                    });
                });
        })
        .id();

    let profiles_container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    width: Val::Px(PROFILE_CONTAINER_WIDTH),
                    height: Val::Auto,
                    column_gap: Val::Px(PROFILE_SPACING),
                    row_gap: Val::Px(PROFILE_SPACING),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                ..default()
            },
            UiProfileContainer,
        ))
        .id();

    let play = commands
        .spawn((
            ButtonBundle {
                background_color: Color::NONE.into(),
                ..default()
            },
            ButtonAction::Play,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "PLAY",
                TextStyle {
                    font: ui_materials.font.clone(),
                    font_size: 40.0,
                    color: TEXT_COLOR,
                },
            ));
        })
        .id();

    commands.entity(top_container).push_children(&[
        selected_profiles[0],
        swap,
        selected_profiles[1],
    ]);

    commands
        .entity(screen)
        .push_children(&[title, top_container, profiles_container, play]);
}

fn spawn_profiles(
    mut commands: Commands,
    mut profile_container: Query<Entity, With<UiProfileContainer>>,
    mut children_query: Query<&Children>,
    ui_assets: Res<UiAssets>,
    profiles: Res<Persistent<Profiles>>,
    selected: Res<Selected>,
) {
    if !profiles.is_added() && !profiles.is_changed() && !selected.is_changed() {
        return;
    }

    let container = profile_container.single_mut();

    if let Ok(children) = children_query.get_mut(container) {
        for entity in children.iter() {
            commands.entity(*entity).despawn_recursive();
        }
    }

    for (index, profile) in profiles.0.iter().take(PROFILE_LIMIT).enumerate() {
        let (color, background_color) = if selected.is_selected(index) {
            (PRIMARY_COLOR, ACCENT_COLOR.into())
        } else {
            (TEXT_COLOR, PRIMARY_COLOR.into())
        };

        let button = commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        display: Display::Flex,
                        max_width: Val::Px(PROFILE_SIZE),
                        overflow: Overflow::clip(),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color,
                    ..default()
                },
                ButtonAction::SelectProfile(index),
            ))
            .id();

        let image = commands
            .spawn(ImageBundle {
                style: Style {
                    width: Val::Px(PROFILE_SIZE),
                    aspect_ratio: Some(1.0),
                    ..default()
                },
                image: ui_assets.profile.clone().into(),
                ..default()
            })
            .id();

        let text = commands
            .spawn(TextBundle::from_section(
                &profile.name,
                TextStyle {
                    font: ui_assets.font.clone(),
                    font_size: 40.,
                    color,
                },
            ))
            .id();

        commands.entity(button).push_children(&[image, text]);
        commands.entity(container).add_child(button);
    }

    if profiles.0.len() < PROFILE_LIMIT {
        let add_profile = commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        display: Display::Flex,
                        width: Val::Px(PROFILE_SIZE),
                        height: Val::Px(PROFILE_SIZE),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: Color::BLACK.into(),
                    ..default()
                },
                ButtonAction::AddProfile,
            ))
            .with_children(|parent| {
                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(PROFILE_SIZE),
                        aspect_ratio: Some(1.0),
                        ..default()
                    },
                    image: ui_assets.plus.clone().into(),
                    ..default()
                });
            })
            .id();

        commands.entity(container).add_child(add_profile);
    }
}

fn selected_changed(
    mut text_query: Query<&mut Text, With<UiSelected>>,
    profiles: Res<Persistent<Profiles>>,
    selected: Res<Selected>,
) {
    if !selected.is_changed() || text_query.iter().count() != 2 {
        return;
    }

    for (mut text, profile_index) in text_query.iter_mut().zip(selected.profiles.iter()) {
        let name = profiles
            .0
            .get(*profile_index)
            .map_or("Unknown".to_string(), |profile| profile.name.clone());

        text.sections[0].value = name;
    }
}

fn button_action(
    interaction_query: Query<
        (&Children, &Interaction, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut app_state: ResMut<NextState<AppState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameMode>>,
    mut selected: ResMut<Selected>,
) {
    for (children, interaction, action) in &interaction_query {
        match interaction {
            Interaction::Pressed => match *action {
                ButtonAction::SelectMode(mode) => {
                    match mode {
                        GameMode::Avalanche => {
                            game_state.set(GameMode::Avalanche);
                        }
                        GameMode::Capture => {
                            game_state.set(GameMode::Capture);
                        }
                    }
                    menu_state.set(MenuState::Profile);
                }
                ButtonAction::SwapProfiles => {
                    selected.swap();
                }
                ButtonAction::SelectProfile(index) => {
                    selected.select(index);
                }
                ButtonAction::Play => {
                    app_state.set(AppState::Game);
                }
                _ => {}
            },
            Interaction::Hovered => match *action {
                ButtonAction::SelectMode(_) | ButtonAction::Play => {
                    let Ok(mut text) = text_query.get_mut(children[0]) else {
                        return;
                    };
                    text.sections[0].value = format!("> {} <", text.sections[0].value);
                }
                _ => {}
            },
            Interaction::None => match *action {
                ButtonAction::SelectMode(_) | ButtonAction::Play => {
                    let Ok(mut text) = text_query.get_mut(children[0]) else {
                        return;
                    };
                    let value = &text.sections[0].value;
                    if value.starts_with(">") {
                        text.sections[0].value = value[2..value.len() - 2].to_string();
                    }
                }
                _ => {}
            },
        }
    }
}

fn despawn<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &mut query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
