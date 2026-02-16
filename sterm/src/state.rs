use chessoteric_core::ai::{Ai, get_ai};
use clap::Parser;

use crate::StermArgs;

pub struct AppState {
    pub args: StermArgs,
    pub board: chessoteric_core::board::Board,
    pub ai: Option<Box<dyn Ai>>,
    pub time_per_move: std::time::Duration,
}

pub trait Command {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    fn execute(&self, state: &mut AppState, args: &[String]);
}

pub fn all_commands() -> Vec<Box<dyn Command>> {
    vec![
        Box::new(ResetCommand),
        Box::new(ExitCommand),
        Box::new(LoadAiCommand),
        Box::new(DisplayBoardCommand),
        Box::new(MoveCommand),
        Box::new(ListMovesCommand),
        Box::new(GenerateCommand),
        Box::new(ColorCommand),
        Box::new(SetThinkTimeCommand),
    ]
}

pub struct ResetCommand;
impl Command for ResetCommand {
    fn name(&self) -> &str {
        "reset"
    }

    fn description(&self) -> &str {
        "Reset the board to the initial position"
    }

    fn execute(&self, state: &mut AppState, args: &[String]) {
        if args.len() > 2 {
            eprintln!("Usage: reset [FEN]");
            return;
        }
        let fen = if args.len() == 2 {
            &args[1]
        } else {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        };

        match chessoteric_core::board::Board::from_fen(fen) {
            Ok(board) => {
                state.board = board;
                if !state.args.no_output {
                    println!("Board reset to:\n{}", state.board);
                }
            }
            Err(_) => eprintln!("Invalid FEN string"),
        }
    }
}

pub struct ExitCommand;
impl Command for ExitCommand {
    fn name(&self) -> &str {
        "exit"
    }

    fn description(&self) -> &str {
        "Exit the application"
    }

    fn execute(&self, _state: &mut AppState, _args: &[String]) {
        std::process::exit(0);
    }
}

pub struct LoadAiCommand;
impl Command for LoadAiCommand {
    fn name(&self) -> &str {
        "load_ai"
    }

    fn description(&self) -> &str {
        "Load an AI by name (e.g. 'random')"
    }

    fn execute(&self, state: &mut AppState, args: &[String]) {
        if args.len() != 2 {
            eprintln!("Usage: load_ai <ai_name>");
            return;
        }
        let ai_name = &args[1];
        match get_ai(ai_name) {
            Some(ai) => {
                state.ai = Some(ai);
                if !state.args.no_output {
                    println!("Loaded AI: {}", ai_name);
                }
            }
            None => eprintln!("Unknown AI name: {}", ai_name),
        }
    }
}

pub struct DisplayBoardCommand;
impl Command for DisplayBoardCommand {
    fn name(&self) -> &str {
        "display"
    }

    fn description(&self) -> &str {
        "Display the current board position"
    }

    fn execute(&self, state: &mut AppState, _args: &[String]) {
        #[derive(Parser)]
        struct Args {
            #[clap(short, long)]
            fen: bool,
        }

        let args = match Args::try_parse_from(_args) {
            Ok(args) => args,
            Err(e) => {
                eprintln!("Error parsing arguments: {}", e);
                return;
            }
        };

        if args.fen {
            println!("{}", state.board.fen());
        } else {
            println!("{}", state.board);
        }
    }
}

pub struct MoveCommand;
impl Command for MoveCommand {
    fn name(&self) -> &str {
        "move"
    }

    fn description(&self) -> &str {
        "Make a move in UCI format (e.g. 'e2e4')"
    }

    fn execute(&self, state: &mut AppState, args: &[String]) {
        if args.len() != 2 {
            eprintln!("Usage: move <uci_move>");
            return;
        }
        let uci_move = &args[1];
        match chessoteric_core::moves::Move::from_uci(uci_move.as_str(), &state.board) {
            Some(mv) => {
                mv.apply(&mut state.board);
            }
            None => eprintln!("Invalid UCI move format"),
        }
    }
}

pub struct ListMovesCommand;
impl Command for ListMovesCommand {
    fn name(&self) -> &str {
        "list_moves"
    }

    fn description(&self) -> &str {
        "List all legal moves in the current position"
    }

    fn execute(&self, state: &mut AppState, _args: &[String]) {
        let mut moves = Vec::new();
        let mut currently_in_check = false;
        chessoteric_core::moves::generate_moves(&state.board, &mut moves, &mut currently_in_check);
        for mv in &moves {
            println!("{}", mv.algebraic_notation(&state.board, &moves));
        }
    }
}

pub struct SetThinkTimeCommand;
impl Command for SetThinkTimeCommand {
    fn name(&self) -> &str {
        "think_time"
    }

    fn description(&self) -> &str {
        "Set the time per move for the AI (in seconds)"
    }

    fn execute(&self, state: &mut AppState, args: &[String]) {
        if args.len() != 2 {
            eprintln!("Usage: think_time <seconds>");
            return;
        }
        match args[1].parse::<f64>() {
            Ok(seconds) => {
                state.time_per_move = std::time::Duration::from_secs_f64(seconds);
                if !state.args.no_output {
                    println!("Set AI think time to {} seconds", seconds);
                }
            }
            Err(_) => eprintln!("Invalid number format for seconds"),
        }
    }
}

pub struct GenerateCommand;
impl Command for GenerateCommand {
    fn name(&self) -> &str {
        "generate"
    }

    fn description(&self) -> &str {
        "Generate a move using the loaded AI"
    }

    fn execute(&self, state: &mut AppState, _args: &[String]) {
        if let Some(ai) = &mut state.ai {
            match ai.best_move(&state.board, state.time_per_move) {
                Some((mv, score)) => {
                    if state.args.no_output {
                        println!("{},{}", mv.uci(), score);
                    } else {
                        println!("AI recommends move: {} (score: {})", mv.uci(), score);
                    }
                }
                None => println!("AI has no move to recommend"),
            }
        } else {
            if state.args.no_output {
                std::process::exit(1);
            }

            eprintln!("No AI loaded. Use 'load_ai <ai_name>' to load an AI.");
        }
    }
}

pub struct ColorCommand;
impl Command for ColorCommand {
    fn name(&self) -> &str {
        "color"
    }

    fn description(&self) -> &str {
        "Display the color of the next player to move"
    }

    fn execute(&self, state: &mut AppState, _args: &[String]) {
        let color = match state.board.next_to_move() {
            chessoteric_core::board::Color::White => "white",
            chessoteric_core::board::Color::Black => "black",
        };
        if state.args.no_output {
            println!("{}", color);
        } else {
            println!("Next player to move: {}", color);
        }
    }
}
