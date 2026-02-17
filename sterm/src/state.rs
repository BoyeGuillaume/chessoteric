use chessoteric_core::ai::{Ai, AiLimit, get_ai};
use clap::Parser;

use crate::StermArgs;

pub struct AppState {
    pub args: StermArgs,
    pub board: chessoteric_core::board::Board,
    pub ai: Option<Box<dyn Ai>>,
}

pub trait Command {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    fn execute(&self, state: &mut AppState, args: &[String]);
}

pub fn all_commands() -> Vec<Box<dyn Command>> {
    vec![
        Box::new(PositionCommand),
        Box::new(QuitCommand),
        Box::new(LoadAiCommand),
        Box::new(DisplayBoardCommand),
        Box::new(MoveCommand),
        Box::new(UciCommand),
        Box::new(ListMovesCommand),
        Box::new(GoCommand),
        Box::new(StopCommand),
        Box::new(ColorCommand),
        Box::new(UciNewGameCommand),
        Box::new(IsReadyCommand),
    ]
}

pub struct PositionCommand;
impl Command for PositionCommand {
    fn name(&self) -> &str {
        "position"
    }

    fn description(&self) -> &str {
        "Sets the position on the board using a FEN string or the starting position. Syntax: position [fen <fen_string> | startpos] moves <move1> <move2> ..."
    }

    fn execute(&self, state: &mut AppState, args: &[String]) {
        // Syntax if position [fen <fen_string> | startpos] moves <move1> <move2> ...
        if args.len() < 2 {
            eprintln!("Usage: position [fen <fen_string> | startpos] moves <move1> <move2> ...");
            return;
        }

        // We will parse the arguments in two steps
        let mut board = None;

        {
            let mut index = 1;
            while index < args.len() && args[index] != "moves" {
                match args[index].as_str() {
                    "fen" => {
                        // Parse until "moves" or end of args
                        let mut fen_parts = Vec::new();
                        index += 1;
                        while index < args.len() && args[index] != "moves" {
                            fen_parts.push(args[index].clone());
                            index += 1;
                        }

                        let fen = fen_parts.join(" ");
                        match chessoteric_core::board::Board::from_fen(&fen) {
                            Ok(parsed_board) => {
                                assert!(board.is_none(), "Multiple position specifications found");
                                board = Some(parsed_board);
                            }
                            Err(_) => {
                                eprintln!("Invalid FEN string");
                                return;
                            }
                        }
                    }
                    "moves" => {
                        break;
                    }
                    "startpos" => {
                        assert!(board.is_none(), "Multiple position specifications found");
                        board = Some(chessoteric_core::board::Board::default_position());
                        index += 1;
                    }
                    _ => {
                        eprintln!(
                            "Usage: position [fen <fen_string> | startpos] moves <move1> <move2>, unknown argument: {}",
                            args[index]
                        );
                        return;
                    }
                }
            }

            if index < args.len() && args[index] == "moves" {
                index += 1;
                if board.is_none() {
                    eprintln!("Position must be specified before moves");
                    return;
                }
                let board = board.as_mut().unwrap();
                for move_str in &args[index..] {
                    match chessoteric_core::moves::Move::from_uci(move_str.as_str(), board) {
                        Some(mv) => mv.apply(board),
                        None => {
                            eprintln!("Invalid move: {}", move_str);
                            return;
                        }
                    }
                }
            } else if board.is_none() {
                eprintln!(
                    "Required 'moves' keyword not found. Usage: position [fen <fen_string> | startpos] moves <move1> <move2> ..."
                );
                return;
            }
        }

        if board.is_none() {
            eprintln!(
                "No position specified. Usage: position [fen <fen_string> | startpos] moves <move1> <move2> ..."
            );
            return;
        }

        state.board = board.unwrap();
        if state.args.human {
            println!("Board reset to:\n{}", state.board);
        }
    }
}
pub struct QuitCommand;
impl Command for QuitCommand {
    fn name(&self) -> &str {
        "quit"
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
                if state.args.human {
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
        "d"
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
            // println!("{}", mv.algebraic_notation(&state.board, &moves));
            println!("{}", mv.uci());
        }
    }
}

pub struct GoCommand;
impl Command for GoCommand {
    fn name(&self) -> &str {
        "go"
    }

    fn description(&self) -> &str {
        "Generate a move using the loaded AI"
    }

    fn execute(&self, state: &mut AppState, args: &[String]) {
        const USAGE: &str = "Usage: go [movetime <milliseconds>] [depth <ply>] [wtime <milliseconds>] [btime <milliseconds>] [winc <milliseconds>] [binc <milliseconds>]";

        // let mut search_time = state.time_per_move;
        let mut movetime = None;
        let mut depth = None;
        let mut wtime = None;
        let mut btime = None;
        let mut winc = None;
        let mut binc = None;

        {
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "infinite" => {
                        movetime.take();
                        depth.take();
                        wtime.take();
                        btime.take();
                        winc.take();
                        binc.take();
                    }
                    "movetime" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", USAGE);
                            return;
                        }
                        let time_ms = match args[i + 1].parse::<u64>() {
                            Ok(time_ms) => time_ms,
                            Err(_) => {
                                eprintln!("Invalid movetime value: {}", args[i + 1]);
                                return;
                            }
                        };
                        movetime.replace(std::time::Duration::from_millis(time_ms));
                    }
                    "depth" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", USAGE);
                            return;
                        }
                        let depth_value = match args[i + 1].parse::<u16>() {
                            Ok(depth) => depth,
                            Err(_) => {
                                eprintln!("Invalid depth value: {}", args[i + 1]);
                                return;
                            }
                        };
                        depth.replace(depth_value);
                    }
                    "wtime" | "btime" | "winc" | "binc" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", USAGE);
                            return;
                        }
                        let time_ms = match args[i + 1].parse::<u64>() {
                            Ok(time_ms) => time_ms,
                            Err(_) => {
                                eprintln!("Invalid wtime value: {}", args[i + 1]);
                                return;
                            }
                        };
                        let duration = std::time::Duration::from_millis(time_ms);
                        match args[i].as_str() {
                            "wtime" => wtime.replace(duration),
                            "btime" => btime.replace(duration),
                            "winc" => winc.replace(duration),
                            "binc" => binc.replace(duration),
                            _ => unreachable!(),
                        };
                    }
                    _ => {
                        eprintln!("Unknown argument: {}", args[i]);
                        eprintln!("{}", USAGE);
                        eprintln!("Got command: {}", args.join(" "));
                    }
                }

                i += 2;
            }
        }

        if movetime.is_none()
            && depth.is_none()
            && (wtime.is_some() || btime.is_some() || winc.is_some() || binc.is_some())
        {
            let winc = winc.unwrap_or_else(|| std::time::Duration::from_millis(0));
            let binc = binc.unwrap_or_else(|| std::time::Duration::from_millis(0));
            let wtime = wtime.unwrap_or_else(|| std::time::Duration::from_millis(0));
            let btime = btime.unwrap_or_else(|| std::time::Duration::from_millis(0));

            let next_to_move = state.board.next_to_move();
            let time_for_move = match next_to_move {
                chessoteric_core::board::Color::White => {
                    wtime.checked_div(30).unwrap_or_default() + winc
                }
                chessoteric_core::board::Color::Black => {
                    btime.checked_div(30).unwrap_or_default() + binc
                }
            };

            // We consider that we will need 30 move in the future
            movetime.replace(time_for_move);
        }

        let limit = AiLimit { movetime, depth };
        if let Some(ai) = &mut state.ai {
            ai.start(&state.board, limit, true);
        } else {
            if !state.args.human {
                std::process::exit(1);
            }

            eprintln!("No AI loaded. Use 'load_ai <ai_name>' to load an AI.");
        }
    }
}

pub struct StopCommand;
impl Command for StopCommand {
    fn name(&self) -> &str {
        "stop"
    }

    fn description(&self) -> &str {
        "Stop the AI from thinking"
    }

    fn execute(&self, state: &mut AppState, args: &[String]) {
        if args.len() != 1 {
            eprintln!("Usage: stop");
            return;
        }

        if let Some(ai) = &mut state.ai {
            ai.stop();

            // match ai.stop() {
            //     Some(result) => {
            //         if result.pv.len() > 1 {
            //             println!(
            //                 "bestmove {} ponder {}",
            //                 result.best_move.uci(),
            //                 result.pv[1].uci()
            //             );
            //         } else {
            //             println!("bestmove {}", result.best_move.uci());
            //         }
            //         // println!("Best move: {}, score: {}", result.best_move, result.score);
            //         // result.pv.iter().for_each(|mv| println!("PV move: {}", mv));
            //     }
            //     None => eprintln!("AI was not thinking or failed to return a result"),
            // }
            // state.ai_state = AiState::Idle;
        } else if state.ai.is_none() {
            if !state.args.human {
                std::process::exit(1);
            }

            eprintln!("No AI loaded. Use 'load_ai <ai_name>' to load an AI.");
        } else {
            if !state.args.human {
                std::process::exit(0);
            }

            eprintln!("AI is not currently thinking. Use 'go' to start the AI.");
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
        if state.args.human {
            println!("Next player to move: {}", color);
        } else {
            println!("{}", color);
        }
    }
}

pub struct UciCommand;
impl Command for UciCommand {
    fn name(&self) -> &str {
        "uci"
    }

    fn description(&self) -> &str {
        "Enter UCI mode"
    }

    fn execute(&self, state: &mut AppState, _args: &[String]) {
        let ai = match &mut state.ai {
            Some(ai) => ai,
            None => {
                eprintln!("No AI loaded. Use 'load_ai <ai_name>' to load an AI.");
                return;
            }
        };
        println!("id name {}", ai.name());
        println!("id author {}", ai.authors().join(", "));
        println!("");
        println!("uciok");
    }
}

pub struct UciNewGameCommand;
impl Command for UciNewGameCommand {
    fn name(&self) -> &str {
        "ucinewgame"
    }

    fn description(&self) -> &str {
        "Signal the start of a new game in UCI mode"
    }

    fn execute(&self, state: &mut AppState, _args: &[String]) {
        if let Some(ai) = &mut state.ai {
            ai.reset();
        }
    }
}

pub struct IsReadyCommand;
impl Command for IsReadyCommand {
    fn name(&self) -> &str {
        "isready"
    }

    fn description(&self) -> &str {
        "Check if the AI is ready in UCI mode"
    }

    fn execute(&self, state: &mut AppState, _args: &[String]) {
        if state.ai.is_some() {
            println!("readyok");
        } else {
            eprintln!("No AI loaded. Use 'load_ai <ai_name>' to load an AI.");
        }
    }
}
