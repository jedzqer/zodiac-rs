use crate::ai::action_codec::{NUM_ACTIONS, NUM_CELLS, pos_to_index};
use crate::game::board::{Board, Cell};
use crate::game::piece::{Camp, Piece};

pub const CELL_FEATURE_DIM: usize = 36;
pub const GLOBAL_FEATURE_DIM: usize = 16;
pub const HISTORY_FEATURE_DIM: usize = 59;
pub const HISTORY_LEN: usize = 16;

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub action_type: Option<super::ActionType>,
    pub self_pos: Option<(usize, usize)>,
    pub target_pos: Option<(usize, usize)>,
    pub actor: Camp,
    pub result: HistoryResult,
}

#[derive(Debug, Clone, Copy)]
pub enum HistoryResult {
    Ok,
    Invalid,
    Eat,
    Special,
}

pub struct EncodedState {
    pub cell_features: Vec<f32>,
    pub global_features: Vec<f32>,
    pub history_features: Vec<f32>,
    pub history_mask: Vec<bool>,
    pub legal_action_mask: Vec<bool>,
}

#[derive(Default)]
pub struct StateEncoder {
    pub history_len: usize,
}

impl StateEncoder {
    pub fn new(history_len: usize) -> Self {
        Self { history_len }
    }

    pub fn encode(
        &self,
        board: &Board,
        perspective: Camp,
        current_player: Camp,
        history: &[HistoryEntry],
    ) -> EncodedState {
        let mut cell_features = vec![0.0; NUM_CELLS * CELL_FEATURE_DIM];
        let mut visible_own_count = 0.0;
        let mut visible_opp_count = 0.0;
        let mut hidden_count = 0.0;
        let mut revealed_count = 0.0;
        let mut own_revealed_count = 0.0;
        let mut opp_revealed_count = 0.0;
        let mut own_visible_strength = 0.0;
        let mut opp_visible_strength = 0.0;

        for x in 0..6 {
            for y in 0..4 {
                let idx = pos_to_index((x, y));
                let base = idx * CELL_FEATURE_DIM;
                let feat = &mut cell_features[base..base + CELL_FEATURE_DIM];
                let cell = &board.cells[x][y];

                feat[32] = x as f32 / 5.0;
                feat[33] = y as f32 / 3.0;
                let center_dist = (x as f32 - 2.5).abs() + (y as f32 - 1.5).abs();
                feat[34] = center_dist / 4.0;
                let edge_dist = x.min(5 - x).min(y.min(3 - y)) as f32;
                feat[35] = edge_dist / 1.5;

                if let Some(piece) = &cell.piece {
                    feat[0] = 1.0;
                    if cell.revealed {
                        feat[1] = 1.0;
                        fill_piece_features(feat, 3, 4, 5, piece, perspective);
                        revealed_count += 1.0;
                        if piece.camp == perspective {
                            visible_own_count += 1.0;
                            own_revealed_count += 1.0;
                            own_visible_strength += 13.0 - piece.lv as f32;
                        } else {
                            visible_opp_count += 1.0;
                            opp_revealed_count += 1.0;
                            opp_visible_strength += 13.0 - piece.lv as f32;
                        }
                    } else {
                        feat[2] = 1.0;
                        hidden_count += 1.0;
                    }
                }

                if let Some(monkey) = &cell.monkey {
                    feat[17] = 1.0;
                    fill_piece_features(feat, 18, 19, 20, monkey, perspective);
                    if monkey.camp == perspective {
                        visible_own_count += 1.0;
                        own_visible_strength += 13.0 - monkey.lv as f32;
                    } else {
                        visible_opp_count += 1.0;
                        opp_visible_strength += 13.0 - monkey.lv as f32;
                    }
                }
            }
        }

        let mut global_features = vec![0.0; GLOBAL_FEATURE_DIM];
        global_features[0] = if current_player == Camp::Black { 1.0 } else { 0.0 };
        global_features[1] = if current_player == Camp::Red { 1.0 } else { 0.0 };
        global_features[2] = if perspective == Camp::Black { 1.0 } else { 0.0 };
        global_features[3] = if perspective == Camp::Red { 1.0 } else { 0.0 };
        global_features[4] = visible_own_count / 12.0;
        global_features[5] = visible_opp_count / 12.0;
        global_features[6] = hidden_count / NUM_CELLS as f32;
        global_features[7] = revealed_count / NUM_CELLS as f32;
        global_features[8] = own_revealed_count / 12.0;
        global_features[9] = opp_revealed_count / 12.0;
        global_features[10] = hidden_count * 0.5 / 12.0;
        global_features[11] = hidden_count * 0.5 / 12.0;
        global_features[12] = own_visible_strength / 78.0;
        global_features[13] = opp_visible_strength / 78.0;
        global_features[14] = (history.len() as f32 / 200.0).min(1.0);
        global_features[15] = (history.len() as f32 / self.history_len.max(1) as f32).min(1.0);

        let (history_features, history_mask) = self.encode_history(history, perspective);
        let legal_action_mask = build_legal_action_mask(board, current_player);

        EncodedState {
            cell_features,
            global_features,
            history_features,
            history_mask,
            legal_action_mask,
        }
    }

    fn encode_history(&self, history: &[HistoryEntry], perspective: Camp) -> (Vec<f32>, Vec<bool>) {
        let len = self.history_len.max(1);
        let mut feats = vec![0.0; len * HISTORY_FEATURE_DIM];
        let mut mask = vec![false; len];
        let recent = if history.len() > len {
            &history[history.len() - len..]
        } else {
            history
        };
        let start = len - recent.len();

        for (i, h) in recent.iter().enumerate() {
            let row = start + i;
            mask[row] = true;
            let offset = row * HISTORY_FEATURE_DIM;
            let feat = &mut feats[offset..offset + HISTORY_FEATURE_DIM];

            match h.action_type {
                Some(super::ActionType::Flip) => feat[0] = 1.0,
                Some(super::ActionType::Move) => feat[1] = 1.0,
                None => feat[2] = 1.0,
            }

            if let Some(src) = h.self_pos {
                let src_idx = pos_to_index(src);
                feat[3 + src_idx] = 1.0;
                let dst_idx = pos_to_index(h.target_pos.unwrap_or(src));
                feat[3 + NUM_CELLS + dst_idx] = 1.0;
            }

            if h.actor == perspective {
                feat[3 + 2 * NUM_CELLS] = 1.0;
            } else {
                feat[3 + 2 * NUM_CELLS + 1] = 1.0;
            }

            let result_offset = 3 + 2 * NUM_CELLS + 2;
            feat[result_offset + h.result.as_index()] = 1.0;
        }

        (feats, mask)
    }
}

impl HistoryResult {
    fn as_index(self) -> usize {
        match self {
            Self::Ok => 0,
            Self::Invalid => 1,
            Self::Eat => 2,
            Self::Special => 3,
        }
    }
}

fn fill_piece_features(
    feat: &mut [f32],
    own_offset: usize,
    opp_offset: usize,
    lv_offset: usize,
    piece: &Piece,
    perspective: Camp,
) {
    if piece.camp == perspective {
        feat[own_offset] = 1.0;
    } else {
        feat[opp_offset] = 1.0;
    }
    let lv = piece.lv as usize;
    if (1..=12).contains(&lv) {
        feat[lv_offset + lv - 1] = 1.0;
    }
}

fn build_legal_action_mask(board: &Board, camp: Camp) -> Vec<bool> {
    let mut legal = vec![false; NUM_ACTIONS];
    for x in 0..6 {
        for y in 0..4 {
            let cell = &board.cells[x][y];
            if cell.piece.is_some() && !cell.revealed && cell.monkey.is_none() {
                let action_id = pos_to_index((x, y));
                legal[action_id] = true;
            }
        }
    }

    for sx in 0..6 {
        for sy in 0..4 {
            let cell = &board.cells[sx][sy];
            let Some(piece) = movable_piece(cell) else { continue };
            if piece.camp != camp || (!cell.revealed && cell.monkey.is_none()) {
                continue;
            }
            for tx in 0..6 {
                for ty in 0..4 {
                    let mut sim = board.clone();
                    let (ok, _) = sim.dispose_piece((sx, sy), Some((tx, ty)), camp);
                    if ok {
                        let action_id = super::action_codec::encode_move_action((sx, sy), (tx, ty));
                        legal[action_id] = true;
                    }
                }
            }
        }
    }
    legal
}

fn movable_piece(cell: &Cell) -> Option<&Piece> {
    cell.monkey.as_ref().or(cell.piece.as_ref())
}
