use bevy::prelude::*;

mod animation;
mod assets;
pub mod board;
mod game_over;
mod helpers;
mod label;
mod marble;
mod player;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(board::BoardPlugin)
            .add_plugins(animation::AnimationPlugin)
            .add_plugins(marble::MarblePlugin)
            .add_plugins(player::PlayerPlugin)
            .add_plugins(label::LabelPlugin)
            .add_plugins(game_over::GameOverPlugin)
            .add_event::<ReloadUiEvent>()
            .add_systems(Startup, assets::load_assets);
    }
}

#[derive(Event)]
pub struct ReloadUiEvent;

impl Default for ReloadUiEvent {
    fn default() -> Self {
        Self
    }
}

#[derive(Resource)]
pub struct UiAssets {
    pub font: Handle<Font>,
    pub marble: Handle<Image>,
}
