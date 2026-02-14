use crate::{board::Board, moves::Move};

pub mod random;
pub mod simple;

pub trait Ai {
    fn best_move(&mut self, board: &Board) -> Option<(Move, f32)>;
}
