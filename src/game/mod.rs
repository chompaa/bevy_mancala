use self::animation::AnimationState;
use crate::{
    states::{AppState, GameMode},
    ui::ReloadUiEvent,
};
use bevy::prelude::*;
use board::SlotPressEvent;
use std::{cmp::Ordering, collections::VecDeque, ops::Range};

pub mod ai;
mod animation;
mod board;
mod game_over;
mod helpers;
mod label;
mod marble;
mod turn_indicator;

const SLOT_START_AMOUNT: u32 = 6;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ai::AiPlugin,
            animation::AnimationPlugin,
            board::BoardPlugin,
            game_over::GameOverPlugin,
            label::LabelPlugin,
            marble::MarblePlugin,
            turn_indicator::TurnIndicatorPlugin,
        ))
        .init_state::<GameState>()
        .init_resource::<CurrentPlayer>()
        .init_resource::<Board>()
        .init_resource::<Winner>()
        .add_event::<MoveEvent>()
        .add_event::<CaptureEvent>()
        .add_event::<TurnEndEvent>()
        .add_systems(OnEnter(AppState::Game), setup_slots)
        .add_systems(Update, (handle_move).run_if(in_state(AppState::Game)))
        .add_systems(
            OnEnter(AnimationState::Animating),
            handle_animation_start
                .run_if(in_state(GameState::Idle))
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(
            OnEnter(AnimationState::Idle),
            handle_animation_end
                .run_if(in_state(GameState::Playing))
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(OnEnter(GameState::Idle), check_game_over);
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Player {
    #[default]
    One,
    Two,
}

impl Player {
    pub const fn flip(self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::One,
        }
    }

    pub fn iter() -> impl Iterator<Item = Player> {
        [Player::One, Player::Two].iter().copied()
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::One, Self::One) | (Self::Two, Self::Two)
        )
    }
}

impl ToString for Player {
    fn to_string(&self) -> String {
        match self {
            Self::One => "PLAYER 1".to_string(),
            Self::Two => "PLAYER 2".to_string(),
        }
    }
}

enum MoveEndAction {
    Repeat,
    Continue,
    End,
}

#[derive(Event)]
pub struct MoveEvent(pub VecDeque<Entity>);

#[derive(Event, Clone)]
pub struct CaptureEvent {
    slots: Vec<Entity>,
    store: Entity,
}

#[derive(Event, Default)]
pub struct TurnEndEvent;

#[derive(Component)]
pub struct Slot {
    pub index: usize,
    pub count: u32,
}

#[derive(Resource, Default, Debug)]
pub struct CurrentPlayer(pub Player);

impl CurrentPlayer {
    pub fn flip(&mut self) {
        self.0 = self.0.flip();
    }
}

#[derive(Resource, Default)]
pub struct Board {
    pub slots: Vec<Entity>,
}

impl Board {
    pub const LENGTH: usize = 14;
    pub const STORE_1: usize = (Board::LENGTH - 1) / 2;
    pub const STORE_2: usize = Board::LENGTH - 1;
    pub const ROWS: usize = 2;
    pub const COLS: usize = 6;

    pub const fn is_store(index: usize) -> bool {
        index == Self::STORE_1 || index == Self::STORE_2
    }

    pub const fn get_store(player: Player) -> usize {
        match player {
            Player::One => Self::STORE_1,
            Player::Two => Self::STORE_2,
        }
    }

    pub const fn get_slots(player: Player) -> Range<usize> {
        match player {
            Player::One => 0..Self::STORE_1,
            Player::Two => Self::STORE_1 + 1..Self::STORE_2,
        }
    }

    pub const fn owner(index: usize) -> Player {
        if index <= (Self::LENGTH - 1) / 2 {
            Player::One
        } else {
            Player::Two
        }
    }

    pub fn slot_order() -> Vec<usize> {
        let mid = (Self::LENGTH - 2) / 2;

        (0..Self::LENGTH)
            .map(|s| if s > mid { s } else { mid - s })
            .collect()
    }
}
#[derive(Resource, Default)]
pub struct Winner(Option<Player>);

#[derive(States, SystemSet, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum GameState {
    #[default]
    None,
    Idle,
    Playing,
    Over,
}

fn setup_slots(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    mut reload_ui_event: EventWriter<ReloadUiEvent>,
    mut board: ResMut<Board>,
) {
    let board_texture = asset_server.load("textures/board.png");

    commands.spawn(SpriteBundle {
        texture: board_texture,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -100.),
            scale: Vec3::new(4.0, 4.0, 1.0),
            ..default()
        },
        ..default()
    });

    for index in 0..Board::LENGTH {
        let mut slot = Slot {
            index,
            count: if Board::is_store(index) {
                0
            } else {
                SLOT_START_AMOUNT
            },
        };

        if slot.index == 4 || slot.index == 7 {
            slot.count = 1;
        } else {
            slot.count = 0;
        }

        let entity = commands.spawn(slot).id();
        board.slots.push(entity);
    }

    game_state.set(GameState::Idle);

    reload_ui_event.send_default();
}

fn handle_move(
    board: Res<Board>,
    game_mode: Res<State<GameMode>>,
    mut current_player: ResMut<CurrentPlayer>,
    mut slot_query: Query<&mut Slot>,
    mut slot_press_events: EventReader<SlotPressEvent>,
    mut move_events: EventWriter<MoveEvent>,
    mut capture_events: EventWriter<CaptureEvent>,
) {
    for event in slot_press_events.read() {
        let mut counts: Vec<u32> = vec![0; Board::LENGTH];

        for slot in &mut slot_query.iter() {
            counts[slot.index] = slot.count;
        }

        let start = slot_query.get(event.0).unwrap();
        let mut index = start.index;

        loop {
            let mut stack = counts[index];
            let mut moves: VecDeque<Entity> = VecDeque::new();
            counts[index] = 0;

            moves.push_back(board.slots[index]);

            while stack > 0 {
                index = (index + 1) % Board::LENGTH;

                if index == Board::get_store(current_player.0.flip()) {
                    // skip the opponent's store
                    continue;
                }

                counts[index] += 1;
                stack -= 1;

                moves.push_back(board.slots[index]);
            }

            move_events.send(MoveEvent(moves));

            let move_end_action = handle_move_end(
                &board,
                index,
                *game_mode.get(),
                current_player.0,
                &mut counts,
                &mut capture_events,
            );

            match move_end_action {
                MoveEndAction::Repeat => continue,
                MoveEndAction::Continue => break,
                MoveEndAction::End => {
                    current_player.flip();
                    break;
                }
            }
        }

        for mut slot in &mut slot_query {
            slot.count = counts[slot.index];
        }
    }
}

fn handle_move_end(
    board: &Board,
    index: usize,
    game_mode: GameMode,
    current_player: Player,
    counts: &mut Vec<u32>,
    capture_evw: &mut EventWriter<CaptureEvent>,
) -> MoveEndAction {
    if index == Board::get_store(current_player) {
        return MoveEndAction::Continue;
    }

    match game_mode {
        GameMode::Capture => {
            if counts[index] == 1 && Board::owner(index) == current_player {
                let opposite_index = Board::LENGTH - index - 2;

                if counts[opposite_index] > 0 {
                    let store = Board::get_store(current_player);

                    counts[store] += counts[opposite_index] + 1;
                    counts[opposite_index] = 0;
                    counts[index] = 0;

                    capture_evw.send(CaptureEvent {
                        slots: vec![board.slots[index], board.slots[opposite_index]],
                        store: board.slots[store],
                    });

                    return MoveEndAction::Continue;
                }
            }
        }
        GameMode::Avalanche => {
            if counts[index] > 1 {
                return MoveEndAction::Repeat;
            }
        }
    }

    MoveEndAction::End
}

fn check_game_over(
    mut winner: ResMut<Winner>,
    mut game_state: ResMut<NextState<GameState>>,
    capture_events: EventWriter<CaptureEvent>,
    slot_query: Query<&Slot>,
    board: Res<Board>,
    game_mode: Res<State<GameMode>>,
) {
    let slots = &board.slots.clone();

    let mut scores = (0, 0);
    let mut empty = (true, true);

    for slot in slots {
        if let Ok(slot) = slot_query.get(*slot) {
            match Board::owner(slot.index) {
                Player::One => {
                    if Board::is_store(slot.index) {
                        scores.0 = slot.count;
                    } else if slot.count > 0 {
                        empty.0 = false;
                    }
                }
                Player::Two => {
                    if Board::is_store(slot.index) {
                        scores.1 = slot.count;
                    } else if slot.count > 0 {
                        empty.1 = false;
                    }
                }
            }
        }
    }

    if !empty.0 && !empty.1 {
        return;
    }

    if *game_mode.get() == GameMode::Capture {
        if !empty.0 {
            scores.0 += capture_side(capture_events, slot_query, Player::One, slots);
        } else if !empty.1 {
            scores.1 += capture_side(capture_events, slot_query, Player::Two, slots);
        }
    }

    winner.0 = match scores.0.cmp(&scores.1) {
        Ordering::Greater => Some(Player::One),
        Ordering::Less => Some(Player::Two),
        Ordering::Equal => None,
    };

    game_state.set(GameState::Over);
}

fn capture_side(
    mut capture_events: EventWriter<CaptureEvent>,
    slot_query: Query<&Slot>,
    player: Player,
    slots: &[Entity],
) -> u32 {
    let slot_slice = Board::get_slots(player);
    let store_index = Board::get_store(player);

    capture_events.send(CaptureEvent {
        slots: slots[slot_slice.clone()].to_vec(),
        store: slots[store_index],
    });

    slots[slot_slice].iter().fold(0, |acc, slot| {
        let count = slot_query.get(*slot).unwrap().count;
        acc + count
    })
}

fn handle_animation_start(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::Playing);
}

fn handle_animation_end(
    mut game_state: ResMut<NextState<GameState>>,
    mut turn_end_evr: EventWriter<TurnEndEvent>,
) {
    game_state.set(GameState::Idle);
    turn_end_evr.send_default();
}
