#[test]
fn test_fen() {
    use chessoteric_core::board::Board;

    let fen = "r1bqkbnr/pppppppp/2n5/8/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq -"; // Technically NOT a valid FEN
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(board.fen().to_string(), fen);
}
