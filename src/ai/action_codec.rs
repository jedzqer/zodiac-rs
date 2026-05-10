pub const BOARD_WIDTH: usize = 6;
pub const BOARD_HEIGHT: usize = 4;
pub const NUM_CELLS: usize = BOARD_WIDTH * BOARD_HEIGHT;
pub const FLIP_ACTIONS: usize = NUM_CELLS;
pub const MOVE_ACTIONS: usize = NUM_CELLS * NUM_CELLS;
pub const NUM_ACTIONS: usize = FLIP_ACTIONS + MOVE_ACTIONS;
pub const MOVE_OFFSET: usize = FLIP_ACTIONS;

use super::{Action, ActionType};

pub fn pos_to_index(pos: (usize, usize)) -> usize {
    pos.0 * BOARD_HEIGHT + pos.1
}

pub fn index_to_pos(index: usize) -> (usize, usize) {
    (index / BOARD_HEIGHT, index % BOARD_HEIGHT)
}

pub fn encode_flip_action(pos: (usize, usize)) -> usize {
    pos_to_index(pos)
}

pub fn encode_move_action(src_pos: (usize, usize), dst_pos: (usize, usize)) -> usize {
    MOVE_OFFSET + pos_to_index(src_pos) * NUM_CELLS + pos_to_index(dst_pos)
}

pub fn decode_action(action_id: usize) -> Option<Action> {
    if action_id >= NUM_ACTIONS {
        return None;
    }

    if action_id < MOVE_OFFSET {
        return Some(Action {
            action_type: ActionType::Flip,
            self_pos: index_to_pos(action_id),
            target_pos: None,
            score: 0.0,
        });
    }

    let move_id = action_id - MOVE_OFFSET;
    let src = move_id / NUM_CELLS;
    let dst = move_id % NUM_CELLS;
    Some(Action {
        action_type: ActionType::Move,
        self_pos: index_to_pos(src),
        target_pos: Some(index_to_pos(dst)),
        score: 0.0,
    })
}
