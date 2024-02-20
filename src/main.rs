use bevy::prelude::*;

mod game;
mod menu;
mod profile;
mod states;
mod ui;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some("bevy".to_string()),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_state::<states::AppState>()
        .add_state::<states::GameMode>()
        .add_systems(Startup, setup)
        .add_plugins((
            ui::UiPlugin,
            menu::MenuPlugin,
            game::GamePlugin,
            profile::ProfilePlugin,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(),));
}
