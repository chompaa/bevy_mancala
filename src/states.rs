use bevy::prelude::*;

#[derive(States, SystemSet, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum AppState {
    #[default]
    Menu,
    Game,
}

#[derive(States, SystemSet, Debug, Hash, PartialEq, Eq, Copy, Clone, Default)]
pub enum GameMode {
    #[default]
    Avalanche,
    Capture,
}

impl ToString for GameMode {
    fn to_string(&self) -> String {
        match self {
            Self::Avalanche => "AVALANCHE".to_string(),
            Self::Capture => "CAPTURE".to_string(),
        }
    }
}

impl GameMode {
    pub fn iter() -> impl Iterator<Item = GameMode> {
        [GameMode::Avalanche, GameMode::Capture].iter().copied()
    }
}
