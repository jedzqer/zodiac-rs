pub mod board;
pub mod piece;

use serde::{Deserialize, Serialize};

use board::Board;
use piece::Camp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub board: Board,
    pub current_player: Camp,
    pub game_over: bool,
    pub winner: Option<Camp>,
    pub move_count: u32,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            board: Board::new(),
            current_player: Camp::Black,
            game_over: false,
            winner: None,
            move_count: 0,
        }
    }

    pub fn toggle_player(&mut self) {
        self.current_player = self.current_player.opponent();
    }

    pub fn check_winner(&mut self) {
        if self.board.has_hidden_piece() {
            return;
        }
        let (red, black) = self.board.count_pieces();
        if red == 0 {
            self.game_over = true;
            self.winner = Some(Camp::Black);
        } else if black == 0 {
            self.game_over = true;
            self.winner = Some(Camp::Red);
        }
    }
}
