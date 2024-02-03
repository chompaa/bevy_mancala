use crate::game::Board;

pub const SLOT_SIZE: f32 = 64.0;
pub const SLOT_GAP: f32 = 12.0;

pub const STORE_WIDTH: f32 = 64.0 + 8.0;
pub const STORE_HEIGHT: f32 = 128.0 + 28.0;
pub const STORE_GAP: f32 = 0.0;

pub const LABEL_SIZE: f32 = 64.0;
pub const LABEL_SLOT_GAP_X: f32 = 12.;
pub const LABEL_SLOT_GAP_Y: f32 = 208.0;
pub const LABEL_STORE_GAP_X: f32 = 103.0;

pub const BOARD_WIDTH: f32 = SLOT_SIZE * (Board::COLS as f32)
    + SLOT_GAP * ((Board::COLS - 1) as f32)
    + 2. * STORE_WIDTH
    + 2. * STORE_GAP;
pub const BOARD_HEIGHT: f32 =
    SLOT_SIZE * (Board::ROWS as f32) + SLOT_GAP * ((Board::ROWS - 1) as f32);

pub const MARBLE_SIZE: f32 = 48.0;

pub const MOVE_SPEED: f32 = 100.;
pub const MOVE_TOLERANCE: f32 = 2.;
