use std::path::{Path, PathBuf};

use ndarray::{Array2, Array3, Axis};
use ort::{
    inputs,
    session::{Session, builder::GraphOptimizationLevel},
    value::TensorRef,
};

use crate::ai::action_codec::decode_action;
use crate::ai::model::{
    INPUT_CELL_FEATURES, INPUT_GLOBAL_FEATURES, INPUT_HISTORY_FEATURES, INPUT_HISTORY_MASK,
    INPUT_LEGAL_ACTION_MASK, OUTPUT_POLICY_LOGITS,
};
use crate::ai::state_encoder::{HISTORY_LEN, HistoryEntry, HistoryResult, StateEncoder};
use crate::game::board::Board;
use crate::game::piece::Camp;

use super::Action;

pub struct NeuralAIPlayer {
    camp: Camp,
    session: Session,
    encoder: StateEncoder,
    history: Vec<HistoryEntry>,
    model_path: PathBuf,
}

impl NeuralAIPlayer {
    pub fn new(camp: Camp, model_path: PathBuf) -> Result<Self, String> {
        if !model_path.exists() {
            return Err(format!("model not found: {}", model_path.display()));
        }

        let session = Session::builder()
            .map_err(|e| e.to_string())?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| e.to_string())?
            .commit_from_file(&model_path)
            .map_err(|e| e.to_string())?;

        Ok(Self {
            camp,
            session,
            encoder: StateEncoder::new(HISTORY_LEN),
            history: Vec::new(),
            model_path,
        })
    }

    pub fn choose_action(&mut self, board: &Board) -> Option<Action> {
        let encoded = self.encoder.encode(board, self.camp, self.camp, &self.history);

        let cell = Array3::from_shape_vec((1, 24, 36), encoded.cell_features).ok()?;
        let global = Array2::from_shape_vec((1, 16), encoded.global_features).ok()?;
        let history =
            Array3::from_shape_vec((1, HISTORY_LEN, 59), encoded.history_features).ok()?;
        let history_mask =
            Array2::from_shape_vec((1, HISTORY_LEN), encoded.history_mask).ok()?;
        let legal_mask = Array2::from_shape_vec((1, 600), encoded.legal_action_mask).ok()?;

        let outputs = self
            .session
            .run(inputs![
                INPUT_CELL_FEATURES => TensorRef::from_array_view(&cell).ok()?,
                INPUT_GLOBAL_FEATURES => TensorRef::from_array_view(&global).ok()?,
                INPUT_HISTORY_FEATURES => TensorRef::from_array_view(&history).ok()?,
                INPUT_HISTORY_MASK => TensorRef::from_array_view(&history_mask).ok()?,
                INPUT_LEGAL_ACTION_MASK => TensorRef::from_array_view(&legal_mask).ok()?
            ])
            .ok()?;

        let logits = outputs[OUTPUT_POLICY_LOGITS].try_extract_array::<f32>().ok()?;
        let logits = logits.index_axis(Axis(0), 0);
        let action_id = argmax(logits.iter().copied())?;
        let mut action = decode_action(action_id)?;
        action.score = logits[action_id];
        Some(action)
    }

    pub fn record_action(
        &mut self,
        board_before: &Board,
        board_after: &Board,
        action: &Action,
        actor: Camp,
        executed: bool,
    ) {
        let result = classify_result(board_before, board_after, executed);
        self.history.push(HistoryEntry {
            action_type: Some(action.action_type),
            self_pos: Some(action.self_pos),
            target_pos: action.target_pos,
            actor,
            result,
        });
        if self.history.len() > HISTORY_LEN {
            let drain = self.history.len() - HISTORY_LEN;
            self.history.drain(0..drain);
        }
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }
}

fn argmax(values: impl IntoIterator<Item = f32>) -> Option<usize> {
    let mut best_idx = None;
    let mut best_val = f32::NEG_INFINITY;
    for (idx, value) in values.into_iter().enumerate() {
        if value.is_finite() && value > best_val {
            best_val = value;
            best_idx = Some(idx);
        }
    }
    best_idx
}

fn classify_result(board_before: &Board, board_after: &Board, executed: bool) -> HistoryResult {
    if !executed {
        return HistoryResult::Invalid;
    }
    let before = board_before.count_pieces();
    let after = board_after.count_pieces();
    if before != after {
        return HistoryResult::Eat;
    }
    if board_before.cells != board_after.cells {
        return HistoryResult::Ok;
    }
    HistoryResult::Special
}
