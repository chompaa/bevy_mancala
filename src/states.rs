use bevy::prelude::*;

#[derive(States, SystemSet, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum AppState {
    #[default]
    Menu,
    Game,
}

#[derive(States, SystemSet, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum GameMode {
    #[default]
    Avalanche,
}
