use crate::bitboard::{Bitboard, square_to_algebraic};
use bitflags::bitflags;
use strum::{EnumIter, FromRepr, IntoEnumIterator};

bitflags! {
    /// Flag representing the state of the chessboard, including which player's turn it is and castling rights.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BoardFlags: u8 {
        const WHITE_TO_MOVE = 1 << 0;
        const WHITE_KING_SIDE_CASTLE = 1 << 1;
        const WHITE_QUEEN_SIDE_CASTLE = 1 << 2;
        const BLACK_KING_SIDE_CASTLE = 1 << 3;
        const BLACK_QUEEN_SIDE_CASTLE = 1 << 4;
        const WHITE_CASTLE = Self::WHITE_KING_SIDE_CASTLE.bits() | Self::WHITE_QUEEN_SIDE_CASTLE.bits();
        const BLACK_CASTLE = Self::BLACK_KING_SIDE_CASTLE.bits() | Self::BLACK_QUEEN_SIDE_CASTLE.bits();
        const CASTLE = Self::WHITE_CASTLE.bits() | Self::BLACK_CASTLE.bits();
    }
}

/// Represents the state of a chessboard, including the positions of all pieces and the current game state flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Board {
    /// An array of bitboards, where each bitboard represents the positions of a specific piece type
    ///
    /// The indexing of the bitboards is as follows:
    /// - Index 0: Pawn bitboard (both white and black pawns)
    /// - Index 1: Knight bitboard (both white and black knights)
    /// - Index 2: Bishop bitboard (both white and black bishops)
    /// - Index 3: Rook bitboard (both white and black rooks)
    /// - Index 4: Queen bitboard (both white and black queens)
    /// - Index 5: King bitboard (both white and black kings)
    pub bitboards: [Bitboard; 6],

    /// A bitboard representing all white pieces on the board (i.e. the union of all white piece bitboards).
    pub white: Bitboard,

    /// A bitboard representing all occupied squares on the board (i.e. the union of all piece bitboards).
    pub occupied: Bitboard,

    /// Flags representing the state of the board, such as which player's turn it is and castling rights.
    pub flags: BoardFlags,

    /// The file index (0-7) of the en passant target square, or any value outside that range
    /// if there is no en passant target square. For example, if a white pawn moves from e2 to e4,
    /// the en passant target square is e3, which corresponds to file index 4 (since files are
    /// indexed from 0 for 'a' to 7 for 'h').
    pub en_passant_square: u8,
}

impl Board {
    pub const DEFAULT_POSITION_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn verify(&self) -> bool {
        // Check no collision between board
        let mut a = Bitboard::empty();
        for bitboard in &self.bitboards {
            if (a & *bitboard).0 != 0 {
                return false;
            }
            a |= *bitboard;
        }

        // Check occupied is correct
        if a != self.occupied {
            return false;
        }

        // Check white is a subset of occupied
        if (self.white & self.occupied) != self.white {
            return false;
        }

        true
    }

    pub const fn empty() -> Self {
        Self {
            bitboards: [Bitboard::empty(); 6], // For Pawn, Knight, Bishop, Rook, Queen, King
            white: Bitboard::empty(),
            occupied: Bitboard::empty(),
            flags: BoardFlags::empty(),
            en_passant_square: 8,
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        SquareCentricBoard::parse_fen(fen).map(|square_centric| square_centric.into())
    }

    pub fn fen(&self) -> impl std::fmt::Display + 'static {
        let square_centric: SquareCentricBoard = (*self).into();
        square_centric.fen()
    }

    pub fn default_position() -> Self {
        Self::from_fen(Self::DEFAULT_POSITION_FEN)
            .expect("Default position FEN should always be valid")
    }

    pub fn get(&self, piece: Piece) -> &Bitboard {
        debug_assert!(
            piece.is_white(),
            "get() should only be called with white pieces, since the bitboards are colorless and the white bitboard is used to determine the color of pieces on the board"
        );
        &self.bitboards[piece.colorless() as usize]
    }

    pub fn get_mut(&mut self, piece: Piece) -> &mut Bitboard {
        debug_assert!(
            piece.is_white(),
            "get_mut() should only be called with white pieces, since the bitboards are colorless and the white bitboard is used to determine the color of pieces on the board"
        );
        &mut self.bitboards[piece.colorless() as usize]
    }

    pub fn friendly_bitboard(&self) -> Bitboard {
        if self.flags.contains(BoardFlags::WHITE_TO_MOVE) {
            self.white
        } else {
            self.occupied ^ self.white
        }
    }

    pub fn enemy_bitboard(&self) -> Bitboard {
        if self.flags.contains(BoardFlags::WHITE_TO_MOVE) {
            self.occupied ^ self.white
        } else {
            self.white
        }
    }

    pub fn next_to_move(&self) -> Color {
        if self.flags.contains(BoardFlags::WHITE_TO_MOVE) {
            Color::White
        } else {
            Color::Black
        }
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let square_centric: SquareCentricBoard = (*self).into();
        square_centric.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromRepr)]
#[repr(u8)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn from_boolean_is_white(white: bool) -> Self {
        if white { Color::White } else { Color::Black }
    }

    pub fn opposite(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub unsafe fn from_repr_unsafe(repr: u8) -> Self {
        debug_assert!(
            repr <= 1,
            "Invalid color representation: expected 0 or 1, got {}",
            repr
        );

        // SAFETY: The caller must ensure that repr is a valid representation of a Color (0 or 1).
        unsafe { Color::from_repr(repr).unwrap_unchecked() }
    }

    pub fn score_multiplier(self) -> f32 {
        match self {
            Color::White => 1.0,
            Color::Black => -1.0,
        }
    }

    pub fn infinity(self) -> f32 {
        match self {
            Color::White => f32::INFINITY,
            Color::Black => f32::NEG_INFINITY,
        }
    }

    pub fn minmax_ini(self) -> f32 {
        match self {
            Color::White => f32::NEG_INFINITY,
            Color::Black => f32::INFINITY,
        }
    }

    pub fn at_depth(self, depth: usize) -> Color {
        if depth % 2 == 0 {
            self
        } else {
            self.opposite()
        }
    }

    pub fn minmax(&self, a: f32, b: f32) -> f32 {
        match self {
            Color::White => a.max(b),
            Color::Black => a.min(b),
        }
    }

    pub fn minmax_cmp(&self, a: f32, b: f32) -> bool {
        match self {
            Color::White => a > b,
            Color::Black => a < b,
        }
    }

    /// Updates alpha and beta based on the given score and returns true if the current branch should be pruned.
    ///
    /// Alpha-beta pruning is a search algorithm that reduces the number of nodes evaluated in the search tree by
    /// keeping track of the best scores for both players (alpha for the maximizing player and beta for the
    /// minimizing player) and pruning branches that cannot possibly influence the final decision.
    ///
    pub fn alpha_beta_prune(&self, score: f32, alpha: &mut f32, beta: &mut f32) -> bool {
        match self {
            Color::White => {
                if score > *alpha {
                    *alpha = score;
                }
                *alpha >= *beta
            }
            Color::Black => {
                if score < *beta {
                    *beta = score;
                }
                *beta <= *alpha
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter, FromRepr)]
#[repr(u8)]
pub enum Piece {
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,

    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}

impl Piece {
    pub const FIRST_WHITE: Piece = Piece::WhitePawn;
    pub const LAST_WHITE: Piece = Piece::WhiteKing;
    pub const FIRST_BLACK: Piece = Piece::BlackPawn;
    pub const LAST_BLACK: Piece = Piece::BlackKing;

    #[allow(non_upper_case_globals)]
    pub const Pawn: Piece = Piece::WhitePawn; // Colorless pawn piece type (used for indexing bitboards)
    #[allow(non_upper_case_globals)]
    pub const Rook: Piece = Piece::WhiteRook; // Colorless rook piece type (used for indexing bitboards)
    #[allow(non_upper_case_globals)]
    pub const Knight: Piece = Piece::WhiteKnight; // Colorless knight piece type (used for indexing bitboards)
    #[allow(non_upper_case_globals)]
    pub const Bishop: Piece = Piece::WhiteBishop; // Colorless bishop piece type (used for indexing bitboards)
    #[allow(non_upper_case_globals)]
    pub const Queen: Piece = Piece::WhiteQueen; // Colorless queen piece type (used for indexing bitboards)
    #[allow(non_upper_case_globals)]
    pub const King: Piece = Piece::WhiteKing; // Colorless king piece type (used for indexing bitboards)

    pub unsafe fn from_repr_unsafe(repr: u8) -> Self {
        debug_assert!(
            repr <= 11,
            "Invalid piece representation: expected 0-11, got {}",
            repr
        );

        // SAFETY: The caller must ensure that repr is a valid representation of a Piece (0-11).
        unsafe { Piece::from_repr(repr).unwrap_unchecked() }
    }

    /// Returns the piece type without color information (e.g. WhitePawn and BlackPawn both return WhitePawn).
    pub fn colorless(self) -> Piece {
        // SAFETY: The Piece enum is guaranteed to have the white pieces in the first
        // 6 variants and the black pieces in the next 6 variants, so taking the repr
        // modulo 6 will always yield a valid piece type (pawn, knight, bishop, rook, queen, king).
        unsafe { Self::from_repr_unsafe(self as u8 % 6) }
    }

    /// Iterator colorless (e.g. WhitePawn and BlackPawn both yield a single WhitePawn variant).
    pub fn colorless_iter() -> impl Iterator<Item = Piece> {
        (0..6).map(|i| {
            // SAFETY: The Piece enum is guaranteed to have the white pieces in the first
            // 6 variants and the black pieces in the next 6 variants, so taking the
            // repr modulo 6 will always yield a valid piece type (pawn, knight, bishop, rook, queen, king).
            unsafe { Piece::from_repr_unsafe(i) }
        })
    }

    /// Returns the color of the piece (White or Black).
    pub fn color(self) -> Color {
        if self.is_white() {
            Color::White
        } else {
            Color::Black
        }
    }

    /// Convert this piece to the corresponding color
    /// (e.g. WhitePawn becomes BlackPawn, and BlackKnight becomes WhiteKnight).
    pub fn with_color(self, color: Color) -> Piece {
        let color_bit = match color {
            Color::White => 0,
            Color::Black => 6,
        };
        // SAFETY: The Piece enum is guaranteed to have the white pieces in the first
        // 6 variants and the black pieces in the next 6 variants, so adding a color_bit of
        // either 0 or 6 will always yield a valid piece variant.
        unsafe { Self::from_repr_unsafe((self as u8 % 6) + color_bit) }
    }
}

impl std::str::FromStr for Piece {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Piece::iter()
            .find(|piece| piece.symbol() == s)
            .ok_or_else(|| format!("Invalid piece symbol: '{}'", s))
    }
}

impl Piece {
    pub fn is_white(&self) -> bool {
        matches!(
            self,
            Piece::WhitePawn
                | Piece::WhiteKnight
                | Piece::WhiteBishop
                | Piece::WhiteRook
                | Piece::WhiteQueen
                | Piece::WhiteKing
        )
    }

    pub fn is_black(&self) -> bool {
        matches!(
            self,
            Piece::BlackPawn
                | Piece::BlackKnight
                | Piece::BlackBishop
                | Piece::BlackRook
                | Piece::BlackQueen
                | Piece::BlackKing
        )
    }

    pub fn is_king(&self) -> bool {
        matches!(self, Piece::WhiteKing | Piece::BlackKing)
    }

    pub fn is_queen(&self) -> bool {
        matches!(self, Piece::WhiteQueen | Piece::BlackQueen)
    }

    pub fn is_rook(&self) -> bool {
        matches!(self, Piece::WhiteRook | Piece::BlackRook)
    }

    pub fn is_bishop(&self) -> bool {
        matches!(self, Piece::WhiteBishop | Piece::BlackBishop)
    }

    pub fn is_knight(&self) -> bool {
        matches!(self, Piece::WhiteKnight | Piece::BlackKnight)
    }

    pub fn is_pawn(&self) -> bool {
        matches!(self, Piece::WhitePawn | Piece::BlackPawn)
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Piece::BlackPawn => "p",
            Piece::BlackKnight => "n",
            Piece::BlackBishop => "b",
            Piece::BlackRook => "r",
            Piece::BlackQueen => "q",
            Piece::BlackKing => "k",
            Piece::WhitePawn => "P",
            Piece::WhiteKnight => "N",
            Piece::WhiteBishop => "B",
            Piece::WhiteRook => "R",
            Piece::WhiteQueen => "Q",
            Piece::WhiteKing => "K",
        }
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

/// A representation of a square-centric representation of the chessboard, where each square
/// holds information about which piece is on it, if any. This is used for move generation and
/// other operations that require quick access to piece information on specific squares.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SquareCentricBoard {
    pub squares: [Option<Piece>; 64],
    pub flags: BoardFlags,
    pub en_passant_square: u8,
}

impl From<SquareCentricBoard> for Board {
    fn from(value: SquareCentricBoard) -> Self {
        let mut board = Board::empty();
        for (index, square) in value.squares.iter().enumerate() {
            if let Some(piece) = square {
                board.bitboards[piece.colorless() as usize].set(index as u8);
                board.occupied.set(index as u8);
                if piece.is_white() {
                    board.white.set(index as u8);
                }
            }
        }
        board.flags = value.flags;
        board.en_passant_square = value.en_passant_square;
        board
    }
}

impl From<Board> for SquareCentricBoard {
    fn from(value: Board) -> Self {
        assert!(
            value.verify(),
            "Invalid board state: overlapping pieces detected"
        );

        let mut squares = [None; 64];
        for piece in Piece::colorless_iter() {
            let bitboard = value.bitboards[piece as usize];

            for square_index in bitboard.scan() {
                let color = Color::from_boolean_is_white(value.white.get(square_index));
                squares[square_index as usize] = Some(piece.with_color(color));
            }
        }

        SquareCentricBoard {
            squares,
            en_passant_square: value.en_passant_square,
            flags: value.flags,
        }
    }
}

impl std::fmt::Display for SquareCentricBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            if f.alternate() {
                let letter = (b'1' + rank) as char;
                write!(f, "{}   ", letter)?;
            }

            for file in 0..8 {
                let index = rank * 8 + file;
                let symbol = match self.squares[index as usize] {
                    Some(piece) => piece.symbol(),
                    None => "·",
                };
                write!(f, "{} ", symbol)?;
            }
            writeln!(f)?;
        }

        if f.alternate() {
            write!(f, "\n    ")?;
            for file in 0..8 {
                let number = (b'A' + file) as char;
                write!(f, "{} ", number)?;
            }
        }

        Ok(())
    }
}

impl SquareCentricBoard {
    pub const fn empty() -> Self {
        Self {
            squares: [None; 64],
            en_passant_square: 8,
            flags: BoardFlags::empty(),
        }
    }

    pub fn default_position() -> Self {
        Self::parse_fen(Board::DEFAULT_POSITION_FEN)
            .expect("Default position FEN should always be valid")
    }

    pub fn parse_fen(fen: &str) -> Result<Self, String> {
        let mut board = SquareCentricBoard::empty();
        let mut rank = 7;
        let mut file = 0;
        let mut meta_index = 0;

        for c in fen.chars() {
            if meta_index >= 1 {
                if c.is_ascii_whitespace() {
                    meta_index += 1;
                } else if meta_index == 1 {
                    match c {
                        'w' => board.flags |= BoardFlags::WHITE_TO_MOVE,
                        'b' => board.flags.remove(BoardFlags::WHITE_TO_MOVE),
                        _ => {
                            return Err(format!(
                                "Invalid FEN: expected 'w' or 'b' for active color, got '{}'",
                                c
                            ));
                        }
                    }
                } else if meta_index == 2 {
                    // Castling rights
                    match c {
                        'K' => board.flags |= BoardFlags::WHITE_KING_SIDE_CASTLE,
                        'Q' => board.flags |= BoardFlags::WHITE_QUEEN_SIDE_CASTLE,
                        'k' => board.flags |= BoardFlags::BLACK_KING_SIDE_CASTLE,
                        'q' => board.flags |= BoardFlags::BLACK_QUEEN_SIDE_CASTLE,
                        '-' => {} // No castling rights
                        _ => {
                            return Err(format!(
                                "Invalid FEN: unexpected character '{}' in castling rights section",
                                c
                            ));
                        }
                    }
                } else if meta_index == 3 {
                    // En passant target square
                    if c == '-' {
                        board.en_passant_square = 64;
                    } else if c.is_ascii_alphabetic() && c.is_ascii_lowercase() {
                        board.en_passant_square = (c as u8 - b'a') % 8; // Convert file character to index (0-7)
                        if board.flags.contains(BoardFlags::WHITE_TO_MOVE) {
                            board.en_passant_square += 40;
                        } else {
                            board.en_passant_square += 16;
                        }
                    } else if c.is_digit(10) {
                    } else {
                        return Err(format!(
                            "Invalid FEN: unexpected character '{}' in en passant section",
                            c
                        ));
                    }
                } else if meta_index >= 4 {
                    continue;
                } else {
                    return Err(format!(
                        "Invalid FEN: unexpected character '{}' in metadata section (for meta_index {})",
                        c, meta_index
                    ));
                }
            } else {
                if c == '/' {
                    if file != 8 {
                        return Err(format!(
                            "Invalid FEN: expected 8 files per rank, got {}",
                            file
                        ));
                    }
                    rank -= 1;
                    file = 0;
                } else if c.is_ascii_whitespace() {
                    if file != 8 || rank != 0 {
                        return Err(format!(
                            "Invalid FEN: incomplete board representation, ended at rank {}, file {}",
                            7 - rank,
                            file
                        ));
                    }

                    meta_index = 1;
                } else if c.is_digit(10) {
                    file += c.to_digit(10).unwrap() as u8;
                } else {
                    if file >= 8 {
                        return Err(format!("Invalid FEN: too many files in rank {}", 7 - rank));
                    }
                    let piece = c.to_string().parse::<Piece>()?;
                    board.squares[(rank * 8 + file) as usize] = Some(piece);
                    file += 1;
                }
            }
        }

        Ok(board)
    }

    pub fn fen(self) -> impl std::fmt::Display + 'static {
        struct Fmt {
            board: SquareCentricBoard,
        }

        impl std::fmt::Display for Fmt {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                for rank in (0..8).rev() {
                    let mut empty_count = 0;
                    for file in 0..8 {
                        let index = rank * 8 + file;
                        match self.board.squares[index as usize] {
                            Some(piece) => {
                                if empty_count > 0 {
                                    write!(f, "{}", empty_count)?;
                                    empty_count = 0;
                                }
                                write!(f, "{}", piece.symbol())?;
                            }
                            None => empty_count += 1,
                        }
                    }
                    if empty_count > 0 {
                        write!(f, "{}", empty_count)?;
                    }
                    if rank > 0 {
                        write!(f, "/")?;
                    }
                }

                // Active color
                if self.board.flags.contains(BoardFlags::WHITE_TO_MOVE) {
                    write!(f, " w ")?;
                } else {
                    write!(f, " b ")?;
                }

                // Castling rights
                if self
                    .board
                    .flags
                    .contains(BoardFlags::WHITE_KING_SIDE_CASTLE)
                {
                    write!(f, "K")?;
                }
                if self
                    .board
                    .flags
                    .contains(BoardFlags::WHITE_QUEEN_SIDE_CASTLE)
                {
                    write!(f, "Q")?;
                }
                if self
                    .board
                    .flags
                    .contains(BoardFlags::BLACK_KING_SIDE_CASTLE)
                {
                    write!(f, "k")?;
                }
                if self
                    .board
                    .flags
                    .contains(BoardFlags::BLACK_QUEEN_SIDE_CASTLE)
                {
                    write!(f, "q")?;
                }
                if !self.board.flags.intersects(BoardFlags::CASTLE) {
                    write!(f, "-")?;
                }

                // En passant target square
                if self.board.en_passant_square < 64 {
                    write!(f, " {}", square_to_algebraic(self.board.en_passant_square))?;
                } else {
                    write!(f, " -")?;
                }

                Ok(())
            }
        }

        Fmt { board: self }
    }
}
