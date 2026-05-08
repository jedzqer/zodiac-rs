use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::piece::{Camp, MoveResult, Piece, create_all_pieces};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub has_piece: bool,
    pub piece: Option<Piece>,
    pub revealed: bool,
    pub monkey: Option<Piece>,
}

impl Cell {
    pub fn empty() -> Self {
        Cell {
            has_piece: false,
            piece: None,
            revealed: false,
            monkey: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub cells: [[Cell; 4]; 6],
}

impl Board {
    pub fn new() -> Self {
        let mut pieces = create_all_pieces();
        let mut rng = rand::rng();
        pieces.shuffle(&mut rng);

        let mut cells: [[Cell; 4]; 6] = std::array::from_fn(|_| {
            std::array::from_fn(|_| Cell::empty())
        });

        for i in 0..6 {
            for j in 0..4 {
                let p = pieces[i * 4 + j].clone();
                cells[i][j] = Cell {
                    has_piece: true,
                    piece: Some(p),
                    revealed: false,
                    monkey: None,
                };
            }
        }

        Board { cells }
    }

    pub fn is_valid_position(x: i32, y: i32) -> bool {
        x >= 0 && x < 6 && y >= 0 && y < 4
    }

    pub fn count_pieces(&self) -> (u32, u32) {
        let mut red = 0u32;
        let mut black = 0u32;
        for x in 0..6 {
            for y in 0..4 {
                let cell = &self.cells[x][y];
                if let Some(p) = &cell.piece {
                    match p.camp {
                        Camp::Red => red += 1,
                        Camp::Black => black += 1,
                    }
                }
                if let Some(m) = &cell.monkey {
                    match m.camp {
                        Camp::Red => red += 1,
                        Camp::Black => black += 1,
                    }
                }
            }
        }
        (red, black)
    }

    pub fn has_hidden_piece(&self) -> bool {
        for x in 0..6 {
            for y in 0..4 {
                let cell = &self.cells[x][y];
                if cell.piece.is_some() && !cell.revealed && cell.monkey.is_none() {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_target_info(&self, pos: (usize, usize)) -> ((Option<u8>, bool), Option<Camp>) {
        let cell = &self.cells[pos.0][pos.1];
        match &cell.piece {
            None => ((None, cell.revealed), None),
            Some(p) => ((Some(p.lv), cell.revealed), Some(p.camp)),
        }
    }

    pub fn get_path_state(
        &self,
        self_pos: (usize, usize),
        target_pos: (usize, usize),
        dx: usize,
        dy: usize,
    ) -> (Option<(bool, bool)>, Option<(usize, usize)>) {
        if dx + dy == 2 {
            if dx == 0 {
                let mid_y = self_pos.1.min(target_pos.1) + 1;
                let cell = &self.cells[self_pos.0][mid_y];
                return (
                    Some((cell.has_piece, cell.revealed)),
                    Some((self_pos.0, mid_y)),
                );
            } else if dy == 0 {
                let mid_x = self_pos.0.min(target_pos.0) + 1;
                let cell = &self.cells[mid_x][self_pos.1];
                return (
                    Some((cell.has_piece, cell.revealed)),
                    Some((mid_x, self_pos.1)),
                );
            }
        }
        (None, None)
    }

    pub fn get_move_result(
        &self,
        self_pos: (usize, usize),
        target_pos: (usize, usize),
    ) -> Option<MoveResult> {
        let cell = &self.cells[self_pos.0][self_pos.1];
        let piece = if let Some(m) = &cell.monkey {
            m
        } else {
            cell.piece.as_ref()?
        };

        let (target_info, target_camp) = self.get_target_info(target_pos);
        let dx = self_pos.0.abs_diff(target_pos.0);
        let dy = self_pos.1.abs_diff(target_pos.1);
        let (path_state, _) = self.get_path_state(self_pos, target_pos, dx, dy);

        piece.is_valid_move(target_pos, target_info, self_pos, path_state, target_camp, dx, dy)
    }

    pub fn execute_move(&mut self, result: MoveResult, self_pos: (usize, usize), target_pos: (usize, usize)) {
        let (_, path_p_pos) = {
            let dx = self_pos.0.abs_diff(target_pos.0);
            let dy = self_pos.1.abs_diff(target_pos.1);
            self.get_path_state(self_pos, target_pos, dx, dy)
        };

        match result {
            MoveResult::Move => self.handle_move(self_pos, target_pos),
            MoveResult::Eat => self.handle_eat(self_pos, target_pos),
            MoveResult::Pong => self.handle_pong(self_pos, target_pos),
            MoveResult::Cow => {
                if let Some(path_pos) = path_p_pos {
                    self.cells[path_pos.0][path_pos.1].revealed = true;
                }
                self.handle_move(self_pos, target_pos);
            }
            MoveResult::Sheep => self.handle_sheep(self_pos, target_pos),
            MoveResult::Chicken => self.handle_chicken(self_pos),
            MoveResult::Dog => self.handle_dog(self_pos, target_pos),
            MoveResult::Monkey => self.handle_monkey(self_pos, target_pos),
            MoveResult::MonkeyPong => self.handle_monkey_pong(self_pos, target_pos),
            MoveResult::MonkeyEat => self.handle_monkey_eat(self_pos, target_pos),
            MoveResult::MonkeyMove => self.handle_monkey_move(self_pos, target_pos),
        }
    }

    fn handle_move(&mut self, self_pos: (usize, usize), target_pos: (usize, usize)) {
        let tmp = self.cells[self_pos.0][self_pos.1].clone();
        self.cells[self_pos.0][self_pos.1] = self.cells[target_pos.0][target_pos.1].clone();
        self.cells[target_pos.0][target_pos.1] = tmp;
    }

    fn handle_eat(&mut self, self_pos: (usize, usize), target_pos: (usize, usize)) {
        self.cells[target_pos.0][target_pos.1] = self.cells[self_pos.0][self_pos.1].clone();
        self.cells[self_pos.0][self_pos.1] = Cell::empty();
    }

    fn handle_pong(&mut self, self_pos: (usize, usize), target_pos: (usize, usize)) {
        self.cells[self_pos.0][self_pos.1] = Cell::empty();
        self.cells[target_pos.0][target_pos.1] = Cell::empty();
    }

    fn handle_sheep(&mut self, self_pos: (usize, usize), target_pos: (usize, usize)) {
        self.cells[self_pos.0][self_pos.1] = self.cells[target_pos.0][target_pos.1].clone();
        self.cells[target_pos.0][target_pos.1] = Cell::empty();
    }

    fn handle_chicken(&mut self, self_pos: (usize, usize)) {
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                let nx = self_pos.0 as i32 + dx;
                let ny = self_pos.1 as i32 + dy;
                if Self::is_valid_position(nx, ny) {
                    self.cells[nx as usize][ny as usize].revealed = true;
                }
            }
        }
    }

    fn handle_dog(&mut self, self_pos: (usize, usize), target_pos: (usize, usize)) {
        let new_pos = if target_pos.0 == self_pos.0 {
            if self_pos.1 > target_pos.1 {
                (target_pos.0, target_pos.1.wrapping_sub(1))
            } else {
                (target_pos.0, target_pos.1 + 1)
            }
        } else {
            if self_pos.0 > target_pos.0 {
                (target_pos.0.wrapping_sub(1), target_pos.1)
            } else {
                (target_pos.0 + 1, target_pos.1)
            }
        };

        if Self::is_valid_position(new_pos.0 as i32, new_pos.1 as i32)
            && !self.cells[new_pos.0][new_pos.1].has_piece
        {
            self.cells[new_pos.0][new_pos.1] = self.cells[target_pos.0][target_pos.1].clone();
            self.cells[target_pos.0][target_pos.1] = Cell::empty();
        }
    }

    fn handle_monkey(&mut self, self_pos: (usize, usize), target_pos: (usize, usize)) {
        if self.cells[target_pos.0][target_pos.1].monkey.is_none() {
            if self.cells[self_pos.0][self_pos.1].monkey.is_some() {
                self.cells[target_pos.0][target_pos.1].monkey =
                    self.cells[self_pos.0][self_pos.1].monkey.take();
            } else {
                self.cells[target_pos.0][target_pos.1].monkey =
                    self.cells[self_pos.0][self_pos.1].piece.take();
                self.cells[self_pos.0][self_pos.1].has_piece = false;
            }
        } else {
            self.cells[self_pos.0][self_pos.1].monkey = None;
            self.cells[target_pos.0][target_pos.1].monkey = None;
        }
    }

    fn handle_monkey_pong(&mut self, self_pos: (usize, usize), target_pos: (usize, usize)) {
        if self.cells[self_pos.0][self_pos.1].monkey.is_some() {
            self.cells[self_pos.0][self_pos.1].monkey = None;
            self.cells[target_pos.0][target_pos.1] = Cell::empty();
        } else if self.cells[target_pos.0][target_pos.1].monkey.is_some() {
            self.cells[self_pos.0][self_pos.1] = Cell::empty();
            self.cells[target_pos.0][target_pos.1].monkey = None;
        } else {
            self.cells[self_pos.0][self_pos.1] = Cell::empty();
            self.cells[target_pos.0][target_pos.1] = Cell::empty();
        }
    }

    fn handle_monkey_eat(&mut self, self_pos: (usize, usize), target_pos: (usize, usize)) {
        if self.cells[self_pos.0][self_pos.1].monkey.is_some() {
            let monkey = self.cells[self_pos.0][self_pos.1].monkey.take().unwrap();
            self.cells[target_pos.0][target_pos.1].piece = Some(monkey);
            self.cells[target_pos.0][target_pos.1].has_piece = true;
            self.cells[target_pos.0][target_pos.1].revealed = true;
            self.cells[target_pos.0][target_pos.1].monkey = None;
        } else {
            self.cells[target_pos.0][target_pos.1] = self.cells[self_pos.0][self_pos.1].clone();
            self.cells[self_pos.0][self_pos.1] = Cell::empty();
        }
    }

    fn handle_monkey_move(&mut self, self_pos: (usize, usize), target_pos: (usize, usize)) {
        if self.cells[self_pos.0][self_pos.1].monkey.is_some() {
            let monkey = self.cells[self_pos.0][self_pos.1].monkey.take().unwrap();
            self.cells[target_pos.0][target_pos.1] = Cell {
                has_piece: true,
                piece: Some(monkey),
                revealed: true,
                monkey: None,
            };
            self.cells[self_pos.0][self_pos.1].monkey = None;
        } else {
            self.handle_move(self_pos, target_pos);
        }
    }

    pub fn dispose_piece(
        &mut self,
        self_pos: (usize, usize),
        target_pos: Option<(usize, usize)>,
        gamer_camp: Camp,
    ) -> (bool, Option<MoveResult>) {
        let has_piece = self.cells[self_pos.0][self_pos.1].has_piece;
        let revealed = self.cells[self_pos.0][self_pos.1].revealed;
        let has_monkey = self.cells[self_pos.0][self_pos.1].monkey.is_some();
        let piece_camp = self.cells[self_pos.0][self_pos.1].piece.as_ref().map(|p| p.camp);

        if !has_piece {
            return (true, None);
        }

        if !revealed && !has_monkey {
            self.cells[self_pos.0][self_pos.1].revealed = true;
            return (true, Some(MoveResult::Move));
        }

        if let Some(camp) = piece_camp {
            if gamer_camp != camp {
                return (true, None);
            }
        }

        let target_pos = match target_pos {
            Some(tp) => tp,
            None => return (false, None),
        };

        let result = self.get_move_result(self_pos, target_pos);
        match result {
            Some(r) => {
                self.execute_move(r, self_pos, target_pos);
                (true, Some(r))
            }
            None => (false, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::piece::{Camp, Piece};

    fn make_cell(piece: Option<Piece>, revealed: bool) -> Cell {
        Cell {
            has_piece: piece.is_some(),
            piece,
            revealed,
            monkey: None,
        }
    }

    fn test_board() -> Board {
        Board::new()
    }

    #[test]
    fn test_new_board_has_24_pieces() {
        let board = test_board();
        let (red, black) = board.count_pieces();
        assert_eq!(red, 12);
        assert_eq!(black, 12);
    }

    #[test]
    fn test_new_board_all_hidden() {
        let board = test_board();
        for x in 0..6 {
            for y in 0..4 {
                assert!(board.cells[x][y].piece.is_some());
                assert!(!board.cells[x][y].revealed);
                assert!(board.cells[x][y].monkey.is_none());
            }
        }
    }

    #[test]
    fn test_has_hidden_piece_initial() {
        let board = test_board();
        assert!(board.has_hidden_piece());
    }

    #[test]
    fn test_dispose_piece_flip() {
        let mut board = test_board();
        let (ok, _result) = board.dispose_piece((0, 0), None, Camp::Black);
        assert!(ok);
        assert!(board.cells[0][0].revealed);
    }

    #[test]
    fn test_dispose_piece_wrong_camp() {
        let mut board = test_board();
        // Reveal a piece first
        board.cells[0][0].revealed = true;
        // Try to move it with wrong camp
        let piece_camp = board.cells[0][0].piece.as_ref().unwrap().camp;
        let wrong_camp = piece_camp.opponent();
        let (ok, result) = board.dispose_piece((0, 0), Some((1, 0)), wrong_camp);
        assert!(ok);
        assert!(result.is_none());
    }

    #[test]
    fn test_mouse_move_one_step() {
        let mut board = Board { cells: std::array::from_fn(|_| std::array::from_fn(|_| Cell::empty())) };
        let mouse = Piece { name: "鼠".to_string(), lv: 1, camp: Camp::Red };
        board.cells[2][2] = make_cell(Some(mouse.clone()), true);
        board.cells[3][2] = make_cell(None, false);

        let result = board.get_move_result((2, 2), (3, 2));
        assert_eq!(result, Some(MoveResult::Move));
    }

    #[test]
    fn test_mouse_cannot_move_two_steps() {
        let mut board = Board { cells: std::array::from_fn(|_| std::array::from_fn(|_| Cell::empty())) };
        let mouse = Piece { name: "鼠".to_string(), lv: 1, camp: Camp::Red };
        board.cells[0][0] = make_cell(Some(mouse), true);
        board.cells[2][0] = make_cell(None, false);

        let result = board.get_move_result((0, 0), (2, 0));
        assert_eq!(result, None);
    }

    #[test]
    fn test_dragon_can_fly() {
        let mut board = Board { cells: std::array::from_fn(|_| std::array::from_fn(|_| Cell::empty())) };
        let dragon = Piece { name: "龙".to_string(), lv: 5, camp: Camp::Red };
        board.cells[0][0] = make_cell(Some(dragon), true);
        board.cells[5][3] = make_cell(None, false);

        let result = board.get_move_result((0, 0), (5, 3));
        assert_eq!(result, Some(MoveResult::Move));
    }

    #[test]
    fn test_snake_diagonal() {
        let mut board = Board { cells: std::array::from_fn(|_| std::array::from_fn(|_| Cell::empty())) };
        let snake = Piece { name: "蛇".to_string(), lv: 6, camp: Camp::Red };
        board.cells[2][2] = make_cell(Some(snake), true);
        board.cells[3][3] = make_cell(None, false);

        let result = board.get_move_result((2, 2), (3, 3));
        assert_eq!(result, Some(MoveResult::Move));
    }

    #[test]
    fn test_lower_eats_higher() {
        let mut board = Board { cells: std::array::from_fn(|_| std::array::from_fn(|_| Cell::empty())) };
        let mouse = Piece { name: "鼠".to_string(), lv: 1, camp: Camp::Red };
        let dragon = Piece { name: "龙".to_string(), lv: 5, camp: Camp::Black };
        board.cells[2][2] = make_cell(Some(mouse), true);
        board.cells[3][2] = make_cell(Some(dragon), true);

        let result = board.get_move_result((2, 2), (3, 2));
        assert_eq!(result, Some(MoveResult::Eat));
    }

    #[test]
    fn test_same_level_pong() {
        let mut board = Board { cells: std::array::from_fn(|_| std::array::from_fn(|_| Cell::empty())) };
        let m1 = Piece { name: "鼠".to_string(), lv: 1, camp: Camp::Red };
        let m2 = Piece { name: "鼠".to_string(), lv: 1, camp: Camp::Black };
        board.cells[2][2] = make_cell(Some(m1), true);
        board.cells[3][2] = make_cell(Some(m2), true);

        let result = board.get_move_result((2, 2), (3, 2));
        assert_eq!(result, Some(MoveResult::Pong));
    }

    #[test]
    fn test_chicken_self_reveal() {
        let mut board = Board { cells: std::array::from_fn(|_| std::array::from_fn(|_| Cell::empty())) };
        let chicken = Piece { name: "鸡".to_string(), lv: 10, camp: Camp::Red };
        board.cells[2][2] = make_cell(Some(chicken), true);

        let result = board.get_move_result((2, 2), (2, 2));
        assert_eq!(result, Some(MoveResult::Chicken));
    }

    #[test]
    fn test_game_state_winner_detection() {
        let mut gs = crate::game::GameState::new();
        // Remove all black pieces
        for x in 0..6 {
            for y in 0..4 {
                if let Some(p) = &gs.board.cells[x][y].piece {
                    if p.camp == Camp::Black {
                        gs.board.cells[x][y] = Cell::empty();
                    }
                }
                gs.board.cells[x][y].revealed = true;
            }
        }
        gs.check_winner();
        assert!(gs.game_over);
        assert_eq!(gs.winner, Some(Camp::Red));
    }
}
