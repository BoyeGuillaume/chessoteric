use rand::prelude::*;

use crate::ai::Ai;

pub struct RandomAi {
    rng: ThreadRng,
}

impl std::default::Default for RandomAi {
    fn default() -> Self {
        RandomAi { rng: rand::rng() }
    }
}

impl Ai for RandomAi {
    fn best_move(
        &mut self,
        board: &crate::board::Board,
        _timeout: std::time::Duration,
    ) -> Option<(crate::moves::Move, f32)> {
        // Generate moves
        let mut moves = Vec::new();
        let mut currently_in_check = false;
        crate::moves::generate_moves(board, &mut moves, &mut currently_in_check);

        if moves.is_empty() {
            return None;
        }

        // Pick a random move
        let random_index = self.rng.random_range(0..moves.len());
        Some((moves[random_index], 0.0))
    }
}
