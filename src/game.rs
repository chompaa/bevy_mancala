use std::collections::VecDeque;

use crate::ui::{board::SlotPressEvent, ReloadUiEvent, UiPlugin};
use crate::GameState;
use bevy::prelude::*;

const SLOT_START_AMOUNT: u32 = 6;

#[derive(Default, Debug)]
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
pub struct MoveEvent(pub u32, pub u32, pub VecDeque<Entity>);

#[derive(Event, Default)]
pub struct MoveEndEvent;

#[derive(Event)]
pub struct GameOverEvent(pub Option<Player>);

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
    pub const ROWS: usize = 2;
    pub const COLS: usize = 6;

    pub fn is_store(index: usize) -> bool {
        index == (Board::LENGTH - 1) / 2 || index == Board::LENGTH - 1
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

#[derive(Component)]
pub struct Slot {
    pub index: usize,
    pub count: u32,
}

#[derive(Component, Debug)]
pub struct Store;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiPlugin)
            .init_resource::<CurrentPlayer>()
            .init_resource::<Board>()
            .add_event::<MoveEvent>()
            .add_event::<MoveEndEvent>()
            .add_event::<GameOverEvent>()
            .add_systems(OnEnter(GameState::Game), setup_slots)
            .add_systems(
                Update,
                (handle_move, check_game_over.run_if(on_event::<MoveEvent>()))
                    .run_if(in_state(GameState::Game)),
            );
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
            count: SLOT_START_AMOUNT,
        };

        let entity = if Board::is_store(index) {
            slot.count = 0;
            commands.spawn((slot, Store)).id()
        } else {
            commands.spawn(slot).id()
        };

        board.slots.push(entity)
    }

    reload_ui_event.send_default();
}

fn handle_move(
    board: Res<Board>,
    mut current_player: ResMut<CurrentPlayer>,
    mut slot_query: Query<&mut Slot>,
    mut slot_press_events: EventReader<SlotPressEvent>,
    mut move_events: EventWriter<MoveEvent>,
    mut move_end_events: EventWriter<MoveEndEvent>,
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
            let start_stack = stack;
            moves.push_back(board.slots[index]);

            while stack > 0 {
                index = (index + 1) % Board::LENGTH;

                if Board::is_store(index) && Board::owner(index) != current_player.0 {
                    continue;
                }

                move_count += 1;
                counts[index] += 1;
                stack -= 1;

                moves.push_back(board.slots[index]);
            }

            move_events.send(MoveEvent(start_move, start_stack, moves));

            if Board::owner(index) == current_player.0 && Board::is_store(index) {
                // if we end in our own store, we get another turn
                break;
            }

            if counts[index] == 1 {
                current_player.flip();
                break;
            }
        }

        for mut slot in &mut slot_query {
            slot.count = counts[slot.index];
        }

        move_end_events.send_default();
    }
}

fn check_game_over(
    mut game_over_events: EventWriter<GameOverEvent>,
    slot_query: Query<&Slot>,
    board: Res<Board>,
) {
    let mut player_1_score = 0;
    let mut player_2_score = 0;

    let mut player_1_empty = true;
    let mut player_2_empty = true;

    for slot in &board.slots {
        if let Ok(slot) = slot_query.get(*slot) {
            if Board::is_store(slot.index) {
                match Board::owner(slot.index) {
                    Player::Player1 => player_1_score += slot.count,
                    Player::Player2 => player_2_score += slot.count,
                }
            } else if slot.count > 0 {
                match Board::owner(slot.index) {
                    Player::Player1 => player_1_empty = false,
                    Player::Player2 => player_2_empty = false,
                }
            }
        }
    }

    if !player_1_empty && !player_2_empty {
        return;
    }

    let winner: Option<Player> = if player_1_score > player_2_score {
        Some(Player::Player1)
    } else if player_2_score > player_1_score {
        Some(Player::Player2)
    } else {
        None
    };

    game_over_events.send(GameOverEvent(winner));
}
