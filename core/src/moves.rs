use crate::{
    bitboard::{Bitboard, Direction, algebraic_to_square, square_to_algebraic},
    board::{Board, BoardFlags, Color, Piece},
};
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MoveFlags: u8 {
        const CASTLE = 1 << 0;
        const EN_PASSANT = 1 << 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub piece: Piece,
    pub promotion: Option<Piece>,
    pub flags: MoveFlags,
}

impl Move {
    fn display_castle(&self) -> Option<&'static str> {
        if self.flags.contains(MoveFlags::CASTLE) {
            match self.to {
                6 | 62 => Some("O-O"),
                2 | 58 => Some("O-O-O"),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn algebraic_notation<'a>(
        &'a self,
        board: &'a Board,
        other: &'a Vec<Move>,
    ) -> impl std::fmt::Display + 'a {
        struct SimplifiedUciMove<'a> {
            r#move: &'a Move,
            board: &'a Board,
            other_moves: &'a Vec<Move>,
        }

        impl std::fmt::Display for SimplifiedUciMove<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if let Some(castle_str) = self.r#move.display_castle() {
                    return write!(f, "{}", castle_str);
                }

                // Is this a capture move
                let is_capture_move = self.board.occupied.get(self.r#move.to)
                    || (self.r#move.flags.contains(MoveFlags::EN_PASSANT));

                // Do we need to include the piece symbol in the move notation,
                let needs_piece_symbol = self.r#move.piece != Piece::Pawn;

                // Do we need to disambiguate the file
                let needs_source_disambiguation = self.other_moves.iter().any(|m| {
                    m.to == self.r#move.to
                        && m.piece == self.r#move.piece
                        && m.from != self.r#move.from
                });

                let file_disambiguation_sufficient = self.other_moves.iter().all(|m| {
                    m.to != self.r#move.to
                        || m.piece != self.r#move.piece
                        || m.from == self.r#move.from
                        || (m.from % 8) as u8 != (self.r#move.from % 8) as u8
                });

                let needs_file_disambiguation = (needs_source_disambiguation
                    && file_disambiguation_sufficient)
                    || (is_capture_move && self.r#move.piece == Piece::Pawn);

                let needs_rank_disambiguation =
                    needs_source_disambiguation && !file_disambiguation_sufficient;

                // Determine if this is a check or checkmate move, to include the + or # symbol in the move notation
                let mut board_after_move = self.board.clone();
                self.r#move.apply(&mut board_after_move);
                let mut mvs = Vec::new();
                let mut currently_in_check = false;
                generate_moves(&board_after_move, &mut mvs, &mut currently_in_check);
                let is_checkmate = mvs.is_empty() && currently_in_check;

                // Finally, construct the move string
                let piece_str = if needs_piece_symbol {
                    self.r#move.piece.symbol().to_ascii_uppercase()
                } else {
                    String::new()
                };

                // If this is a capture move, we need to include the 'x' symbol in the move notation
                let capture_str = if is_capture_move { "x" } else { "" };

                // If we need to disambiguate the file, we include the file of the origin square in the move notation
                let file_disambiguation_str = if needs_file_disambiguation {
                    let file = (self.r#move.from % 8) as u8;
                    ((b'a' + file) as char).to_string()
                } else {
                    String::new()
                };

                let rank_disambiguation_str = if needs_rank_disambiguation {
                    let rank = (self.r#move.from / 8) as u8;
                    ((b'1' + rank) as char).to_string()
                } else {
                    String::new()
                };

                // If this is a promotion move, we need to include the symbol of the promotion piece in the move notation
                let promotion_str = if let Some(promotion_piece) = self.r#move.promotion {
                    format!("={}", promotion_piece.symbol().to_ascii_uppercase())
                } else {
                    String::new()
                };

                // If this is a check move, we need to include the '+' symbol in the move
                let check_str = if is_checkmate {
                    "#"
                } else if currently_in_check {
                    "+"
                } else {
                    ""
                };

                // Finally, we construct the move string
                write!(
                    f,
                    "{}{}{}{}{}{}{}",
                    piece_str,
                    file_disambiguation_str,
                    rank_disambiguation_str,
                    capture_str,
                    square_to_algebraic(self.r#move.to),
                    promotion_str,
                    check_str
                )
            }
        }

        SimplifiedUciMove {
            r#move: self,
            board,
            other_moves: other,
        }
    }

    pub fn uci(&self) -> impl std::fmt::Display + '_ {
        struct UciMove<'a>(&'a Move);

        impl std::fmt::Display for UciMove<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // if let Some(castle_str) = self.0.display_castle() {
                //     return write!(f, "{}", castle_str);
                // }

                let promotion_str = if let Some(promotion_piece) = self.0.promotion {
                    format!("{}", promotion_piece.with_color(Color::Black).symbol())
                } else {
                    String::new()
                };
                write!(
                    f,
                    "{}{}{}",
                    square_to_algebraic(self.0.from),
                    square_to_algebraic(self.0.to),
                    promotion_str
                )
            }
        }

        UciMove(self)
    }

    pub fn from_uci(value: &str, board: &Board) -> Option<Self> {
        if value.len() < 4 {
            return None;
        }

        let from = algebraic_to_square(&value[0..2])?;
        let to = algebraic_to_square(&value[2..4])?;

        let promotion = if value.len() > 4 {
            match value.chars().nth(4)? {
                'q' => Some(Piece::Queen),
                'r' => Some(Piece::Rook),
                'b' => Some(Piece::Bishop),
                'n' => Some(Piece::Knight),
                _ => return None,
            }
        } else {
            None
        };

        let mut current_piece = None;
        for piece in Piece::colorless_iter() {
            if board.get(piece).get(from) {
                current_piece = Some(piece);
                break;
            }
        }
        let current_piece = current_piece?;

        // If pawn and diagonal move without destination piece, it is an en passant capture
        let flags = if current_piece == Piece::Pawn
            && (from as i8 - to as i8).abs() % 8 != 0
            && !board.occupied.get(to)
        {
            MoveFlags::EN_PASSANT
        } else if current_piece == Piece::King && (from == 4 && to == 6 || from == 60 && to == 62) {
            MoveFlags::CASTLE
        } else if current_piece == Piece::King && (from == 4 && to == 2 || from == 60 && to == 58) {
            MoveFlags::CASTLE
        } else {
            MoveFlags::empty()
        };

        Some(Move {
            from,
            to,
            piece: current_piece,
            promotion,
            flags,
        })
    }

    pub fn apply(&self, board: &mut Board) {
        // Remove all pieces of all bitboards on the destination square, to handle captures and promotions
        for bitboard in board.bitboards.iter_mut() {
            bitboard.unset(self.to);
        }

        let bitboard = board.get_mut(self.piece.colorless());
        if let Some(promotion_piece) = self.promotion {
            bitboard.unset(self.from); // Remove the piece from the origin square
            board.get_mut(promotion_piece.colorless()).set(self.to); // Place the promoted piece on the destination square
        } else {
            *bitboard ^= Bitboard((1 << self.from) | (1 << self.to)); // Move the piece from the origin square to the destination square
        }

        // Handle occupied
        board.occupied.unset(self.from);
        board.occupied.set(self.to);

        board.white.unset(self.from);

        if board.flags.contains(BoardFlags::WHITE_TO_MOVE) {
            board.white.set(self.to);
        } else {
            board.white.unset(self.to);
        }

        // Check if current move generates a en passant square, if so, set the en passant square in the board flags
        if self.piece == Piece::Pawn && (self.to as i8 - self.from as i8).abs() == 16 {
            let en_passant_square = match board.next_to_move() {
                Color::White => self.to - 8,
                Color::Black => self.to + 8,
            };
            board.en_passant_square = en_passant_square;
        } else {
            board.en_passant_square = 64; // Invalid square, to indicate that there is no en passant square
        }

        // If current move is an en passant capture, we need to remove the captured pawn
        if self.flags.contains(MoveFlags::EN_PASSANT) {
            let captured_pawn_square = match board.next_to_move() {
                Color::White => self.to - 8,
                Color::Black => self.to + 8,
            };
            board.get_mut(Piece::Pawn).unset(captured_pawn_square);
            board.occupied.unset(captured_pawn_square);
            board.white.unset(captured_pawn_square);
        }

        // Handle castling rights, if the move is a king move, remove all castling right of the current side
        if self.piece == Piece::King {
            match board.next_to_move() {
                Color::White => board.flags.remove(
                    BoardFlags::WHITE_KING_SIDE_CASTLE | BoardFlags::WHITE_QUEEN_SIDE_CASTLE,
                ),
                Color::Black => board.flags.remove(
                    BoardFlags::BLACK_KING_SIDE_CASTLE | BoardFlags::BLACK_QUEEN_SIDE_CASTLE,
                ),
            }
        }

        // If the move is a rook move, we need to remove the castling right corresponding to the rook if it is on its original square
        match self.to {
            0 => board.flags.remove(BoardFlags::WHITE_QUEEN_SIDE_CASTLE),
            7 => board.flags.remove(BoardFlags::WHITE_KING_SIDE_CASTLE),
            56 => board.flags.remove(BoardFlags::BLACK_QUEEN_SIDE_CASTLE),
            63 => board.flags.remove(BoardFlags::BLACK_KING_SIDE_CASTLE),
            _ => { /* No castling rights to remove */ }
        }

        match board.next_to_move() {
            Color::White => {
                if self.from == 0 {
                    board.flags.remove(BoardFlags::WHITE_QUEEN_SIDE_CASTLE);
                } else if self.from == 7 {
                    board.flags.remove(BoardFlags::WHITE_KING_SIDE_CASTLE);
                }
            }
            Color::Black => {
                if self.from == 56 {
                    board.flags.remove(BoardFlags::BLACK_QUEEN_SIDE_CASTLE);
                } else if self.from == 63 {
                    board.flags.remove(BoardFlags::BLACK_KING_SIDE_CASTLE);
                }
            }
        }

        // If the move is a castle, we need to move the rook as well
        if self.piece == Piece::King && self.flags.contains(MoveFlags::CASTLE) {
            match self.to {
                6 => {
                    // White king side castle
                    board.get_mut(Piece::Rook).unset(7);
                    board.get_mut(Piece::Rook).set(5);
                    board.occupied.unset(7);
                    board.occupied.set(5);
                    board.white.unset(7);
                    board.white.set(5);
                    board.flags.remove(BoardFlags::WHITE_CASTLE);
                }
                2 => {
                    // White queen side castle
                    board.get_mut(Piece::Rook).unset(0);
                    board.get_mut(Piece::Rook).set(3);
                    board.occupied.unset(0);
                    board.occupied.set(3);
                    board.white.unset(0);
                    board.white.set(3);
                    board.flags.remove(BoardFlags::WHITE_CASTLE);
                }
                62 => {
                    // Black king side castle
                    board.get_mut(Piece::Rook).unset(63);
                    board.get_mut(Piece::Rook).set(61);
                    board.occupied.unset(63);
                    board.occupied.set(61);
                    board.flags.remove(BoardFlags::BLACK_CASTLE);
                }
                58 => {
                    // Black queen side castle
                    board.get_mut(Piece::Rook).unset(56);
                    board.get_mut(Piece::Rook).set(59);
                    board.occupied.unset(56);
                    board.occupied.set(59);
                    board.flags.remove(BoardFlags::BLACK_CASTLE);
                }
                _ => {}
            }
        }

        board.flags.toggle(BoardFlags::WHITE_TO_MOVE);
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let promotion_str = if let Some(promotion_piece) = self.promotion {
            format!("={}", promotion_piece)
        } else {
            String::new()
        };
        write!(
            f,
            "{}{}{}{}",
            self.piece,
            square_to_algebraic(self.from),
            square_to_algebraic(self.to),
            promotion_str
        )
    }
}

pub fn generate_rook_movement(occlusion: Bitboard, origin: Bitboard) -> Bitboard {
    origin.sliding_attack(occlusion, Direction::East)
        | origin.sliding_attack(occlusion, Direction::West)
        | origin.sliding_attack(occlusion, Direction::North)
        | origin.sliding_attack(occlusion, Direction::South)
}

pub fn generate_bishop_movement(occlusion: Bitboard, origin: Bitboard) -> Bitboard {
    origin.sliding_attack(occlusion, Direction::NorthEast)
        | origin.sliding_attack(occlusion, Direction::SouthWest)
        | origin.sliding_attack(occlusion, Direction::NorthWest)
        | origin.sliding_attack(occlusion, Direction::SouthEast)
}

pub fn generate_queen_movement(occlusion: Bitboard, origin: Bitboard) -> Bitboard {
    generate_rook_movement(occlusion, origin) | generate_bishop_movement(occlusion, origin)
}

pub fn generate_knight_movement(origin: Bitboard) -> Bitboard {
    let l1 = (origin.0 >> 1) & 0x7f7f7f7f7f7f7f7f;
    let l2 = (origin.0 >> 2) & 0x3f3f3f3f3f3f3f3f;
    let r1 = (origin.0 << 1) & 0xfefefefefefefefe;
    let r2 = (origin.0 << 2) & 0xfcfcfcfcfcfcfcfc;
    let h1 = l1 | r1;
    let h2 = l2 | r2;
    let attacks = (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8);
    Bitboard(attacks)
}

pub fn generate_king_movement(origin: Bitboard) -> Bitboard {
    origin.surrounding_mask()
}

fn generate_pawn_attacks(origin: Bitboard, color: Color) -> Bitboard {
    match color {
        Color::White => {
            let east_attacks = (origin.0 << 9) & !Bitboard::FILE_A;
            let west_attacks = (origin.0 << 7) & !Bitboard::FILE_H;
            Bitboard(east_attacks | west_attacks)
        }
        Color::Black => {
            let east_attacks = (origin.0 >> 7) & !Bitboard::FILE_A;
            let west_attacks = (origin.0 >> 9) & !Bitboard::FILE_H;
            Bitboard(east_attacks | west_attacks)
        }
    }
}

pub fn generate_moves(board: &Board, moves: &mut Vec<Move>, currently_in_check: &mut bool) {
    moves.clear();

    let rook_like = *board.get(Piece::Rook) | *board.get(Piece::Queen);
    let bishop_like = *board.get(Piece::Bishop) | *board.get(Piece::Queen);
    let knight_like = *board.get(Piece::Knight);
    let pawn_like = *board.get(Piece::Pawn);
    let rook_like_enemy = rook_like & board.enemy_bitboard();
    let bishop_like_enemy = bishop_like & board.enemy_bitboard();
    let knight_enemy = knight_like & board.enemy_bitboard();
    let pawn_like_enemy = pawn_like & board.enemy_bitboard();
    *currently_in_check = false;

    // Retrieve the ally king position and bitboard
    let ally_king_bitboard = *board.get(Piece::King) & board.friendly_bitboard();
    debug_assert!(
        ally_king_bitboard.count_ones() == 1,
        "There should be exactly one king for the side to move"
    );
    let king_square = ally_king_bitboard.square();

    // A list of all of the squares that pieces can move to, except for king moves
    let mut destination_filter_outside_king = !board.friendly_bitboard();
    let mut pinned_bitboard = Bitboard::empty();

    // Find all of the squares the ennemy is attacking, to filter out king moves to those squares,
    // we separature them in rook-like, bishop-like, and everything else to help figure out pinning and checks later
    // on
    let board_occupied_except_king = board.occupied & !ally_king_bitboard;
    let enemy_rook_like_attacks =
        generate_rook_movement(board_occupied_except_king, rook_like_enemy);
    let enemy_bishop_like_attacks =
        generate_bishop_movement(board_occupied_except_king, bishop_like_enemy);
    let enemy_knight_attacks = generate_knight_movement(knight_enemy);
    let enemy_pawn_attacks =
        generate_pawn_attacks(pawn_like_enemy, board.next_to_move().opposite());
    let enemy_king_attacks =
        generate_king_movement(*board.get(Piece::King) & board.enemy_bitboard());
    let all_enemy_attacks = enemy_rook_like_attacks
        | enemy_bishop_like_attacks
        | enemy_knight_attacks
        | enemy_pawn_attacks
        | enemy_king_attacks;

    // If currently in check, we need to filter out any moves that don't block the check or move the king
    {
        let king_rook_ray = generate_rook_movement(board.occupied, ally_king_bitboard);
        let king_bishop_ray = generate_bishop_movement(board.occupied, ally_king_bitboard);
        let king_rook_checkers = king_rook_ray & rook_like_enemy;
        let king_bishop_checkers = king_bishop_ray & bishop_like_enemy;

        // Bitscan other the king_rook_checkers and update the allowed_destinations
        // as needed
        for checker_square in king_rook_checkers.scan_bitboard() {
            debug_assert!(
                checker_square.count_ones() == 1,
                "Checker square should have exactly one bit set"
            );
            destination_filter_outside_king &=
                (generate_rook_movement(board.occupied, checker_square) & king_rook_ray)
                    | checker_square;
            *currently_in_check = true;
        }

        // Bitscan over the king_bishop_checkers and update the allowed_destinations as needed
        for checker_square in king_bishop_checkers.scan_bitboard() {
            debug_assert!(
                checker_square.count_ones() == 1,
                "Checker square should have exactly one bit set"
            );
            destination_filter_outside_king &=
                (generate_bishop_movement(board.occupied, checker_square) & king_bishop_ray)
                    | checker_square;
            *currently_in_check = true;
        }

        // Compute x-ray attacks for pinned pieces, to do so, we perform a ray attack, where the occlusion as been modified
        // to remove the piece being considered
        let king_rook_friendly_blocker = king_rook_ray & board.friendly_bitboard();
        let king_bishop_friendly_blocker = king_bishop_ray & board.friendly_bitboard();

        let king_rook_xray = generate_rook_movement(
            board.occupied & !king_rook_friendly_blocker,
            ally_king_bitboard,
        );
        let king_bishop_xray = generate_bishop_movement(
            board.occupied & !king_bishop_friendly_blocker,
            ally_king_bitboard,
        );

        // Iterate over all pinners (enemy pieces that have less that are technically shielded by a friendly piece
        // but that are actually pinning that piece because of the presence of the king behind it)
        let pinner_rook_squares = king_rook_xray & rook_like_enemy & !king_rook_ray;
        let pinner_bishop_squares = king_bishop_xray & bishop_like_enemy & !king_bishop_ray;

        for pinner in pinner_rook_squares.scan_bitboard() {
            // Raycast from this piece to the king
            let pinner_ray = generate_rook_movement(board.occupied, pinner);
            pinned_bitboard |= pinner_ray & king_rook_ray; // & king_rook_friendly_blocker;
        }

        for pinner in pinner_bishop_squares.scan_bitboard() {
            // Raycast from this piece to the king
            let pinner_ray = generate_bishop_movement(board.occupied, pinner);
            pinned_bitboard |= pinner_ray & king_bishop_ray; // & king_bishop_friendly_blocker;
        }

        // Consider knight checks as well, which are simpler since they can't be blocked
        let king_knight_checkers = generate_knight_movement(ally_king_bitboard) & knight_enemy;
        let king_pawn_checkers =
            generate_pawn_attacks(ally_king_bitboard, board.next_to_move()) & pawn_like_enemy;
        for checker_square in (king_knight_checkers | king_pawn_checkers).scan_bitboard() {
            debug_assert!(
                checker_square.count_ones() == 1,
                "Checker square should have exactly one bit set"
            );
            destination_filter_outside_king &= checker_square; // Only the square the knight is on can be moved to, to capture the knight
            *currently_in_check = true;
        }
    }

    // Generate pawn moves (non-captures first, since they are simpler and also can be double moves)
    let friendly_pawns = *board.get(Piece::Pawn) & board.friendly_bitboard();
    let pawn_non_capture_single_moves = match board.next_to_move() {
        Color::White => (friendly_pawns.0 << 8) & !board.occupied.0,
        Color::Black => (friendly_pawns.0 >> 8) & !board.occupied.0,
    };
    let pawn_non_capture_double_moves = match board.next_to_move() {
        Color::White => {
            ((pawn_non_capture_single_moves & Bitboard::RANK_3) << 8) & !board.occupied.0
        }
        Color::Black => {
            ((pawn_non_capture_single_moves & Bitboard::RANK_6) >> 8) & !board.occupied.0
        }
    };

    for pawn_single_move in
        Bitboard(pawn_non_capture_single_moves & destination_filter_outside_king.0).scan()
    {
        let can_promote = match board.next_to_move() {
            Color::White => pawn_single_move >= 56,
            Color::Black => pawn_single_move <= 7,
        };

        let from_square = match board.next_to_move() {
            Color::White => pawn_single_move - 8,
            Color::Black => pawn_single_move + 8,
        };

        if can_promote {
            for promotion_piece in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                moves.push(Move {
                    from: from_square as u8,
                    to: pawn_single_move as u8,
                    piece: Piece::Pawn,
                    promotion: Some(promotion_piece),
                    flags: MoveFlags::empty(),
                });
            }
        } else {
            moves.push(Move {
                from: from_square as u8,
                to: pawn_single_move as u8,
                piece: Piece::Pawn,
                promotion: None,
                flags: MoveFlags::empty(),
            });
        }
    }

    for pawn_double_move in
        Bitboard(pawn_non_capture_double_moves & destination_filter_outside_king.0).scan()
    {
        let from_square = match board.next_to_move() {
            Color::White => pawn_double_move - 16,
            Color::Black => pawn_double_move + 16,
        };

        moves.push(Move {
            from: from_square as u8,
            to: pawn_double_move as u8,
            piece: Piece::Pawn,
            promotion: None,
            flags: MoveFlags::empty(),
        });
    }

    // Generate pawn captures
    let pawn_attacks = generate_pawn_attacks(friendly_pawns, board.next_to_move());
    let pawn_capture_moves =
        pawn_attacks & board.enemy_bitboard() & destination_filter_outside_king;
    for pawn_capture_move in pawn_capture_moves.scan() {
        let can_promote = match board.next_to_move() {
            Color::White => pawn_capture_move >= 56,
            Color::Black => pawn_capture_move <= 7,
        };
        let dir = match board.next_to_move() {
            Color::White => [
                (Direction::SouthEast, !Bitboard::FILE_A),
                (Direction::SouthWest, !Bitboard::FILE_H),
            ],
            Color::Black => [
                (Direction::NorthEast, !Bitboard::FILE_A),
                (Direction::NorthWest, !Bitboard::FILE_H),
            ],
        };

        for (direction, mask) in dir.iter() {
            let from_square = direction.shift(pawn_capture_move).unwrap();
            if (Bitboard(1 << from_square) & Bitboard(*mask)) & friendly_pawns != Bitboard::empty()
            {
                if can_promote {
                    for promotion_piece in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight]
                    {
                        moves.push(Move {
                            from: from_square as u8,
                            to: pawn_capture_move as u8,
                            piece: Piece::Pawn,
                            promotion: Some(promotion_piece),
                            flags: MoveFlags::empty(),
                        });
                    }
                } else {
                    moves.push(Move {
                        from: from_square as u8,
                        to: pawn_capture_move as u8,
                        piece: Piece::Pawn,
                        promotion: None,
                        flags: MoveFlags::empty(),
                    });
                }
            }
        }
    }

    // Generate en passant moves
    if (pawn_attacks & destination_filter_outside_king)
        & Bitboard(u64::checked_shl(1, board.en_passant_square as u32).unwrap_or(0))
        != Bitboard::empty()
    {
        let dir = match board.next_to_move() {
            Color::White => [Direction::SouthEast, Direction::SouthWest],
            Color::Black => [Direction::NorthEast, Direction::NorthWest],
        };

        for direction in dir.iter() {
            let from_square = direction.shift(board.en_passant_square).unwrap();
            if Bitboard(1 << from_square) & friendly_pawns != Bitboard::empty() {
                moves.push(Move {
                    from: from_square as u8,
                    to: board.en_passant_square as u8,
                    piece: Piece::Pawn,
                    promotion: None,
                    flags: MoveFlags::EN_PASSANT,
                });
            }
        }
    }

    // Generate rook moves
    let rook_friendly = *board.get(Piece::Rook) & board.friendly_bitboard();
    for rook_square in rook_friendly.scan() {
        let rook_moves = generate_rook_movement(board.occupied, Bitboard(1 << rook_square))
            & destination_filter_outside_king;
        for rook_move in rook_moves.scan() {
            moves.push(Move {
                from: rook_square as u8,
                to: rook_move as u8,
                piece: Piece::Rook,
                promotion: None,
                flags: MoveFlags::empty(),
            });
        }
    }

    let bishop_friendly = *board.get(Piece::Bishop) & board.friendly_bitboard();
    for bishop_square in bishop_friendly.scan() {
        let bishop_moves = generate_bishop_movement(board.occupied, Bitboard(1 << bishop_square))
            & destination_filter_outside_king;
        for bishop_move in bishop_moves.scan() {
            moves.push(Move {
                from: bishop_square as u8,
                to: bishop_move as u8,
                piece: Piece::Bishop,
                promotion: None,
                flags: MoveFlags::empty(),
            });
        }
    }

    // Generate queen moves
    let queen_friendly = *board.get(Piece::Queen) & board.friendly_bitboard();
    for queen_square in queen_friendly.scan() {
        let queen_moves = generate_queen_movement(board.occupied, Bitboard(1 << queen_square))
            & destination_filter_outside_king;
        for queen_move in queen_moves.scan() {
            moves.push(Move {
                from: queen_square as u8,
                to: queen_move as u8,
                piece: Piece::Queen,
                promotion: None,
                flags: MoveFlags::empty(),
            });
        }
    }

    // Generate knight moves
    let knight_friendly = *board.get(Piece::Knight) & board.friendly_bitboard();
    let knight_moves = generate_knight_movement(knight_friendly) & destination_filter_outside_king;
    for knight_move in knight_moves.scan() {
        let from_squares = generate_knight_movement(Bitboard(1 << knight_move)) & knight_friendly;
        for from_square in from_squares.scan() {
            moves.push(Move {
                from: from_square as u8,
                to: knight_move as u8,
                piece: Piece::Knight,
                promotion: None,
                flags: MoveFlags::empty(),
            });
        }
    }

    // Generate king moves
    let king_moves = generate_king_movement(ally_king_bitboard)
        & !board.friendly_bitboard()
        & !all_enemy_attacks;
    for king_move in king_moves.scan() {
        moves.push(Move {
            from: king_square as u8,
            to: king_move as u8,
            piece: Piece::King,
            promotion: None,
            flags: MoveFlags::empty(),
        });
    }

    // Generate castling moves, we need to check that the squares between the king and the rook are empty, and
    // that the king is not in threat during transit
    let threat_or_non_empty = board.occupied | all_enemy_attacks;
    if !*currently_in_check {
        match board.next_to_move() {
            Color::White => {
                if board.flags.contains(BoardFlags::WHITE_KING_SIDE_CASTLE)
                    && (threat_or_non_empty.0 & 0x60) == 0
                {
                    moves.push(Move {
                        from: king_square as u8,
                        to: 6,
                        piece: Piece::King,
                        promotion: None,
                        flags: MoveFlags::CASTLE,
                    });
                }
                if board.flags.contains(BoardFlags::WHITE_QUEEN_SIDE_CASTLE)
                    && (threat_or_non_empty.0 & 0x0c) == 0
                    && (board.occupied.0 & 0x0e) == 0
                {
                    moves.push(Move {
                        from: king_square as u8,
                        to: 2,
                        piece: Piece::King,
                        promotion: None,
                        flags: MoveFlags::CASTLE,
                    });
                }
            }
            Color::Black => {
                if board.flags.contains(BoardFlags::BLACK_KING_SIDE_CASTLE)
                    && (threat_or_non_empty.0 & 0x6000000000000000) == 0
                {
                    moves.push(Move {
                        from: king_square as u8,
                        to: 62,
                        piece: Piece::King,
                        promotion: None,
                        flags: MoveFlags::CASTLE,
                    });
                }
                if board.flags.contains(BoardFlags::BLACK_QUEEN_SIDE_CASTLE)
                    && (threat_or_non_empty.0 & 0x0c00000000000000) == 0
                    && (board.occupied.0 & 0x0e00000000000000) == 0
                {
                    moves.push(Move {
                        from: king_square as u8,
                        to: 58,
                        piece: Piece::King,
                        promotion: None,
                        flags: MoveFlags::CASTLE,
                    });
                }
            }
        }
    }

    // Convert king position to 8x8
    let king_position_8x8 = (king_square % 8, king_square / 8);

    // Remove all moves that are made by pinned pieces and that don't move along the ray of the pin
    moves.retain(|m| {
        if pinned_bitboard.get(m.from) {
            // Pinned can only move along the ray direction between the king and the pinner, to compute this direction
            let from_square_8x8 = (m.from % 8, m.from / 8);
            let to_square_8x8 = (m.to % 8, m.to / 8);

            // Only valid if the king_position_8x8, from_square_8x8 and to_square_8x8 are all aligned
            let from_dir_unit = (
                (from_square_8x8.0 as i8 - king_position_8x8.0 as i8).signum(),
                (from_square_8x8.1 as i8 - king_position_8x8.1 as i8).signum(),
            );

            let to_dir = (
                (to_square_8x8.0 as i8 - king_position_8x8.0 as i8),
                (to_square_8x8.1 as i8 - king_position_8x8.1 as i8),
            );

            let a = to_dir.0 * from_dir_unit.0; // >= 0, if 0 should ensure that to_dir.0 == 0 as well
            let b = to_dir.1 * from_dir_unit.1; // >= 0, if 0 should ensure that to_dir.1 == 0 as well

            if a == 0 {
                return (to_dir.0 == 0) && (from_dir_unit.0 == 0);
            } else if b == 0 {
                return (to_dir.1 == 0) && (from_dir_unit.1 == 0);
            } else if a != b {
                return false;
            }
        }

        true
    });
}
