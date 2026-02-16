use crate::{board::Board, moves::Move};

pub mod random;
pub mod simple;

#[derive(Debug, Clone)]
pub struct AiResult {
    pub best_move: Move,
    pub pv: Vec<Move>,
    pub depth: u16,
    pub nodes: usize,
    pub score: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AiType {
    Async,
    Sync,
}

#[derive(Debug, Clone, Default)]
pub struct AiLimit {
    pub movetime: Option<std::time::Duration>,
    pub depth: Option<u16>,
}

pub trait Ai {
    fn name(&self) -> &str;
    fn authors(&self) -> &[&str];

    fn start(&self, board: &Board, limits: AiLimit) -> AiType;
    fn stop(&self) -> Option<AiResult>;
    fn reset(&self);
}

pub fn get_ai(name: &str) -> Option<Box<dyn Ai>> {
    match name {
        "simple" => Some(Box::new(simple::SimpleAi::default())),
        "random" => Some(Box::new(random::RandomAi::default())),
        _ => None,
    }
}
