pub mod action_codec;
pub mod heuristic;
pub mod model;
pub mod neural;
pub mod state_encoder;

use std::path::PathBuf;

use crate::game::board::Board;
use crate::game::piece::Camp;

pub use heuristic::AIPlayer as HeuristicAIPlayer;
pub use neural::NeuralAIPlayer;

#[derive(Debug, Clone)]
pub struct Action {
    pub action_type: ActionType,
    pub self_pos: (usize, usize),
    pub target_pos: Option<(usize, usize)>,
    pub score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Flip,
    Move,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Heuristic,
    Neural,
}

impl BackendKind {
    pub fn from_env() -> Self {
        match std::env::var("ZODIAC_AI_BACKEND")
            .unwrap_or_else(|_| "heuristic".to_string())
            .to_lowercase()
            .as_str()
        {
            "neural" | "onnx" | "model" => Self::Neural,
            _ => Self::Heuristic,
        }
    }
}

pub enum AIPlayer {
    Heuristic(HeuristicAIPlayer),
    Neural(NeuralAIPlayer),
}

impl AIPlayer {
    pub fn new(camp: Camp) -> Self {
        match BackendKind::from_env() {
            BackendKind::Heuristic => Self::Heuristic(HeuristicAIPlayer::new(camp)),
            BackendKind::Neural => {
                let path = std::env::var("ZODIAC_ONNX_MODEL")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("models/zooformer.onnx"));
                match NeuralAIPlayer::new(camp, path) {
                    Ok(player) => Self::Neural(player),
                    Err(err) => {
                        tracing::warn!("Failed to load neural model, fallback to heuristic AI: {}", err);
                        Self::Heuristic(HeuristicAIPlayer::new(camp))
                    }
                }
            }
        }
    }

    pub fn backend_kind(&self) -> BackendKind {
        match self {
            Self::Heuristic(_) => BackendKind::Heuristic,
            Self::Neural(_) => BackendKind::Neural,
        }
    }

    pub fn choose_action(&mut self, board: &Board) -> Option<Action> {
        match self {
            Self::Heuristic(ai) => ai.choose_action(board),
            Self::Neural(ai) => ai.choose_action(board),
        }
    }

    pub fn record_action(
        &mut self,
        board_before: &Board,
        board_after: &Board,
        action: &Action,
        actor: Camp,
        executed: bool,
    ) {
        if let Self::Neural(ai) = self {
            ai.record_action(board_before, board_after, action, actor, executed);
        }
    }
}
