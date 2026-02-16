use std::usize;

use bitflags::bitflags;
use strum::{EnumIs, EnumTryAs};

use crate::{
    ai::Ai,
    board::{Board, Color, Piece},
    moves::{Move, generate_moves},
    tree::{Tree, TreeNodeRef, TreeRef},
};

pub struct SimpleAi {}

impl std::default::Default for SimpleAi {
    fn default() -> Self {
        SimpleAi {}
    }
}

fn simple_evaluation(board: &crate::board::Board) -> f32 {
    // A very simple evaluation function that just counts material
    let mut score: f32 = 0.0;
    for piece in Piece::colorless_iter() {
        let bitboard = *board.get(piece);

        let count_total = bitboard.count_ones();
        let count_white = (bitboard & board.white).count_ones();
        let count_black = count_total - count_white;

        let value = match piece {
            Piece::Queen => 9.0,
            Piece::Rook => 5.0,
            Piece::Bishop => 3.0,
            Piece::Knight => 3.0,
            Piece::Pawn => 1.0,
            _ => 0.0,
        };
        score += value * (count_white as f32 - count_black as f32);
    }

    score
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct TerminalFlags: u8 {
        const CHECKMATE_WHITE_WIN = 1 << 0;
        const CHECKMATE_BLACK_WIN = 1 << 1;
        const STALEMATE = 1 << 2;
        const ANY_TERMINAL = Self::CHECKMATE_WHITE_WIN.bits() | Self::CHECKMATE_BLACK_WIN.bits() | Self::STALEMATE.bits();
    }
}

struct TreeEntry {
    r#move: Option<Move>, // The move leading to this entry from its parent (None for the root)
    depth: u16,           // The depth of this entry in the search tree
    score: f32,           // Current evaluation score at provided depth
    board: Board,         // The board state after applying the move sequence leading to this entry
    flags: TerminalFlags, // Flags to indicate if this entry is terminal and the type of terminal
}

impl TreeEntry {
    fn is_terminal(&self) -> bool {
        (self.flags & TerminalFlags::ANY_TERMINAL) != TerminalFlags::empty()
    }

    fn should_evaluate(&self, epoch: u16) -> bool {
        // We should evaluate this entry if it's not terminal and we haven't evaluated it at this depth before
        !self.is_terminal() && self.depth < epoch && self.depth < 3
    }
}

// fn display_tree(tree: TreeRef<'_, TreeEntry>, indent: usize, depth: usize) {
//     if let Some(mv) = tree.r#move {
//         println!("{}Move: {}, score: {}", "  ".repeat(indent), mv, tree.score);
//     } else {
//         println!("{}Root -- {}", "  ".repeat(indent), tree.board.fen());
//     }

//     if depth > 0 {
//         let mut child_opt = tree.child();
//         while let Some(child) = child_opt {
//             display_tree(child, indent + 1, depth - 1);
//             child_opt = child.next();
//         }
//     }
// }

impl Ai for SimpleAi {
    fn best_move(&mut self, board: &Board, timeout: std::time::Duration) -> Option<(Move, f32)> {
        let mut tree = Tree::new(TreeEntry {
            r#move: None,
            depth: 0,
            score: simple_evaluation(board),
            board: board.clone(),
            flags: TerminalFlags::empty(),
        });

        // Stack for our iterative deepening search, which will contain references to tree nodes
        // alongside the phase
        #[derive(EnumIs, EnumTryAs)]
        enum StackEntry {
            Evaluating(TreeNodeRef),        // Evaluation phase for this node
            Backtracking(TreeNodeRef, f32), // Backtracking phase for this node
        }
        let mut stack = Vec::new();
        let mut moves = Vec::new();

        let start_time = std::time::Instant::now();
        let mut epoch = 0u16;
        loop {
            // While we have time, we will perform a depth-limited search, increasing the depth limit (epoch) with each iteration
            if start_time.elapsed() >= timeout {
                break;
            }

            // Pop last element from the stack
            match stack.pop() {
                Some(StackEntry::Evaluating(noderef)) => {
                    // Get the tree entry for this node reference, then three possiblity
                    let mut entry = tree.get_mut(noderef);
                    let next_to_move = entry.board.next_to_move();

                    if let Some(child_noderef) = entry.child_noderef() {
                        // Only the first child is pushed as it is responsible for pushing the next child
                        // during backtracking.
                        stack.push(StackEntry::Backtracking(noderef, next_to_move.minmax_ini()));
                        stack.push(StackEntry::Evaluating(child_noderef));
                    } else if entry.should_evaluate(epoch) {
                        // Generate moves for this position and add them to the tree as children of the current node
                        let mut currently_in_check = false;
                        generate_moves(&entry.board, &mut moves, &mut currently_in_check);

                        // Handle terminal positions (checkmate or stalemate)
                        if moves.is_empty() {
                            entry.flags |= if currently_in_check {
                                if next_to_move == Color::White {
                                    TerminalFlags::CHECKMATE_BLACK_WIN
                                } else {
                                    TerminalFlags::CHECKMATE_WHITE_WIN
                                }
                            } else {
                                TerminalFlags::STALEMATE
                            };
                            entry.score = if currently_in_check {
                                // If next to move is white then black has won, therefore
                                // negative infinity
                                next_to_move.minmax_ini()
                            } else {
                                0.0
                            };

                            // Push backtracking on the current node
                            stack.push(StackEntry::Backtracking(noderef, entry.score));
                        } else {
                            // let board = entry.board.clone();
                            // Add as many children as we have moves, and push them to the stack for evaluation
                            for mv in moves.drain(..) {
                                let mut new_board = entry.board.clone();
                                mv.apply(&mut new_board);

                                entry.push_child(TreeEntry {
                                    r#move: Some(mv),
                                    depth: entry.depth + 1,
                                    score: simple_evaluation(&new_board),
                                    board: new_board,
                                    flags: TerminalFlags::empty(),
                                });
                            }

                            // Push the first child for evaluation, the rest will be pushed when we backtrack
                            let first_child_noderef = tree.get(noderef).child().unwrap().noderef();
                            stack
                                .push(StackEntry::Backtracking(noderef, next_to_move.minmax_ini()));
                            stack.push(StackEntry::Evaluating(first_child_noderef));
                        }
                    } else {
                        stack.push(StackEntry::Backtracking(noderef, entry.score));
                    }
                }
                Some(StackEntry::Backtracking(noderef, current_score)) => {
                    let mut entry = tree.get_mut(noderef);
                    let next_sibling_noderef = entry.next_noderef();
                    entry.score = current_score;

                    // Update the parent score based on the current score and the color to move at
                    // parent node (if needed)
                    if let Some(parent_stack_entry) = stack.last_mut() {
                        let (parent_noderef, parent_score) =
                            parent_stack_entry.try_as_backtracking_mut().unwrap();

                        let parent_color = tree.get(*parent_noderef).board.next_to_move();
                        *parent_score = parent_color.minmax(*parent_score, current_score);
                    }

                    // If some sibling nodes haven't been evaluated yet, we need to push them
                    // to the stack for evaluation before we can backtrack
                    if let Some(sibling_noderef) = next_sibling_noderef {
                        stack.push(StackEntry::Evaluating(sibling_noderef));
                    }
                }
                None => {
                    // If the stack is empty, we need to start a new search from the root
                    epoch += 1;
                    stack.push(StackEntry::Evaluating(TreeNodeRef::ROOT));
                }
            }
        }

        // After we have exhausted our time, the best move will be the child of the root with the best score
        let root_entry = tree.get(TreeNodeRef::ROOT);
        let color = root_entry.board.next_to_move();
        let mut best_move = None;
        let mut best_score = color.minmax_ini();
        let mut child_noderef_opt = root_entry.child();
        while let Some(child) = child_noderef_opt {
            if color.minmax_cmp(child.score, best_score) {
                best_score = child.score;
                best_move = child.r#move;
            }
            child_noderef_opt = child.next();
        }

        best_move.map(|mv| (mv, best_score))
    }
}
