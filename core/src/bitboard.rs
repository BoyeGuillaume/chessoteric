use rand::rand_core::block;

use crate::board::Color;

/// Bitboard representation of a chess position. Each bit represents a square on the chessboard, with the
/// least significant bit representing the a1 square and the most significant bit representing the h8 square.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bitboard(pub u64);

impl Default for Bitboard {
    fn default() -> Self {
        Bitboard(0)
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl std::ops::BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl std::ops::BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl From<u64> for Bitboard {
    fn from(value: u64) -> Self {
        Bitboard(value)
    }
}

impl From<Bitboard> for u64 {
    fn from(bitboard: Bitboard) -> Self {
        bitboard.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Direction {
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
    North,
}

impl Into<u8> for Direction {
    fn into(self) -> u8 {
        self as u8
    }
}

impl Direction {
    pub fn vertical_opposite(&self) -> Self {
        match self {
            Direction::NorthEast => Direction::SouthEast,
            Direction::East => Direction::East,
            Direction::SouthEast => Direction::NorthEast,
            Direction::South => Direction::North,
            Direction::SouthWest => Direction::NorthWest,
            Direction::West => Direction::West,
            Direction::NorthWest => Direction::SouthWest,
            Direction::North => Direction::South,
        }
    }

    pub fn as_offset(&self) -> i8 {
        match self {
            Direction::NorthEast => 9,
            Direction::East => 1,
            Direction::SouthEast => -7,
            Direction::South => -8,
            Direction::SouthWest => -9,
            Direction::West => -1,
            Direction::NorthWest => 7,
            Direction::North => 8,
        }
    }

    pub fn shift(&self, index: u8) -> Option<u8> {
        let offset = self.as_offset();
        let result = if offset > 0 {
            index + offset as u8
        } else {
            index - (-offset) as u8
        };
        if result < 64 { Some(result) } else { None }
    }
}

impl std::fmt::Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let index = rank * 8 + file;
                let bit = (self.0 >> index) & 1;
                write!(f, "{}", if bit == 1 { "1 " } else { "· " })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Bitboard {
    pub const FILE: [u64; 8] = [
        0x0101010101010101,
        0x0202020202020202,
        0x0404040404040404,
        0x0808080808080808,
        0x1010101010101010,
        0x2020202020202020,
        0x4040404040404040,
        0x8080808080808080,
    ];

    pub const FILE_A: u64 = 0x0101010101010101;
    pub const FILE_B: u64 = 0x0202020202020202;
    pub const FILE_C: u64 = 0x0404040404040404;
    pub const FILE_D: u64 = 0x0808080808080808;
    pub const FILE_E: u64 = 0x1010101010101010;
    pub const FILE_F: u64 = 0x2020202020202020;
    pub const FILE_G: u64 = 0x4040404040404040;
    pub const FILE_H: u64 = 0x8080808080808080;

    pub const RANK: [u64; 8] = [
        0x00000000000000FF,
        0x000000000000FF00,
        0x0000000000FF0000,
        0x00000000FF000000,
        0x000000FF00000000,
        0x0000FF0000000000,
        0x00FF000000000000,
        0xFF00000000000000,
    ];

    pub const RANK_1: u64 = 0x00000000000000FF;
    pub const RANK_2: u64 = 0x000000000000FF00;
    pub const RANK_3: u64 = 0x0000000000FF0000;
    pub const RANK_4: u64 = 0x00000000FF000000;
    pub const RANK_5: u64 = 0x000000FF00000000;
    pub const RANK_6: u64 = 0x0000FF0000000000;
    pub const RANK_7: u64 = 0x00FF000000000000;
    pub const RANK_8: u64 = 0xFF00000000000000;

    pub const AVOID_WRAP: [u64; 8] = [
        0xfefefefefefefe00,
        0xfefefefefefefefe,
        0x00fefefefefefefe,
        0x00ffffffffffffff,
        0x007f7f7f7f7f7f7f,
        0x7f7f7f7f7f7f7f7f,
        0x7f7f7f7f7f7f7f00,
        0xffffffffffffff00,
    ];

    pub const SHIFT: [u32; 8] = [9, 1, 57, 56, 55, 63, 7, 8];

    /// Find the pawn connected mask for a bitboard (basically expand the bitboard in the north and south directions based on color).
    pub const fn connected_mask(self, color: Color) -> Self {
        match color {
            Color::White => {
                Bitboard(((self.0 >> 7) & !Self::FILE_A) | ((self.0 >> 9) & !Self::FILE_H))
            }
            Color::Black => {
                Bitboard(((self.0 << 7) & !Self::FILE_H) | ((self.0 << 9) & !Self::FILE_A))
            }
        }
    }

    /// Mask surrounding squares for a given bitboard (think of it as a king's move mask).
    pub const fn surrounding_mask(self) -> Self {
        Bitboard(
            ((self.0 << 8) | (self.0 >> 8)) // North and South
            | ((self.0 & !Bitboard::FILE_H) << 1) | ((self.0 & !Bitboard::FILE_A) >> 1) // East and West
            | ((self.0 & !Bitboard::FILE_H) << 9) | ((self.0 & !Bitboard::FILE_A) >> 9) // NorthEast and SouthWest
            | ((self.0 & !Bitboard::FILE_H) >> 7) | ((self.0 & !Bitboard::FILE_A) << 7), // NorthWest and SouthEast
        )
    }
    /// Generate an empty bitboard (i.e., a bitboard with all bits set to 0).
    pub const fn empty() -> Self {
        Bitboard(0)
    }

    /// Generate a full bitboard (i.e., a bitboard with all bits set to 1).
    pub const fn full() -> Self {
        Bitboard(u64::MAX)
    }

    /// Checks if the bitboard is empty (i.e., has no bits set).
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Shifts the bitboard in the specified direction, while preventing wrap-around across the edges of the board.
    pub const fn shift_one(self, direction: Direction) -> Self {
        let direction = direction as u8 as usize;
        Bitboard(self.0.rotate_left(Bitboard::SHIFT[direction]) & Bitboard::AVOID_WRAP[direction])
    }

    /// Performs an occluded fill in the specified direction, which is used to calculate sliding piece attacks while considering blockers.
    #[inline(always)]
    pub const fn occluded_fill(self, mut occlusion: Bitboard, direction: Direction) -> Self {
        let direction = direction as u8 as usize;
        let mut current = self.0;
        let mut fill = 0x0;

        if current != 0 {
            let r = Self::SHIFT[direction];
            occlusion.0 |= !Self::AVOID_WRAP[direction];

            while current != 0 {
                current = current.rotate_left(r) & !occlusion.0;
                fill |= current;
            }
        }

        Bitboard(fill)
    }

    /// An simple iterative function that calculates sliding pieces attacks by repeatedly shifting the bitboard
    /// in the specified direction until it hits an occlusion.
    #[inline(always)]
    pub const fn sliding_attack(self, occlusion: Bitboard, direction: Direction) -> Self {
        let next = self.occluded_fill(occlusion, direction);
        Bitboard(Bitboard(next.0 | self.0).shift_one(direction).0 | next.0)
    }

    /// Function for calculating bishop attacks using the sliding attack function in all four diagonal directions.
    pub fn bishop_raycast(self, occ: Bitboard) -> Self {
        self.sliding_attack(occ, Direction::NorthEast)
            | self.sliding_attack(occ, Direction::NorthWest)
            | self.sliding_attack(occ, Direction::SouthEast)
            | self.sliding_attack(occ, Direction::SouthWest)
    }

    pub fn rook_raycast(self, occ: Bitboard) -> Self {
        self.sliding_attack(occ, Direction::North)
            | self.sliding_attack(occ, Direction::East)
            | self.sliding_attack(occ, Direction::South)
            | self.sliding_attack(occ, Direction::West)
    }

    pub fn shift_south(self) -> Self {
        Bitboard(self.0 >> 8)
    }

    pub fn shift_north(self) -> Self {
        Bitboard(self.0 << 8)
    }

    pub const fn shift_east(self) -> Self {
        Bitboard((self.0 << 1) & !Bitboard::FILE_A)
    }

    pub const fn shift_west(self) -> Self {
        Bitboard((self.0 >> 1) & !Bitboard::FILE_H)
    }

    pub const fn shift_north_east(self) -> Self {
        Bitboard((self.0 << 9) & !Bitboard::FILE_A)
    }

    pub const fn shift_north_west(self) -> Self {
        Bitboard((self.0 << 7) & !Bitboard::FILE_H)
    }

    pub const fn shift_south_east(self) -> Self {
        Bitboard((self.0 >> 7) & !Bitboard::FILE_A)
    }

    pub const fn shift_south_west(self) -> Self {
        Bitboard((self.0 >> 9) & !Bitboard::FILE_H)
    }

    pub fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn scan_bitboard(&self) -> impl Iterator<Item = Bitboard> {
        struct Iter {
            bitboard: u64,
        }

        impl Iterator for Iter {
            type Item = Bitboard;

            fn next(&mut self) -> Option<Self::Item> {
                if self.bitboard == 0 {
                    None
                } else {
                    let previous = self.bitboard;
                    self.bitboard &= self.bitboard - 1; // Clear the least significant bit
                    Some(Bitboard(previous & (!self.bitboard))) // Return the bit that was just cleared
                }
            }
        }

        Iter { bitboard: self.0 }
    }

    pub fn scan(&self) -> impl Iterator<Item = u8> {
        struct Iter {
            bitboard: u64,
        }

        impl Iterator for Iter {
            type Item = u8;

            fn next(&mut self) -> Option<Self::Item> {
                if self.bitboard == 0 {
                    None
                } else {
                    let lsb_index = self.bitboard.ilog2() as u8; // Get the index of the least significant bit
                    self.bitboard &= !(1 << lsb_index); // Clear the least significant bit
                    Some(lsb_index)
                }
            }
        }

        Iter { bitboard: self.0 }
    }

    pub fn get(&self, index: u8) -> bool {
        (self.0 & (1 << index)) != 0
    }

    pub fn set(&mut self, index: u8) {
        self.0 |= 1 << index;
    }

    pub fn unset(&mut self, index: u8) {
        self.0 &= !(1 << index);
    }

    pub fn square(&self) -> u8 {
        debug_assert!(
            self.0.count_ones() == 1,
            "Bitboard must have exactly one bit set to get the square index."
        );
        self.0.ilog2() as u8
    }
}

pub fn square_to_algebraic(square: u8) -> String {
    let file = (square % 8) as u8;
    let rank = (square / 8) as u8;
    format!("{}{}", (b'a' + file) as char, rank + 1)
}

pub fn algebraic_to_square(algebraic: &str) -> Option<u8> {
    if algebraic.len() != 2 {
        return None;
    }
    let file = algebraic.chars().nth(0)?.to_ascii_lowercase();
    let rank = algebraic.chars().nth(1)?.to_digit(10)? as u8;

    if file < 'a' || file > 'h' || rank < 1 || rank > 8 {
        return None;
    }

    Some((rank - 1) * 8 + (file as u8 - b'a'))
}
