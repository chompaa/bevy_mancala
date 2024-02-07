use bevy::prelude::*;

pub mod animation;
pub mod assets;
pub mod board;
pub mod game_over;
mod helpers;
pub mod label;
pub mod marble;
pub mod player;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReloadUiEvent>()
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
    pub title: Handle<Image>,
    pub marble: Handle<Image>,
}
