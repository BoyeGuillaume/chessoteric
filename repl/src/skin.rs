use chessoteric_core::board::Piece;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Small,
    Compact,
    Extended,
    Large,
    ASCII,
}

impl DisplayMode {
    pub fn size(self) -> usize {
        match self {
            DisplayMode::Small => 1,
            DisplayMode::Compact => 2,
            DisplayMode::Extended => 3,
            DisplayMode::Large => 5,
            DisplayMode::ASCII => 1,
        }
    }
    
    pub(crate) fn from_size(size: usize) -> Self {
        match size {
            0..=2 => DisplayMode::Small,
            3..=4 => DisplayMode::Compact,
            5..=6 => DisplayMode::Extended,
            _ => DisplayMode::Large,
        }
    }
}

// From https://github.com/thomas-mauran/chess-tui
pub fn display_from_str(piece: Piece, display_mode: DisplayMode) -> &'static str {
    match display_mode {
        DisplayMode::Compact => {
            match piece.colorless() {
                Piece::Pawn => "\n ▟▙\n ██",
                Piece::Knight => "\n▞█▙\n ██",
                Piece::Bishop => "\n █x█\n ███",
                Piece::Rook => "\n ▙█▟\n ███",
                Piece::Queen => " ▲▲▲\n █◈█\n ███",
                Piece::King => "\n ▙✚▟\n ███",
                _ => unreachable!(),
            }
        }
        DisplayMode::Extended => {
            match piece.colorless() {
                Piece::Pawn => "   \n   \n ▟▙\n ██",
                Piece::Knight => " \n ▞█▙\n▞███\n ▗███▖",
                Piece::Bishop => " \n  █x█\n  ███\n ▗███▖",
                Piece::Rook => " \n ▙█▟\n ███\n▗███▖",
                Piece::Queen => "  ▲▲▲\n ◀█◈█▶\n ◥███◤\n ▗███▖",
                Piece::King => " \n  ▙█▟\n  ▙✚▟\n ▗███▖",
                _ => unreachable!(),
            }
        }
        DisplayMode::Large => {
            match piece.colorless() {
                Piece::Pawn => "\n\n   ▟█▙\n   ▜█▛\n  ▟███▙\n",
                Piece::Knight => "\n ▟▛██▙\n▟█████\n▀▀▟██▌\n ▟████\n",
                Piece::Bishop => "\n    ⭘\n   █x█\n   ███\n ▗█████▖\n",
                Piece::Rook => "\n █▟█▙█\n ▜███▛\n ▐███▌\n▗█████▖\n",
                Piece::Queen => "\n◀█▟█▙█▶\n ◥█◈█◤\n  ███\n▗█████▖\n",
                Piece::King => "   ✚\n ▞▀▄▀▚\n ▙▄█▄▟\n ▐███▌\n▗█████▖\n",
                _ => unreachable!(),
            }
        },
        DisplayMode::Small => {
            match piece.colorless() {
                Piece::Pawn => "♟",
                Piece::Knight => "♞",
                Piece::Bishop => "♝",
                Piece::Rook => "♜",
                Piece::Queen => "♛",
                Piece::King => "♚",
                _ => unreachable!(),
            }
        }
        DisplayMode::ASCII => piece.symbol(),
    }
}
