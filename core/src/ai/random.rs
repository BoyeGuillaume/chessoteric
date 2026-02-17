use std::sync::Mutex;

use rand::prelude::*;

use crate::{
    ai::{Ai, AiLimit, AiType},
    board::Board,
    moves::Move,
};

use super::AiResult;

pub struct RandomAi {
    rng: Mutex<ThreadRng>,
    best_move: Mutex<Option<Move>>,
}

impl std::default::Default for RandomAi {
    fn default() -> Self {
        RandomAi {
            rng: Mutex::new(ThreadRng::default()),
            best_move: Mutex::new(None),
        }
    }
}

impl Ai for RandomAi {
    fn name(&self) -> &str {
        "Random_AI"
    }

    fn authors(&self) -> &[&str] {
        &["Guillaume Boyé"]
    }

    fn start(&self, board: &Board, _limits: AiLimit, print: bool) -> AiType {
        // For a random AI, we don't need to do any setup before generating a move
        let mut moves = Vec::new();
        let mut currently_in_check = false;
        crate::moves::generate_moves(board, &mut moves, &mut currently_in_check);

        if moves.is_empty() {
            *self.best_move.lock().unwrap() = None;
        } else {
            let random_index = self.rng.lock().unwrap().random_range(0..moves.len());
            *self.best_move.lock().unwrap() = Some(moves[random_index]);
        }

        if print && let Some(mv) = *self.best_move.lock().unwrap() {
            println!("bestmove {}", mv.uci());
        }

        AiType::Sync
    }

    fn stop(&self) -> Option<AiResult> {
        // For a random AI, we can return the best move immediately since there is no ongoing search
        self.best_move.lock().unwrap().map(|mv| AiResult {
            best_move: mv,
            pv: vec![mv],
            depth: 1,
            nodes: 1,
            score: 0.0,
        })
    }

    fn reset(&self) {
        *self.best_move.lock().unwrap() = None;
    }
}
