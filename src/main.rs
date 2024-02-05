use bevy::prelude::*;

mod game;
mod menu;
mod states;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_state::<states::AppState>()
        .add_state::<states::GameMode>()
        .add_systems(Startup, setup)
        .add_plugins((menu::MenuPlugin, game::GamePlugin))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(),));
}
