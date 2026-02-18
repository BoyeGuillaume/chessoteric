use chessoteric_core::{
    bitboard::Bitboard, magic::Magic, moves::generate_moves, study::StudyEntry,
};
use criterion::{Criterion, criterion_group, criterion_main};
use rand::{Rng, SeedableRng};

fn bench_move_generation(c: &mut Criterion, studies: Vec<StudyEntry>) {
    for study in studies.iter() {
        let board = chessoteric_core::board::Board::from_fen(&study.start.fen).unwrap();

        c.bench_function(&format!("Move Generation: \"{}\"", study.start.fen), |b| {
            b.iter(|| {
                let mut moves = vec![];
                let mut currently_in_check = false;
                generate_moves(&board, &mut moves, &mut currently_in_check);
                std::hint::black_box(moves);
            });
        });
    }
}

fn rook_bishop_raycast_bench(c: &mut Criterion) {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0x42);
    let magic = Magic::generate();

    c.bench_function("rook_raycast", |b| {
        b.iter(|| {
            let nextu32 = rng.next_u32();
            let filter_count = (nextu32 % 3) as usize; // Not uniform, but good enough for our purposes
            let square = ((nextu32 >> 2) % 64) as u8;
            let mut occupency = Bitboard(rng.next_u64());
            for _ in 0..filter_count {
                occupency.0 &= rng.next_u64();
            }

            let value = magic.rook_raycast(square, occupency);
            std::hint::black_box(value);
        });
    });

    c.bench_function("bishop_raycast", |b| {
        b.iter(|| {
            let nextu32 = rng.next_u32();
            let filter_count = (nextu32 % 3) as usize; // Not uniform, but good enough for our purposes
            let square = ((nextu32 >> 2) % 64) as u8;
            let mut occupency = Bitboard(rng.next_u64());
            for _ in 0..filter_count {
                occupency.0 &= rng.next_u64();
            }

            let value = magic.bishop_raycast(square, occupency);
            // let value = Bitboard(1 << square).bishop_raycast(occupency).0;
            std::hint::black_box(value);
        });
    });
}

fn bench_famous_move_generation(c: &mut Criterion) {
    let studies = chessoteric_core::study::get_famous_study();
    bench_move_generation(c, studies);
}

fn bench_standard_move_generation(c: &mut Criterion) {
    let studies = chessoteric_core::study::get_standard_study();
    bench_move_generation(c, studies);
}

criterion_group!(
    move_generation_benches,
    bench_famous_move_generation,
    bench_standard_move_generation,
    rook_bishop_raycast_bench,
);
criterion_main!(move_generation_benches);
