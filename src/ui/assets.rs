use bevy::prelude::*;

use super::UiAssets;

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/LGGothic.ttf");

    commands.insert_resource(UiAssets { font });
}
