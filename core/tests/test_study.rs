#[cfg(feature = "study")]
mod study {
    use chessoteric_core::{moves::generate_moves, study::StudyEntry};
    use pretty_assertions::assert_eq;

    fn handle_test_study(study: StudyEntry) {
        let board = chessoteric_core::board::Board::from_fen(&study.start.fen).unwrap();

        // For each expected move, we would generate the legal moves from the board position
        // and check if the expected move is among them. This is a placeholder for the actual move generation and validation logic.
        let mut moves = vec![];
        let mut currently_in_check = false;
        generate_moves(&board, &mut moves, &mut currently_in_check);

        let mut generated_moves = moves
            .iter()
            .map(|m| m.algebraic_notation(&board, &moves).to_string())
            .collect::<Vec<_>>();
        generated_moves.sort();

        let mut expected_moves = study
            .expected
            .iter()
            .map(|e| e.r#move.clone())
            .collect::<Vec<_>>();
        expected_moves.sort();

        assert_eq!(
            generated_moves, expected_moves,
            "Generated moves do not match expected moves for study: \"{}\"",
            study.start.fen
        );
    }

    #[test]
    fn test_castling_move_generation() {
        let studies = chessoteric_core::study::get_castling_study();
        for (index, study) in studies.iter().enumerate() {
            println!("===============================");
            println!("Testing study {}: \"{}\"", index + 1, study.start.fen);
            handle_test_study(study.clone());
        }
    }

    #[test]
    fn test_checkmates_move_generation() {
        let studies = chessoteric_core::study::get_checkmates_study();
        for (index, study) in studies.iter().enumerate() {
            println!("===============================");
            println!("Testing study {}: \"{}\"", index + 1, study.start.fen);
            handle_test_study(study.clone());
        }
    }

    #[test]
    fn test_famous_move_generation() {
        let studies = chessoteric_core::study::get_famous_study();
        for (index, study) in studies.iter().enumerate() {
            println!("===============================");
            println!("Testing study {}: \"{}\"", index + 1, study.start.fen);
            handle_test_study(study.clone());
        }
    }

    #[test]
    fn test_pawns_move_generation() {
        let studies = chessoteric_core::study::get_pawns_study();
        for (index, study) in studies.iter().enumerate() {
            println!("===============================");
            println!("Testing study {}: \"{}\"", index + 1, study.start.fen);
            handle_test_study(study.clone());
        }
    }

    #[test]
    fn test_promotions_move_generation() {
        let studies = chessoteric_core::study::get_promotions_study();
        for (index, study) in studies.iter().enumerate() {
            println!("===============================");
            println!("Testing study {}: \"{}\"", index + 1, study.start.fen);
            handle_test_study(study.clone());
        }
    }

    #[test]
    fn test_stalemates_move_generation() {
        let studies = chessoteric_core::study::get_stalemates_study();
        for (index, study) in studies.iter().enumerate() {
            println!("===============================");
            println!("Testing study {}: \"{}\"", index + 1, study.start.fen);
            handle_test_study(study.clone());
        }
    }

    #[test]
    fn test_standard_move_generation() {
        let studies = chessoteric_core::study::get_standard_study();
        for (index, study) in studies.iter().enumerate() {
            println!("===============================");
            println!("Testing study {}: \"{}\"", index + 1, study.start.fen);
            handle_test_study(study.clone());
        }
    }

    #[test]
    fn test_taxing_move_generation() {
        let studies = chessoteric_core::study::get_taxing_study();
        for (index, study) in studies.iter().enumerate() {
            println!("===============================");
            println!("Testing study {}: \"{}\"", index + 1, study.start.fen);
            handle_test_study(study.clone());
        }
    }
}
