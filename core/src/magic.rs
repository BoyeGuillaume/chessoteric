use crate::bitboard::Bitboard;
#[cfg(not(debug_assertions))]
use rand::{RngExt, SeedableRng};

#[derive(Clone, Copy, Default)]
#[cfg(not(debug_assertions))]
struct SMagic {
    mask: u64,     // mask of relevant occupancy bits
    magic: u64,    // magic number, six upper bits as shift amount
    offset: usize, // offset in the attack table
    bits: u8,      // number of bits to shift the occupancy after multiplication
}

pub struct Magic {
    #[cfg(not(debug_assertions))]
    bishop: [SMagic; 64],
    #[cfg(not(debug_assertions))]
    rook: [SMagic; 64],
    #[cfg(not(debug_assertions))]
    attack_table: Vec<u64>,
}

impl Magic {
    fn bishop_legacy_raycast(square: u8, occ: Bitboard) -> u64 {
        Bitboard(1 << square).bishop_raycast(occ).0
    }

    fn rook_legacy_raycast(square: u8, occ: Bitboard) -> u64 {
        Bitboard(1 << square).rook_raycast(occ).0
    }

    #[cfg(not(debug_assertions))]
    fn index_to_bitboard(index: u64, bits: u8, mut mask: u64) -> u64 {
        let mut result = 0u64;
        for i in 0..bits {
            let b = mask ^ (mask - 1);
            mask &= mask - 1; // pop the first bit of the mask
            let j = b.ilog2();
            if (index & (1 << i)) != 0 {
                result |= 1 << j;
            }
        }
        result
    }

    #[cfg(not(debug_assertions))]
    fn transform(b: u64, magic: u64, bits: u8) -> usize {
        ((b.wrapping_mul(magic)) >> (64 - bits)) as usize
    }

    #[cfg(not(debug_assertions))]
    fn find_magic(
        square: u8,
        m: u8,
        is_bishop: bool,
        trial: usize,
        attack_table: &mut Vec<u64>,
        rng: &mut impl rand::Rng,
        mask_out: &mut u64,
    ) -> Option<u64> {
        let mut b: [u64; 4096] = [0; 4096];
        let mut a: [u64; 4096] = [0; 4096];
        let mut used: [u64; 4096] = [0; 4096];

        let mask = if is_bishop {
            // Mask last rank and file because we don't care about edges. When generating
            // attacks, we attack until the last hit therefore even if it is occupied it will be
            // included in the attack set, so we can ignore it in the mask.
            Self::bishop_legacy_raycast(square, Bitboard::empty())
                & !(Bitboard::RANK_1 | Bitboard::RANK_8 | Bitboard::FILE_A | Bitboard::FILE_H)
        } else {
            let mut filter = Bitboard::full().0;
            for mask in [
                Bitboard::RANK_1,
                Bitboard::RANK_8,
                Bitboard::FILE_A,
                Bitboard::FILE_H,
            ] {
                if (Bitboard(1 << square).0 & mask) == 0 {
                    filter &= !mask; // if the square is not on this rank/file, we can ignore it in the mask
                }
            }

            Self::rook_legacy_raycast(square, Bitboard::empty()) & filter
        };
        *mask_out = mask; // output the mask for later use in the SMagic struct

        let n = mask.count_ones();
        assert!(
            n <= 12,
            "Too many relevant bits for square {}, got n {}, mask {}",
            square,
            n,
            mask
        ); // sanity check to avoid overflow in the arrays
        for i in 0..(1 << n) {
            b[i] = Self::index_to_bitboard(i as u64, n as u8, mask);
            a[i] = if is_bishop {
                Self::bishop_legacy_raycast(square, Bitboard(b[i]))
            } else {
                Self::rook_legacy_raycast(square, Bitboard(b[i]))
            };
        }

        'trial_loop: for _ in 0..trial {
            let magic = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>(); // sparse random number
            if ((mask.wrapping_mul(magic)) & 0xFF00000000000000) < 6 {
                continue; // ensure the upper 8 bits have at least 6 bits set (this will become our shift amount)
            }

            used.fill(0);
            for i in 0..(1 << n) {
                let j = Self::transform(b[i], magic, m);
                if used[j] == 0 {
                    used[j] = a[i];
                } else if used[j] != a[i] {
                    continue 'trial_loop; // collision, try another magic number
                }
            }

            // Fill the attack table entries for this magic number (now that we found one without collisions)
            let max_used = used.iter().rposition(|x| *x != 0).unwrap();
            attack_table.extend_from_slice(&used[..=max_used]); // add the new entries to the attack table

            // Return the found magic number and the corresponding attack table entries
            return Some(magic);
        }

        // If no magic number is found after the specified number of trials, return None
        None
    }

    #[cfg(not(debug_assertions))]
    pub fn generate() -> Self {
        const TRIALS: usize = 100000000;
        const ROOK_BITS: [u8; 64] = [
            12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10,
            10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10,
            10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
        ];
        const BISHOP_BITS: [u8; 64] = [
            6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9,
            7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5,
            5, 5, 5, 5, 5, 6,
        ];
        let mut fast_rng = rand::rngs::SmallRng::from_rng(&mut rand::rng());

        // First we generate magic numbers for bishops and rooks, and store the corresponding attack sets in the attack table
        let mut attack_table: Vec<u64> = Vec::new();
        let mut bishop: [SMagic; 64] = [SMagic {
            mask: 0,
            magic: 0,
            offset: 0,
            bits: 0,
        }; 64];
        let mut rook: [SMagic; 64] = [SMagic {
            mask: 0,
            magic: 0,
            offset: 0,
            bits: 0,
        }; 64];

        for i in 0..64 {
            let offset = attack_table.len();

            bishop[i].magic = Self::find_magic(
                i as u8,
                BISHOP_BITS[i],
                true,
                TRIALS,
                &mut attack_table,
                &mut fast_rng,
                &mut bishop[i].mask, // output the mask for this square
            )
            .expect("Failed to find a magic number for bishop");
            bishop[i].offset = offset;
            bishop[i].bits = BISHOP_BITS[i];
        }

        for i in 0..64 {
            let offset = attack_table.len();

            rook[i].magic = Self::find_magic(
                i as u8,
                ROOK_BITS[i],
                false,
                TRIALS,
                &mut attack_table,
                &mut fast_rng,
                &mut rook[i].mask, // output the mask for this square
            )
            .expect("Failed to find a magic number for rook");
            rook[i].offset = offset;
            rook[i].bits = ROOK_BITS[i];
        }

        Self {
            bishop,
            rook,
            attack_table,
        }
    }

    #[cfg(debug_assertions)]
    pub fn generate() -> Self {
        Self {}
    }

    #[cfg(debug_assertions)]
    pub fn bishop_raycast(&self, square: u8, occupancy: Bitboard) -> Bitboard {
        Bitboard(Self::bishop_legacy_raycast(square, occupancy))
    }

    #[cfg(not(debug_assertions))]
    pub fn bishop_raycast(&self, square: u8, mut occupancy: Bitboard) -> Bitboard {
        let entry = &self.bishop[square as usize];
        occupancy.0 &= entry.mask;
        occupancy.0 = occupancy.0.wrapping_mul(entry.magic);
        occupancy.0 >>= 64 - entry.bits;
        Bitboard(self.attack_table[entry.offset + occupancy.0 as usize])
    }

    #[cfg(debug_assertions)]
    pub fn rook_raycast(&self, square: u8, occupancy: Bitboard) -> Bitboard {
        Bitboard(Self::rook_legacy_raycast(square, occupancy))
    }

    #[cfg(not(debug_assertions))]
    pub fn rook_raycast(&self, square: u8, mut occupancy: Bitboard) -> Bitboard {
        let entry = &self.rook[square as usize];
        occupancy.0 &= entry.mask;
        occupancy.0 = occupancy.0.wrapping_mul(entry.magic);
        occupancy.0 >>= 64 - entry.bits;
        Bitboard(self.attack_table[entry.offset + occupancy.0 as usize])
    }
}

#[cfg(test)]
mod tests {
    use rand::{RngExt, SeedableRng};

    use super::*;

    #[test]
    fn test_magic_rook() {
        let magic = Magic::generate();
        let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(0x42);

        // For each square, we test the rook attacks against all possible occupancy combinations
        for square in 0..64 {
            for trials in 0..1000 {
                let mut occupency = seeded_rng.random::<u64>();
                for _ in 0..(trials % 3) {
                    occupency &= seeded_rng.random::<u64>(); // make it more sparse
                }

                // Calculate the expected attacks using the legacy sliding attack function
                let expected = Magic::rook_legacy_raycast(square as u8, Bitboard(occupency));
                let actual = magic.rook_raycast(square, Bitboard(occupency)).0;

                assert_eq!(
                    expected, actual,
                    "Rook attacks mismatch for square {} with occupancy {:064b}",
                    square, occupency
                );
            }
        }
    }

    #[test]
    fn test_magic_bishop() {
        let magic = Magic::generate();
        let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(0x42);

        // For each square, we test the bishop attacks against all possible occupancy combinations
        for square in 0..64 {
            for trials in 0..1000 {
                let mut occupency = seeded_rng.random::<u64>();
                for _ in 0..(trials % 3) {
                    occupency &= seeded_rng.random::<u64>(); // make it more sparse 
                }

                // Calculate the expected attacks using the legacy sliding attack function
                let expected = Magic::bishop_legacy_raycast(square as u8, Bitboard(occupency));
                let actual = magic.bishop_raycast(square, Bitboard(occupency)).0;

                assert_eq!(
                    expected, actual,
                    "Bishop attacks mismatch for square {} with occupancy {:064b}",
                    square, occupency
                );
            }
        }
    }
}
