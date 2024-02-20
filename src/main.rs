use bevy::prelude::*;

mod game;
mod menu;
mod profile;
mod states;
mod ui;

// (30, 48, 51)
const BACKGROUND_COLOR_MAIN: Color = Color::rgb(0.11764706, 0.1882353, 0.2);
// (45, 22, 48)
const BACKGROUND_COLOR_ALT: Color = Color::rgb(0.1764706, 0.08627451, 0.1882353);

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR_MAIN))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<states::AppState>()
        .init_state::<states::GameMode>()
        .add_systems(Startup, setup)
        .add_systems(Update, animate_background_color)
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

fn animate_background_color(time: Res<Time>, mut color: ResMut<ClearColor>) {
    let t = f32::sin(time.elapsed_seconds_wrapped() * 0.25);

    color.0 = Color::rgb(
        BACKGROUND_COLOR_MAIN.r() * t + BACKGROUND_COLOR_ALT.r() * (1. - t),
        BACKGROUND_COLOR_MAIN.g() * t + BACKGROUND_COLOR_ALT.g() * (1. - t),
        BACKGROUND_COLOR_MAIN.b() * t + BACKGROUND_COLOR_ALT.b() * (1. - t),
    );
}
