use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MoveState {
    Logic,
    Animation,
    Tick,
}
