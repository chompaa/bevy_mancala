use super::{board::SlotPressEvent, Board, CurrentPlayer, GameState, Player, Slot};
use crate::{
    menu::{MenuState, Selected},
    profile::Profiles,
};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use rand::Rng;

pub const AI_NAME: &str = "CPU";

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AiState>()
            .init_resource::<AiPlayer>()
            .init_resource::<AiThinkTimer>()
            .add_systems(
                Update,
                selected_changed.run_if(in_state(MenuState::Profile)),
            )
            .add_systems(
                OnEnter(GameState::Idle),
                move_end.run_if(in_state(AiState::Idle)),
            )
            .add_systems(Update, play.run_if(in_state(AiState::Thinking)));
    }
}

#[derive(Resource, Default)]
pub struct AiPlayer(pub Option<Player>);

#[derive(Resource)]
struct AiThinkTimer(Timer);

impl Default for AiThinkTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1., TimerMode::Once))
    }
}

#[derive(States, SystemSet, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum AiState {
    #[default]
    Inactive,
    Idle,
    Thinking,
}

fn selected_changed(
    mut ai_state: ResMut<NextState<AiState>>,
    mut ai_player: ResMut<AiPlayer>,
    selected: Res<Selected>,
    profiles: Res<Persistent<Profiles>>,
) {
    if !selected.is_changed() {
        return;
    }

    for (player, profile) in Player::iter().zip(selected.profiles.into_iter()) {
        if profiles.0[profile].name == AI_NAME {
            ai_player.0 = Some(player);
            ai_state.set(AiState::Idle);
            return;
        }
    }

    // no AI profile was selected
    ai_player.0 = None;
    ai_state.set(AiState::Inactive);
}

fn move_end(
    mut ai_state: ResMut<NextState<AiState>>,
    ai_player: Res<AiPlayer>,
    current_player: Res<CurrentPlayer>,
) {
    if ai_player.0 != Some(current_player.0) {
        return;
    }

    ai_state.set(AiState::Thinking);
}

fn play(
    slot_query: Query<&Slot>,
    ai_player: Res<AiPlayer>,
    board: Res<Board>,
    time: Res<Time>,
    mut ai_state: ResMut<NextState<AiState>>,
    mut ai_think_timer: ResMut<AiThinkTimer>,
    mut slot_press_evw: EventWriter<SlotPressEvent>,
) {
    if !ai_think_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let slot = loop {
        let mut rng = rand::thread_rng();
        let slots = Board::get_slots(ai_player.0.unwrap());
        let choice = rng.gen_range(0..slots.len());
        let slot = board.slots[choice];

        if slot_query.get(slot).unwrap().count > 0 {
            break slot;
        }
    };

    slot_press_evw.send(SlotPressEvent(slot));

    ai_state.set(AiState::Idle);
    ai_think_timer.0.reset();
}
