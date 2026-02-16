use crate::{board::Board, moves::Move};

pub mod random;
pub mod simple;

pub trait Ai {
    fn best_move(&mut self, board: &Board, timeout: std::time::Duration) -> Option<(Move, f32)>;
}

pub fn get_ai(name: &str) -> Option<Box<dyn Ai>> {
    match name {
        "simple" => Some(Box::new(simple::SimpleAi::default())),
        "random" => Some(Box::new(random::RandomAi::default())),
        _ => None,
    }
}
