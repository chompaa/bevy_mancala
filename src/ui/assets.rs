use bevy::prelude::*;

use super::UiAssets;

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/LGGothic.ttf");
    let marble = asset_server.load("textures/marble.png");

    commands.insert_resource(UiAssets { font, marble });
}
