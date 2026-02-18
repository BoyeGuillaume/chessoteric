use chessoteric_core::{moves::generate_moves, study::StudyEntry};
use criterion::{Criterion, criterion_group, criterion_main};
use pretty_assertions::assert_eq;

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

// fn bench_castling_move_generation(c: &mut Criterion) {
//     let studies = chessoteric_core::study::get_castling_study();
//     bench_move_generation(c, studies);
// }

// fn bench_checkmates_move_generation(c: &mut Criterion) {
//     let studies = chessoteric_core::study::get_checkmates_study();
//     bench_move_generation(c, studies);
// }

fn bench_famous_move_generation(c: &mut Criterion) {
    let studies = chessoteric_core::study::get_famous_study();
    bench_move_generation(c, studies);
}

// fn bench_pawns_move_generation(c: &mut Criterion) {
//     let studies = chessoteric_core::study::get_pawns_study();
//     bench_move_generation(c, studies);
// }

// fn bench_promotions_move_generation(c: &mut Criterion) {
//     let studies = chessoteric_core::study::get_promotions_study();
//     bench_move_generation(c, studies);
// }

// fn bench_stalemates_move_generation(c: &mut Criterion) {
//     let studies = chessoteric_core::study::get_stalemates_study();
//     bench_move_generation(c, studies);
// }

fn bench_standard_move_generation(c: &mut Criterion) {
    let studies = chessoteric_core::study::get_standard_study();
    bench_move_generation(c, studies);
}

// fn bench_taxing_move_generation(c: &mut Criterion) {
//     let studies = chessoteric_core::study::get_taxing_study();
//     bench_move_generation(c, studies);
// }

criterion_group!(
    move_generation_benches,
    // bench_castling_move_generation,
    // bench_checkmates_move_generation,
    bench_famous_move_generation,
    // bench_pawns_move_generation,
    // bench_promotions_move_generation,
    // bench_stalemates_move_generation,
    bench_standard_move_generation,
    // bench_taxing_move_generation
);
criterion_main!(move_generation_benches);
