use std::cmp::Ordering;
use std::collections::HashMap;

use rand::Rng;

use crate::ai::{Action, ActionType};
use crate::game::board::Board;
use crate::game::piece::{Camp, Piece};

const SEARCH_DEPTH: u8 = 3;
const WIN_SCORE: f64 = 1_000_000.0;

pub struct AIPlayer {
    pub camp: Camp,
    base_piece_scores: HashMap<u8, f64>,
    hidden_value_factor: f64,
    flip_material_factor: f64,
}

impl AIPlayer {
    pub fn new(camp: Camp) -> Self {
        let mut base_piece_scores = HashMap::new();
        base_piece_scores.insert(1, 20.0);
        base_piece_scores.insert(2, 12.0);
        base_piece_scores.insert(3, 11.0);
        base_piece_scores.insert(4, 10.0);
        base_piece_scores.insert(5, 9.0);
        base_piece_scores.insert(6, 8.0);
        base_piece_scores.insert(7, 7.0);
        base_piece_scores.insert(8, 6.0);
        base_piece_scores.insert(9, 5.0);
        base_piece_scores.insert(10, 4.0);
        base_piece_scores.insert(11, 3.0);
        base_piece_scores.insert(12, 1.0);

        AIPlayer {
            camp,
            base_piece_scores,
            hidden_value_factor: 0.45,
            flip_material_factor: 0.70,
        }
    }

    pub fn choose_action(&self, board: &Board) -> Option<Action> {
        let mut rng = rand::rng();
        let mut root_actions = self.generate_actions(board, self.camp);
        if root_actions.is_empty() {
            return None;
        }

        root_actions.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
        });

        let mut best_score = f64::NEG_INFINITY;
        let mut best_actions = Vec::new();
        let mut alpha = f64::NEG_INFINITY;
        let beta = f64::INFINITY;

        for action in root_actions {
            let mut sim_board = board.clone();
            let executed = self.apply_action(&mut sim_board, self.camp, &action);
            if !executed {
                continue;
            }

            let score = self.minimax(
                &sim_board,
                SEARCH_DEPTH.saturating_sub(1),
                self.camp.opponent(),
                alpha,
                beta,
            );

            let mut scored_action = action;
            scored_action.score = score as f32;

            if score > best_score + 1e-6 {
                best_score = score;
                best_actions.clear();
                best_actions.push(scored_action);
            } else if (score - best_score).abs() <= 1e-6 {
                best_actions.push(scored_action);
            }

            alpha = alpha.max(score);
        }

        if best_actions.is_empty() {
            return None;
        }

        let idx = rng.random_range(0..best_actions.len());
        Some(best_actions.swap_remove(idx))
    }

    fn minimax(
        &self,
        board: &Board,
        depth: u8,
        active_camp: Camp,
        mut alpha: f64,
        mut beta: f64,
    ) -> f64 {
        if let Some(score) = self.terminal_score(board, depth) {
            return score;
        }

        if depth == 0 {
            return self.evaluate_board(board);
        }

        let mut actions = self.generate_actions(board, active_camp);
        if actions.is_empty() {
            return self.evaluate_board(board);
        }

        actions.sort_by(|a, b| {
            if active_camp == self.camp {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(Ordering::Equal)
            } else {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(Ordering::Equal)
            }
        });

        if active_camp == self.camp {
            let mut value = f64::NEG_INFINITY;
            for action in actions {
                let mut sim_board = board.clone();
                if !self.apply_action(&mut sim_board, active_camp, &action) {
                    continue;
                }
                let score = self.minimax(
                    &sim_board,
                    depth - 1,
                    active_camp.opponent(),
                    alpha,
                    beta,
                );
                value = value.max(score);
                alpha = alpha.max(value);
                if alpha >= beta {
                    break;
                }
            }
            value
        } else {
            let mut value = f64::INFINITY;
            for action in actions {
                let mut sim_board = board.clone();
                if !self.apply_action(&mut sim_board, active_camp, &action) {
                    continue;
                }
                let score = self.minimax(
                    &sim_board,
                    depth - 1,
                    active_camp.opponent(),
                    alpha,
                    beta,
                );
                value = value.min(score);
                beta = beta.min(value);
                if alpha >= beta {
                    break;
                }
            }
            value
        }
    }

    fn generate_actions(&self, board: &Board, camp: Camp) -> Vec<Action> {
        let mut rng = rand::rng();
        let mut actions = Vec::new();

        self.collect_move_actions(board, camp, &mut actions, &mut rng);
        self.collect_flip_actions(board, camp, &mut actions, &mut rng);

        actions
    }

    fn collect_move_actions(
        &self,
        board: &Board,
        camp: Camp,
        actions: &mut Vec<Action>,
        rng: &mut impl Rng,
    ) {
        for sx in 0..6 {
            for sy in 0..4 {
                let cell = &board.cells[sx][sy];
                let mover_camp = if let Some(monkey) = &cell.monkey {
                    monkey.camp
                } else if let Some(piece) = &cell.piece {
                    piece.camp
                } else {
                    continue;
                };

                if mover_camp != camp {
                    continue;
                }

                if !cell.revealed && cell.monkey.is_none() {
                    continue;
                }

                for tx in 0..6 {
                    for ty in 0..4 {
                        let mut sim_board = board.clone();
                        let (ok, _) = sim_board.dispose_piece((sx, sy), Some((tx, ty)), camp);
                        if !ok {
                            continue;
                        }

                        let score = self.evaluate_transition(
                            board,
                            &sim_board,
                            camp,
                            ActionType::Move,
                            (sx, sy),
                            Some((tx, ty)),
                            rng,
                        );

                        actions.push(Action {
                            action_type: ActionType::Move,
                            self_pos: (sx, sy),
                            target_pos: Some((tx, ty)),
                            score: score as f32,
                        });
                    }
                }
            }
        }
    }

    fn collect_flip_actions(
        &self,
        board: &Board,
        camp: Camp,
        actions: &mut Vec<Action>,
        rng: &mut impl Rng,
    ) {
        for x in 0..6 {
            for y in 0..4 {
                let cell = &board.cells[x][y];
                if cell.piece.is_none() || cell.revealed || cell.monkey.is_some() {
                    continue;
                }

                let mut sim_board = board.clone();
                let (ok, _) = sim_board.dispose_piece((x, y), None, camp);
                if !ok {
                    continue;
                }

                let score = self.evaluate_transition(
                    board,
                    &sim_board,
                    camp,
                    ActionType::Flip,
                    (x, y),
                    None,
                    rng,
                );

                actions.push(Action {
                    action_type: ActionType::Flip,
                    self_pos: (x, y),
                    target_pos: None,
                    score: score as f32,
                });
            }
        }
    }

    fn apply_action(&self, board: &mut Board, camp: Camp, action: &Action) -> bool {
        match action.action_type {
            ActionType::Flip => {
                let (ok, _) = board.dispose_piece(action.self_pos, None, camp);
                ok
            }
            ActionType::Move => {
                let Some(target_pos) = action.target_pos else {
                    return false;
                };
                let (ok, _) = board.dispose_piece(action.self_pos, Some(target_pos), camp);
                ok
            }
        }
    }

    fn terminal_score(&self, board: &Board, depth: u8) -> Option<f64> {
        if board.has_hidden_piece() {
            return None;
        }

        let (red, black) = board.count_pieces();
        if red == 0 {
            return Some(if self.camp == Camp::Black {
                WIN_SCORE + depth as f64
            } else {
                -WIN_SCORE - depth as f64
            });
        }
        if black == 0 {
            return Some(if self.camp == Camp::Red {
                WIN_SCORE + depth as f64
            } else {
                -WIN_SCORE - depth as f64
            });
        }
        None
    }

    fn evaluate_transition(
        &self,
        base_board: &Board,
        new_board: &Board,
        camp: Camp,
        action_type: ActionType,
        self_pos: (usize, usize),
        target_pos: Option<(usize, usize)>,
        rng: &mut impl Rng,
    ) -> f64 {
        let (own_before, opp_before) = self.count_pieces_for(base_board, camp);
        let (own_after, opp_after) = self.count_pieces_for(new_board, camp);
        let board_score_before = self.board_score_for(base_board, camp);
        let board_score_after = self.board_score_for(new_board, camp);

        let mut score = (opp_before as f64 - opp_after as f64) * 120.0;
        score -= (own_before as f64 - own_after as f64) * 100.0;
        score += (board_score_after - board_score_before) * 8.0;

        match action_type {
            ActionType::Move => {
                score += 10.0;
                if let Some(tp) = target_pos {
                    let target_cell = &base_board.cells[tp.0][tp.1];
                    if let Some(tp_piece) = &target_cell.piece {
                        if target_cell.revealed && tp_piece.camp != camp {
                            score += 25.0;
                        }
                    }
                    if self.is_central_square(tp) {
                        score += 1.5;
                    }
                }
            }
            ActionType::Flip => {
                score += self.flip_position_bonus(base_board, self_pos);
                score += self.flip_material_swing(base_board, new_board, camp, self_pos);
            }
        }

        score += rng.random_range(0.0..0.8);
        score
    }

    fn evaluate_board(&self, board: &Board) -> f64 {
        if let Some(score) = self.terminal_score(board, 0) {
            return score;
        }

        let mut score = self.board_score_for(board, self.camp) * 10.0;

        let (own_count, opp_count) = self.count_pieces_for(board, self.camp);
        score += (own_count as f64 - opp_count as f64) * 25.0;

        score += self.mobility_score(board, self.camp) * 2.0;
        score -= self.mobility_score(board, self.camp.opponent()) * 1.8;

        score += self.revealed_presence_score(board, self.camp) * 3.0;
        score -= self.revealed_presence_score(board, self.camp.opponent()) * 2.5;

        score
    }

    fn mobility_score(&self, board: &Board, camp: Camp) -> f64 {
        self.generate_actions(board, camp)
            .into_iter()
            .map(|action| match action.action_type {
                ActionType::Move => 1.0,
                ActionType::Flip => 0.35,
            })
            .sum()
    }

    fn revealed_presence_score(&self, board: &Board, camp: Camp) -> f64 {
        let mut score = 0.0;
        for x in 0..6 {
            for y in 0..4 {
                let cell = &board.cells[x][y];
                if let Some(piece) = &cell.piece {
                    if piece.camp == camp && cell.revealed {
                        score += 1.0;
                        if self.is_central_square((x, y)) {
                            score += 0.5;
                        }
                    }
                }
                if let Some(monkey) = &cell.monkey {
                    if monkey.camp == camp {
                        score += 1.5;
                    }
                }
            }
        }
        score
    }

    fn is_central_square(&self, pos: (usize, usize)) -> bool {
        matches!(pos, (2, 1) | (2, 2) | (3, 1) | (3, 2))
    }

    fn count_pieces_for(&self, board: &Board, camp: Camp) -> (u32, u32) {
        let mut own = 0u32;
        let mut opp = 0u32;
        for x in 0..6 {
            for y in 0..4 {
                let cell = &board.cells[x][y];
                if let Some(p) = &cell.piece {
                    if p.camp == camp {
                        own += 1;
                    } else {
                        opp += 1;
                    }
                }
                if let Some(m) = &cell.monkey {
                    if m.camp == camp {
                        own += 1;
                    } else {
                        opp += 1;
                    }
                }
            }
        }
        (own, opp)
    }

    fn board_score_for(&self, board: &Board, camp: Camp) -> f64 {
        let mut score = 0.0;
        for x in 0..6 {
            for y in 0..4 {
                let cell = &board.cells[x][y];
                if let Some(p) = &cell.piece {
                    let v = self.piece_score(board, p, cell.revealed, camp);
                    score += if p.camp == camp { v } else { -v };
                }
                if let Some(m) = &cell.monkey {
                    let v = self.piece_score(board, m, true, camp);
                    score += if m.camp == camp { v } else { -v };
                }
            }
        }
        score
    }

    fn piece_score(&self, board: &Board, piece: &Piece, revealed: bool, perspective: Camp) -> f64 {
        let mut value = self.base_piece_scores.get(&piece.lv).copied().unwrap_or(0.0);

        if piece.lv == 1 && self.side_has_level(board, perspective.opponent(), 12) {
            value -= 5.0;
        } else if piece.lv == 12 && self.side_has_level(board, perspective.opponent(), 1) {
            value += 10.0;
        }

        if !revealed {
            value *= self.hidden_value_factor;
        }

        value
    }

    fn side_has_level(&self, board: &Board, camp: Camp, level: u8) -> bool {
        for x in 0..6 {
            for y in 0..4 {
                let cell = &board.cells[x][y];
                if let Some(p) = &cell.piece {
                    if p.camp == camp && p.lv == level {
                        return true;
                    }
                }
                if let Some(m) = &cell.monkey {
                    if m.camp == camp && m.lv == level {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn flip_material_swing(
        &self,
        base_board: &Board,
        new_board: &Board,
        perspective: Camp,
        self_pos: (usize, usize),
    ) -> f64 {
        let before_piece = match &base_board.cells[self_pos.0][self_pos.1].piece {
            Some(p) => p,
            None => return 0.0,
        };
        let after_piece = match &new_board.cells[self_pos.0][self_pos.1].piece {
            Some(p) => p,
            None => return 0.0,
        };

        let before_value = self.piece_score(base_board, before_piece, false, perspective);
        let after_value = self.piece_score(new_board, after_piece, true, perspective);
        let delta = after_value - before_value;

        if after_piece.camp == perspective {
            delta * self.flip_material_factor
        } else {
            -delta * self.flip_material_factor
        }
    }

    fn flip_position_bonus(&self, board: &Board, self_pos: (usize, usize)) -> f64 {
        let x = self_pos.0 as f64;
        let y = self_pos.1 as f64;
        let distance = (x - 2.5).abs() + (y - 1.5).abs();
        let mut bonus = (4.5 - distance).max(0.0);

        for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if Board::is_valid_position(nx, ny) {
                let cell = &board.cells[nx as usize][ny as usize];
                if cell.piece.is_some() && cell.revealed {
                    bonus += 0.5;
                }
            }
        }

        bonus
    }
}
