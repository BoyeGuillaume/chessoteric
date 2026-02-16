use std::usize;

use bitflags::bitflags;

use crate::{
    ai::Ai,
    board::{Board, Color, Piece},
    moves::{Move, generate_moves},
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
            Piece::King => 1000.0,
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
    first_child_id: Option<usize>, // The first child of this entry in the search tree
    next_sibling: Option<usize>,   // The next sibling of this entry in the search tree
    r#move: Option<Move>, // The move leading to this entry from its parent (None for the root)
    board: Board,         // The board state after applying the move sequence leading to this entry
    score: f32,           // Current evaluation score at provided depth
    depth: usize,         // The depth of this entry in the search tree
    flags: TerminalFlags, // Flags to indicate if this entry is terminal and the type of terminal
}

impl TreeEntry {
    fn is_terminal(&self) -> bool {
        (self.flags & TerminalFlags::ANY_TERMINAL) != TerminalFlags::empty()
    }

    fn should_evaluate(&self, epoch: usize) -> bool {
        // We should evaluate this entry if it's not terminal and we haven't evaluated it at this depth before
        !self.is_terminal() && self.depth <= epoch
    }
}

impl Ai for SimpleAi {
    fn best_move(&mut self, board: &Board, timeout: std::time::Duration) -> Option<(Move, f32)> {
        // Generate the initial board position
        let mut tree = vec![TreeEntry {
            first_child_id: None,
            next_sibling: None,
            r#move: None, // No move for the root
            board: board.clone(),
            score: simple_evaluation(board),
            depth: 0,
            flags: TerminalFlags::empty(),
        }];

        struct StackEntry {
            tree_id: usize,  // The ID of the corresponding entry in the search tree
            best_score: f32, // Best score among the siblings of this entry, used for backtracking
        }
        let mut stack: Vec<StackEntry> = vec![];

        // Start of the main exploration loop
        let loop_start = std::time::Instant::now();
        let mut epoch = 0; // max depth for the current iteration
        loop {
            // If timeout reached, break the loop
            if loop_start.elapsed() >= timeout {
                break;
            }

            // Proceed as follow
            // (1) If stack is empty, launch a new epoch by pushing the root entry to the stack
            // (2) If stack is not empty, peek the top entry and
            //    (2.a) If the entry need to be evaluated,
            //          (2.a.i) If entry has children, recursively evaluate children (push first child to the stack)
            //          (2.a.ii) If entry has no children and is terminal, mark it as terminal and backtrack
            //          (2.a.iii) If entry has no children and is not terminal, generate its children and push the first child to the stack
            //    (2.b) If the entry doesn't need to be evaluated, backtrack and update the parent entry with the best score found
            //
            // When backtracking,
            // (1) If the `next_sibling` of the entry is not None, push the next sibling to the stack, pop current
            // (2) If the `next_sibling` of the entry is None, pop current and backtrack parent

            if let Some(last_entry) = stack.last() {
                // If the entry needs to be evaluated, we need to evaluate it
                let mut requires_backtrack = None; // The best score for the parent entry
                let tree_entry = &mut tree[last_entry.tree_id];
                let current_color = tree_entry.board.next_to_move();

                if tree_entry.should_evaluate(epoch) {
                    // We need to evaluate this entry, check if children exist
                    if let Some(first_child_id) = tree_entry.first_child_id {
                        // We have children, we need to evaluate them
                        stack.push(StackEntry {
                            tree_id: first_child_id,
                            best_score: current_color.opposite().minmax_ini(), // Start with -inf for the child, we want to maximize for the child
                        });
                    } else if tree_entry.is_terminal() {
                        // This entry is terminal, we can backtrack
                        let score = current_color.minmax(last_entry.best_score, tree_entry.score);
                        requires_backtrack = Some(score);
                    } else {
                        // No children, we need to evaluate this entry
                        let mut is_in_check = false;
                        let mut moves = vec![];
                        generate_moves(&tree_entry.board, &mut moves, &mut is_in_check);

                        if moves.is_empty() {
                            // Set this entry as checkmate, we can backtrack
                            tree_entry.flags |= if is_in_check {
                                if current_color == Color::White {
                                    TerminalFlags::CHECKMATE_BLACK_WIN
                                } else {
                                    TerminalFlags::CHECKMATE_WHITE_WIN
                                }
                            } else {
                                TerminalFlags::STALEMATE
                            };

                            // Sets the score of this entry according to the result
                            tree_entry.score = if is_in_check {
                                current_color.minmax_ini()
                            } else {
                                0.0 // Stalemate
                            };
                            requires_backtrack = Some(tree_entry.score);
                        } else {
                            // This entry is not terminal, we need to add its children to the tree
                            let mut previous_child_id = None;
                            let mut best_score = current_color.minmax_ini();

                            let board = tree_entry.board.clone();
                            let current_depth = tree_entry.depth;

                            // let mut last_child_id = None;
                            for mv in moves {
                                let mut new_board = board.clone();
                                mv.apply(&mut new_board);
                                let new_score = simple_evaluation(&new_board);
                                let new_entry_id = tree.len();
                                best_score = current_color.minmax(best_score, new_score);

                                tree.push(TreeEntry {
                                    first_child_id: None,
                                    next_sibling: previous_child_id,
                                    board: new_board,
                                    score: new_score,
                                    r#move: Some(mv),
                                    depth: current_depth + 1,
                                    flags: TerminalFlags::empty(),
                                });
                                previous_child_id = Some(new_entry_id);
                            }

                            // Update the first_child_id of this node
                            debug_assert!(
                                previous_child_id.is_some(),
                                "We should have at least one child since moves is not empty"
                            );
                            let tree_entry = &mut tree[last_entry.tree_id];
                            tree_entry.first_child_id = previous_child_id;
                            tree_entry.score = best_score;

                            // Add children to the stack
                            stack.push(StackEntry {
                                tree_id: previous_child_id.unwrap(),
                                best_score: current_color.opposite().minmax_ini(), // Start with -inf for the child, we want to maximize for the child
                            });
                        }
                    }
                } else {
                    // We don't need to evaluate this entry, we can backtrack
                    let tree_entry = &mut tree[last_entry.tree_id];
                    requires_backtrack = Some(tree_entry.score);
                }

                // If requires backtracking, we either
                // (1) pop then push the next sibling to the stack if it exists
                // (2) pop and backtrack to the parent if no sibling
                while let Some(score) = requires_backtrack {
                    let last_entry = stack.pop().unwrap();

                    let tree_entry = &tree[last_entry.tree_id];
                    let current_color = tree_entry.board.next_to_move();

                    if let Some(sibling) = tree_entry.next_sibling {
                        // We have a sibling, we need to evaluate it
                        stack.push(StackEntry {
                            tree_id: sibling,
                            best_score: score,
                        });
                        requires_backtrack = None; // No need to backtrack further since we have a sibling to evaluate
                    } else {
                        // No sibling, we need to backtrack to the parent (if exists)
                        stack.pop();
                        if let Some(parent_entry) = stack.last_mut() {
                            parent_entry.best_score = current_color
                                .opposite()
                                .minmax(parent_entry.best_score, score);
                            requires_backtrack = Some(score);
                        } else {
                            requires_backtrack = None; // No parent, we are back at the root, we can stop backtracking
                        }
                    }
                }
            } else {
                // Start a new epoch
                epoch += 1;
                stack.push(StackEntry {
                    tree_id: 0,
                    best_score: board.next_to_move().minmax_ini(), // Start with -inf for the root, we want to maximize for the root
                });
            }
        }

        // After the exploration loop, the best move is the child of the root with the best score
        let root_entry = &tree[0];
        let mut best_move = None;
        let mut best_score = board.next_to_move().minmax_ini();
        let mut child_id = root_entry.first_child_id;
        while let Some(id) = child_id {
            let child_entry = &tree[id];
            if board
                .next_to_move()
                .minmax_cmp(child_entry.score, best_score)
            {
                best_score = child_entry.score;
                best_move = Some((child_entry.r#move.unwrap(), child_entry.score));
            }
            child_id = child_entry.next_sibling;
        }

        best_move
    }
}
