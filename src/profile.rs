use bevy::prelude::*;
use bevy_persistent::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Profile {
    pub name: String,
    pub wins: u32,
}

#[derive(Resource, Serialize, Deserialize)]
pub struct Profiles(pub Vec<Profile>);

fn setup(mut commands: Commands) {
    let dir = dirs::config_dir()
        .map(|dir| dir.join("bevy_mancala"))
        .unwrap_or(Path::new("local").join("config"));

    commands.insert_resource(
        Persistent::<Profiles>::builder()
            .name("profiles")
            .format(StorageFormat::RonPrettyWithStructNames)
            .path(dir.join("profiles.ron"))
            .default(Profiles(vec![
                Profile {
                    name: "PL1".to_string(),
                    wins: 0,
                },
                Profile {
                    name: "PL2".to_string(),
                    wins: 0,
                },
            ]))
            .build()
            .expect("failed to initialize profiles"),
    );
}
