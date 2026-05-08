use serde::{Deserialize, Serialize};

use crate::game::board::Board;
use crate::game::piece::Camp;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum ClientMessage {
    NewGame { mode: GameMode },
    Flip { x: usize, y: usize },
    Move { from_x: usize, from_y: usize, to_x: usize, to_y: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    Pvp,
    Pve,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum ServerMessage {
    GameStarted {
        board: Board,
        current_player: Camp,
        mode: GameMode,
    },
    BoardUpdate {
        board: Board,
        current_player: Camp,
        message: String,
    },
    GameOver {
        winner: Camp,
        board: Board,
    },
    Error {
        message: String,
    },
    AiThinking,
    AiAction {
        description: String,
    },
}
