use crate::states::{AppState, GameMode};
use crate::ui::ReloadUiEvent;
use bevy::prelude::*;
use board::SlotPressEvent;
use std::collections::VecDeque;
use std::ops::Range;

mod animation;
mod board;
mod helpers;
mod label;
mod marble;
mod player;

const SLOT_START_AMOUNT: u32 = 6;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            animation::AnimationPlugin,
            board::BoardPlugin,
            label::LabelPlugin,
            marble::MarblePlugin,
            player::PlayerPlugin,
        ))
        .init_resource::<CurrentPlayer>()
        .init_resource::<Board>()
        .add_event::<MoveEvent>()
        .add_event::<CaptureEvent>()
        .add_event::<GameOverEvent>()
        .add_systems(OnEnter(AppState::Game), setup_slots)
        .add_systems(
            Update,
            (handle_move, check_game_over.run_if(on_event::<MoveEvent>()))
                .run_if(in_state(AppState::Game))
                .chain(),
        );
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Player {
    #[default]
    Player1,
    Player2,
}

impl Player {
    pub fn flip(&self) -> Self {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Player::Player1, Player::Player1) => true,
            (Player::Player2, Player::Player2) => true,
            _ => false,
        }
    }
}

impl ToString for Player {
    fn to_string(&self) -> String {
        match self {
            Player::Player1 => "Player 1".to_string(),
            Player::Player2 => "Player 2".to_string(),
        }
    }
}

#[derive(Event)]
pub struct MoveEvent(pub u32, pub VecDeque<Entity>);

#[derive(Event, Clone)]
pub struct CaptureEvent {
    slots: Vec<Entity>,
    store: Entity,
}

#[derive(Event)]
pub struct GameOverEvent(pub Option<Player>);

#[derive(Component)]
pub struct Slot {
    pub index: usize,
    pub count: u32,
}

#[derive(Resource, Debug)]
pub struct CurrentPlayer(pub Player);

impl Default for CurrentPlayer {
    fn default() -> Self {
        Self(Player::default())
    }
}

impl CurrentPlayer {
    pub fn flip(&mut self) {
        self.0 = self.0.flip();
    }
}

#[derive(Resource)]
pub struct Board {
    pub slots: Vec<Entity>,
}

impl Board {
    pub const LENGTH: usize = 14;
    pub const STORE_1: usize = (Board::LENGTH - 1) / 2;
    pub const STORE_2: usize = Board::LENGTH - 1;
    pub const ROWS: usize = 2;
    pub const COLS: usize = 6;

    pub fn is_store(index: usize) -> bool {
        index == Board::STORE_1 || index == Board::STORE_2
    }

    pub fn get_store(player: Player) -> usize {
        match player {
            Player::Player1 => Board::STORE_1,
            Player::Player2 => Board::STORE_2,
        }
    }

    pub fn get_slots(player: Player) -> Range<usize> {
        match player {
            Player::Player1 => 0..Board::STORE_1,
            Player::Player2 => Board::STORE_1 + 1..Board::STORE_2,
        }
    }

    pub fn owner(index: usize) -> Player {
        if index <= (Board::LENGTH - 1) / 2 {
            Player::Player1
        } else {
            Player::Player2
        }
    }

    pub fn slot_order() -> Vec<usize> {
        let mid = (Board::LENGTH - 2) / 2;

        (0..Board::LENGTH)
            .map(|s| if s > mid { s } else { mid - s })
            .collect()
    }
}

impl Default for Board {
    fn default() -> Self {
        Self { slots: vec![] }
    }
}

fn setup_slots(
    mut commands: Commands,
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

        let entity = commands.spawn(slot).id();
        board.slots.push(entity)
    }

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
        let slot = slot_query.get(event.0).unwrap();

        if slot.count == 0 || Board::owner(slot.index) != current_player.0 {
            continue;
        }

        let mut counts: Vec<u32> = vec![0; Board::LENGTH];

        for slot in &mut slot_query.iter() {
            counts[slot.index] = slot.count;
        }

        let start = slot_query.get(event.0).unwrap();
        let mut index = start.index;
        let mut move_count = 0;

        loop {
            let mut stack = counts[index];
            let mut moves: VecDeque<Entity> = VecDeque::new();
            counts[index] = 0;

            let start_move = move_count;
            moves.push_back(board.slots[index]);

            while stack > 0 {
                index = (index + 1) % Board::LENGTH;

                if index == Board::get_store(current_player.0.flip()) {
                    // skip the opponent's store
                    continue;
                }

                move_count += 1;
                counts[index] += 1;
                stack -= 1;

                moves.push_back(board.slots[index]);
            }

            move_events.send(MoveEvent(start_move, moves));

            if index == Board::get_store(current_player.0) {
                // if we end in our own store, we get another turn
                break;
            }

            if *game_mode.get() == GameMode::Capture
                && counts[index] == 1
                && Board::owner(index) == current_player.0
            {
                let opposite_index = Board::LENGTH - index - 2;

                if counts[opposite_index] > 0 {
                    counts[index] = 0;
                    counts[opposite_index] = 0;

                    let store = Board::get_store(current_player.0.clone());

                    counts[store] += 2;

                    capture_events.send(CaptureEvent {
                        slots: vec![board.slots[index], board.slots[opposite_index]],
                        store: board.slots[store],
                    });

                    break;
                }
            }

            if *game_mode.get() == GameMode::Avalanche && counts[index] > 1 {
                continue;
            }

            current_player.flip();
            break;
        }

        for mut slot in &mut slot_query {
            slot.count = counts[slot.index];
        }
    }
}

fn check_game_over(
    mut game_over_events: EventWriter<GameOverEvent>,
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
                Player::Player1 => {
                    if Board::is_store(slot.index) {
                        scores.0 = slot.count;
                    } else if slot.count > 0 {
                        empty.0 = false;
                    }
                }
                Player::Player2 => {
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
            scores.0 += capture_side(capture_events, slot_query, &Player::Player1, slots)
        } else if !empty.1 {
            scores.1 += capture_side(capture_events, slot_query, &Player::Player2, slots)
        }
    }

    let winner: Option<Player> = if scores.0 > scores.1 {
        Some(Player::Player1)
    } else if scores.1 > scores.0 {
        Some(Player::Player2)
    } else {
        None
    };

    game_over_events.send(GameOverEvent(winner));
}

fn capture_side(
    mut capture_events: EventWriter<CaptureEvent>,
    slot_query: Query<&Slot>,
    player: &Player,
    slots: &Vec<Entity>,
) -> u32 {
    let slot_slice = Board::get_slots(*player);
    let store_index = Board::get_store(*player);

    capture_events.send(CaptureEvent {
        slots: slots[slot_slice.clone()].to_vec(),
        store: slots[store_index],
    });

    slots[slot_slice].iter().fold(0, |acc, slot| {
        let count = slot_query.get(*slot).unwrap().count;
        acc + count
    })
}
