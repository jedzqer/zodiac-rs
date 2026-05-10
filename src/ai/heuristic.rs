use std::collections::HashMap;

use rand::Rng;

use crate::ai::{Action, ActionType};
use crate::game::board::Board;
use crate::game::piece::{Camp, Piece};

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
        let mut actions = Vec::new();
        let mut rng = rand::rng();

        self.collect_move_actions(board, &mut actions, &mut rng);
        self.collect_flip_actions(board, &mut actions, &mut rng);

        if actions.is_empty() {
            return None;
        }

        let best_score = actions
            .iter()
            .map(|a| a.score)
            .fold(f32::NEG_INFINITY, f32::max);
        let best_actions: Vec<_> = actions.into_iter().filter(|a| a.score == best_score).collect();
        let idx = rng.random_range(0..best_actions.len());
        Some(best_actions.into_iter().nth(idx).unwrap())
    }

    fn collect_move_actions(&self, board: &Board, actions: &mut Vec<Action>, rng: &mut impl Rng) {
        for sx in 0..6 {
            for sy in 0..4 {
                let cell = &board.cells[sx][sy];
                match &cell.piece {
                    Some(p) if p.camp == self.camp => {}
                    _ => continue,
                }
                if !cell.revealed && cell.monkey.is_none() {
                    continue;
                }

                for tx in 0..6 {
                    for ty in 0..4 {
                        let mut sim_board = board.clone();
                        let (ok, _) = sim_board.dispose_piece(
                            (sx, sy),
                            Some((tx, ty)),
                            self.camp,
                        );
                        if !ok {
                            continue;
                        }

                        let score = self.evaluate_action(
                            board,
                            &sim_board,
                            "move",
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

    fn collect_flip_actions(&self, board: &Board, actions: &mut Vec<Action>, rng: &mut impl Rng) {
        for x in 0..6 {
            for y in 0..4 {
                let cell = &board.cells[x][y];
                if cell.piece.is_none() {
                    continue;
                }
                if cell.revealed || cell.monkey.is_some() {
                    continue;
                }

                let mut sim_board = board.clone();
                let (ok, _) = sim_board.dispose_piece((x, y), None, self.camp);
                if !ok {
                    continue;
                }

                let score = self.evaluate_action(
                    board,
                    &sim_board,
                    "flip",
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

    fn evaluate_action(
        &self,
        base_board: &Board,
        new_board: &Board,
        action_type: &str,
        self_pos: (usize, usize),
        target_pos: Option<(usize, usize)>,
        rng: &mut impl Rng,
    ) -> f64 {
        let (own_before, opp_before) = self.count_pieces(base_board);
        let (own_after, opp_after) = self.count_pieces(new_board);
        let board_score_before = self.board_score(base_board);
        let board_score_after = self.board_score(new_board);

        let mut score = (opp_before as f64 - opp_after as f64) * 120.0;
        score -= (own_before as f64 - own_after as f64) * 100.0;
        score += (board_score_after - board_score_before) * 8.0;

        if action_type == "move" {
            score += 10.0;
            if let Some(tp) = target_pos {
                let target_cell = &base_board.cells[tp.0][tp.1];
                if let Some(tp_piece) = &target_cell.piece {
                    if target_cell.revealed && tp_piece.camp != self.camp {
                        score += 25.0;
                    }
                }
            }
        } else {
            score += self.flip_position_bonus(base_board, self_pos);
            score += self.flip_material_swing(base_board, new_board, self_pos);
        }

        score += rng.random_range(0.0..0.8);
        score
    }

    fn count_pieces(&self, board: &Board) -> (u32, u32) {
        let mut own = 0u32;
        let mut opp = 0u32;
        for x in 0..6 {
            for y in 0..4 {
                let cell = &board.cells[x][y];
                if let Some(p) = &cell.piece {
                    if p.camp == self.camp {
                        own += 1;
                    } else {
                        opp += 1;
                    }
                }
                if let Some(m) = &cell.monkey {
                    if m.camp == self.camp {
                        own += 1;
                    } else {
                        opp += 1;
                    }
                }
            }
        }
        (own, opp)
    }

    fn board_score(&self, board: &Board) -> f64 {
        let mut score = 0.0;
        for x in 0..6 {
            for y in 0..4 {
                let cell = &board.cells[x][y];
                if let Some(p) = &cell.piece {
                    let v = self.piece_score(board, p, cell.revealed);
                    score += if p.camp == self.camp { v } else { -v };
                }
                if let Some(m) = &cell.monkey {
                    let v = self.piece_score(board, m, true);
                    score += if m.camp == self.camp { v } else { -v };
                }
            }
        }
        score
    }

    fn piece_score(&self, board: &Board, piece: &Piece, revealed: bool) -> f64 {
        let mut value = self.base_piece_scores.get(&piece.lv).copied().unwrap_or(0.0);

        if piece.lv == 1 && self.side_has_level(board, self.camp.opponent(), 12) {
            value -= 5.0;
        } else if piece.lv == 12 && self.side_has_level(board, self.camp.opponent(), 1) {
            value += 4.0;
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

    fn flip_material_swing(&self, base_board: &Board, new_board: &Board, self_pos: (usize, usize)) -> f64 {
        let before_piece = match &base_board.cells[self_pos.0][self_pos.1].piece {
            Some(p) => p,
            None => return 0.0,
        };
        let after_piece = match &new_board.cells[self_pos.0][self_pos.1].piece {
            Some(p) => p,
            None => return 0.0,
        };

        let before_value = self.piece_score(base_board, before_piece, false);
        let after_value = self.piece_score(new_board, after_piece, true);
        let delta = after_value - before_value;

        if after_piece.camp == self.camp {
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
