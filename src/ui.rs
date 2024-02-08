use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReloadUiEvent>()
            .add_systems(Startup, load_assets);
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

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/8bitoperator_jve.ttf");
    let title = asset_server.load("textures/title.png");
    let marble = asset_server.load("textures/marble.png");

    commands.insert_resource(UiAssets {
        font,
        title,
        marble,
    });
}
