use crate::{
    bitboard::{Bitboard, Direction, square_to_algebraic},
    board::{Board, Color, Piece},
};
use bitflags::bitflags;

bitflags! {
    pub struct MoveFlags: u8 {
        const QUEEN_CASTLE = 1 << 0;
        const KING_CASTLE = 1 << 1;
        const CHECK = 1 << 2;
    }
}

pub struct Move {
    pub from: u8,
    pub to: u8,
    pub piece: Piece,
    pub promotion: Option<Piece>,
    pub flags: MoveFlags,
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
        | origin.sliding_attack(occlusion, Direction::NorthWest)
        | origin.sliding_attack(occlusion, Direction::SouthEast)
        | origin.sliding_attack(occlusion, Direction::SouthWest)
}

pub fn generate_rook_movement_xray(occlusion: Bitboard, origin: Bitboard) -> Bitboard {
    let attack = generate_rook_movement(occlusion, origin);
    let blockers = attack & occlusion;
    attack ^ generate_rook_movement(occlusion ^ blockers, origin)
}

pub fn generate_bishop_movement_xray(occlusion: Bitboard, origin: Bitboard) -> Bitboard {
    let attack = generate_bishop_movement(occlusion, origin);
    let blockers = attack & occlusion;
    attack ^ generate_bishop_movement(occlusion ^ blockers, origin)
}

pub fn generate_queen_movement(occlusion: Bitboard, origin: Bitboard) -> Bitboard {
    generate_rook_movement(occlusion, origin) | generate_bishop_movement(occlusion, origin)
}

pub fn generate_knight_movement(origin: Bitboard, ally: Bitboard) -> Bitboard {
    let l1 = (origin.0 >> 1) & 0x7f7f7f7f7f7f7f7f;
    let l2 = (origin.0 >> 2) & 0x3f3f3f3f3f3f3f3f;
    let r1 = (origin.0 << 1) & 0xfefefefefefefefe;
    let r2 = (origin.0 << 2) & 0xfcfcfcfcfcfcfcfc;
    let h1 = l1 | r1;
    let h2 = l2 | r2;
    let attacks = (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8);
    Bitboard(attacks & !ally.0)
}

pub fn generate_king_movement(origin: Bitboard, ally: Bitboard) -> Bitboard {
    let attacks = ((origin.0 << 8) | (origin.0 >> 8)) // North and South
        | ((origin.0 & !Bitboard::FILE_H) << 1) | ((origin.0 & !Bitboard::FILE_A) >> 1) // East and West
        | ((origin.0 & !Bitboard::FILE_H) << 9) | ((origin.0 & !Bitboard::FILE_A) >> 9) // NorthEast and SouthWest
        | ((origin.0 & !Bitboard::FILE_H) >> 7) | ((origin.0 & !Bitboard::FILE_A) << 7); // NorthWest and SouthEast
    Bitboard(attacks & !ally.0)
}

pub fn generate_simple_pawn_movement(
    origin: Bitboard,
    occlusion: Bitboard,
    color: Color,
) -> Bitboard {
    match color {
        Color::White => {
            let one_square_forward = (origin.0 << 8) & !occlusion.0;
            let two_squares_forward = ((one_square_forward & Bitboard::RANK_3) << 8) & !occlusion.0;
            Bitboard(one_square_forward | two_squares_forward)
        }
        Color::Black => {
            let one_square_forward = (origin.0 >> 8) & !occlusion.0;
            let two_squares_forward = ((one_square_forward & Bitboard::RANK_6) >> 8) & !occlusion.0;
            Bitboard(one_square_forward | two_squares_forward)
        }
    }
}

pub fn generate_threat(board: &Board) {}

pub fn generate_moves(board: &Board, moves: &mut Vec<Move>) {
    let rook_like = *board.get(Piece::Rook) | *board.get(Piece::Queen);
    let bishop_like = *board.get(Piece::Bishop) | *board.get(Piece::Queen);
    let knight_like = *board.get(Piece::Knight);
    let rook_like_enemy = rook_like & board.enemy_bitboard();
    let bishop_like_enemy = bishop_like & board.enemy_bitboard();
    let knight_like_enemy = knight_like & board.enemy_bitboard();
    let rook_like_friendly = rook_like & board.friendly_bitboard();
    let bishop_like_friendly = bishop_like & board.friendly_bitboard();

    // A list of all of the squares that pieces can move to, except for king moves
    let mut destination_filter_outside_king = Bitboard::full();
    let mut pinned_bitboard = Bitboard::empty();

    // If currently in check, we need to filter out any moves that don't block the check or move the king
    let ally_king_bitboard = *board.get(Piece::King) & board.friendly_bitboard();
    debug_assert!(
        ally_king_bitboard.count_ones() == 1,
        "There should be exactly one king for the side to move"
    );
    let king_square = ally_king_bitboard.square();

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
                generate_rook_movement_xray(board.occupied, checker_square);
        }

        // Bitscan over the king_bishop_checkers and update the allowed_destinations as needed
        for checker_square in king_bishop_checkers.scan_bitboard() {
            debug_assert!(
                checker_square.count_ones() == 1,
                "Checker square should have exactly one bit set"
            );
            destination_filter_outside_king &=
                generate_bishop_movement_xray(board.occupied, checker_square);
        }

        // Compute x-ray attacks for pinned pieces, to do so, we perform a ray attack, where the occlusion as been modified
        // to remove the piece being considered
        let king_rook_friendly_blocker = king_rook_ray & board.friendly_bitboard();
        let king_bishop_friendly_blocker = king_bishop_ray & board.friendly_bitboard();

        let king_rook_xray = generate_rook_movement_xray(
            board.occupied & !king_rook_friendly_blocker,
            ally_king_bitboard,
        ) & !king_rook_ray;
        let king_bishop_xray = generate_bishop_movement_xray(
            board.occupied & !king_bishop_friendly_blocker,
            ally_king_bitboard,
        ) & !king_bishop_ray;

        // Consider knight checks as well, which are simpler since they can't be blocked
        let king_knight_checkers =
            generate_knight_movement(ally_king_bitboard, board.friendly_bitboard());
        for checker_square in (king_knight_checkers & knight_like_enemy).scan_bitboard() {
            debug_assert!(
                checker_square.count_ones() == 1,
                "Checker square should have exactly one bit set"
            );
            destination_filter_outside_king &= checker_square; // Only the square the knight is on can be moved to, to capture the knight
        }

        // TODO: Figure out PINNING
    }

    // Generate pawn moves
    let friendly_pawns = *board.get(Piece::Pawn) & board.friendly_bitboard();
    let pawn_non_capture_moves =
        generate_simple_pawn_movement(friendly_pawns, board.occupied, board.next_to_move());

    // Generate knight moves
}
