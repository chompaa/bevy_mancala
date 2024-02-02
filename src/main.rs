use bevy::prelude::*;

mod game;
mod menu;
mod ui;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Game,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_state::<GameState>()
        .add_systems(Startup, setup)
        .add_plugins((menu::MenuPlugin, game::GamePlugin))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(),));
}
