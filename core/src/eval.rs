use crate::{
    bitboard::Bitboard,
    board::{Color, Piece},
};

pub fn simple_evaluation(board: &crate::board::Board) -> f32 {
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

pub fn larry_kaufman_evaluation(board: &crate::board::Board) -> f32 {
    // Split between middlegame, threshold, and endgame
    let num_white_queens = (*board.get(Piece::Queen) & board.white).count_ones();
    let num_black_queens = (*board.get(Piece::Queen) & !board.white).count_ones();
    // let num_white_rooks = (*board.get(Piece::Rook) & board.white).count_ones();
    // let num_black_rooks = (*board.get(Piece::Rook) & !board.white).count_ones();
    let num_white_bishops = (*board.get(Piece::Bishop) & board.white).count_ones();
    let num_black_bishops = (*board.get(Piece::Bishop) & !board.white).count_ones();
    let num_white_knights = (*board.get(Piece::Knight) & board.white).count_ones();
    let num_black_knights = (*board.get(Piece::Knight) & !board.white).count_ones();

    enum GamePhase {
        Middlegame,
        Threshold,
        Endgame,
    }

    let gamephase = if num_white_queens + num_black_queens == 0 {
        GamePhase::Endgame
    } else if num_white_queens != num_black_queens {
        GamePhase::Threshold
    } else {
        GamePhase::Middlegame
    };

    let mut score: f32 = (num_white_knights as f32 - num_black_knights as f32) * 3.2
        + (num_white_bishops as f32 - num_black_bishops as f32) * 3.3
        + (num_white_queens as f32 - num_black_queens as f32) * 9.4;

    for color in [Color::White, Color::Black] {
        let mask = if color == Color::White {
            board.white
        } else {
            !board.white
        };
        let num_rooks = (*board.get(Piece::Rook) & mask).count_ones();
        let num_bishops = (*board.get(Piece::Bishop) & mask).count_ones();
        let num_queens = (*board.get(Piece::Queen) & mask).count_ones();
        let score_multiplier = if color == Color::White { 1.0 } else { -1.0 };

        match gamephase {
            GamePhase::Middlegame => {
                if num_rooks > 0 {
                    score += score_multiplier * (4.7 + 4.5 * (num_rooks - 1) as f32);
                }

                if num_bishops > 1 {
                    score += score_multiplier * 0.3; // Bonus for having two bishops
                }
            }
            GamePhase::Threshold => {
                if num_rooks > 0 {
                    score += score_multiplier * (4.7 + 4.9 * (num_rooks - 1) as f32);
                }
                if num_queens > 1 {
                    // Second queen is worth less than the first one
                    score -= score_multiplier * 0.7 * (num_queens - 1) as f32;
                }
            }
            GamePhase::Endgame => {
                if num_rooks > 0 {
                    score += score_multiplier * (5.3 + 5.0 * (num_rooks - 1) as f32);
                }
            }
        }

        // Evaluate pawns based on the game phase
        let pawn_bitboard = *board.get(Piece::Pawn) & mask;
        let enemy_pawns_bitboard = *board.get(Piece::Pawn) & !mask;
        let isolated_pawns = pawn_bitboard & !pawn_bitboard.surrounding_mask();
        let connected_pawns = pawn_bitboard & pawn_bitboard.connected_mask(color.opposite());

        let mut pawn_score: f32 = 0.0;
        for pawn in pawn_bitboard.scan() {
            let file = pawn % 8;
            let rank = pawn / 8;
            let rank_colorless = if color == Color::White {
                rank
            } else {
                7 - rank
            };

            // Check if isolated
            let is_isolated = isolated_pawns.get(pawn) && rank_colorless >= 2;
            let is_connected = connected_pawns.get(pawn);
            let is_passed = rank_colorless >= 4
                && (enemy_pawns_bitboard.0 & Bitboard::FILE[file as usize]) == 0;

            if is_passed {
                if is_connected {
                    match rank_colorless {
                        4 => pawn_score += 1.55,
                        5 => pawn_score += 2.3,
                        6.. => pawn_score += 3.5,
                        _ => unreachable!(),
                    }
                } else {
                    match rank_colorless {
                        4 => pawn_score += 1.30,
                        5 => pawn_score += 1.55,
                        6.. => pawn_score += 3.5,
                        _ => unreachable!(),
                    }
                }
            } else {
                let multiplier = {
                    if is_isolated {
                        match rank_colorless {
                            2 => 0.8,
                            3 => 0.9,
                            4 => 1.05,
                            5 => 1.30,
                            6.. => 2.1,
                            _ => unreachable!(),
                        }
                    } else if is_connected && rank_colorless >= 4 {
                        match rank_colorless {
                            4 => 1.15,
                            5.. => 1.35,
                            _ => unreachable!(),
                        }
                    } else {
                        1.0
                    }
                };

                let table = match gamephase {
                    GamePhase::Middlegame | GamePhase::Threshold => {
                        [
                            0.90, 0.95, 1.05, 1.10, // Rank 2
                            0.90, 0.95, 1.05, 1.15, // Rank 3
                            0.90, 0.95, 1.10, 1.20, // Rank 4
                            0.97, 1.03, 1.17, 1.27, // Rank 5
                            1.06, 1.12, 1.25, 1.40, // Rank 6
                        ]
                    }
                    GamePhase::Endgame => {
                        [
                            1.20, 1.05, 0.95, 0.90, // Rank 2
                            1.20, 1.05, 0.95, 0.90, // Rank 3
                            1.25, 1.10, 1.00, 0.95, // Rank 4
                            1.33, 1.17, 1.07, 1.00, // Rank 5
                            1.45, 1.29, 1.16, 1.05, // Rank 6
                        ]
                    }
                };

                let mofile = if file < 4 { file } else { 7 - file } as usize;
                let rank_index = (rank_colorless as usize - 1).clamp(0, 4);
                pawn_score += table[mofile + rank_index * 4] * multiplier;
            }
        }

        // Add the pawn score to the total score
        score += score_multiplier * pawn_score;
    }

    score
}

pub fn evaluate(board: &crate::board::Board) -> f32 {
    // For now, we just use the simple evaluation function, but this is where we would implement a more complex evaluation
    // simple_evaluation(board)
    #[cfg(feature = "eval_larry_kaufman")]
    {
        larry_kaufman_evaluation(board)
    }
    #[cfg(not(feature = "eval_larry_kaufman"))]
    {
        simple_evaluation(board)
    }
}
