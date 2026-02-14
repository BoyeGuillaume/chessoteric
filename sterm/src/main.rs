use std::io::{self, Write};

use clap::Parser;
pub mod state;

#[derive(Parser, Clone)]
#[command(name = "sterm", about = "A simple chess terminal application")]
pub struct StermArgs {
    /// FEN string representing the chess position
    #[clap(
        short,
        long,
        default_value = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    )]
    pub fen: String,

    /// A flag to disable use output (useful when using application as a subprocess)
    #[clap(short, long)]
    pub no_output: bool,
}

fn main() {
    let args = StermArgs::parse();

    // Load the chess position from the FEN string
    let mut state = state::AppState {
        board: chessoteric_core::board::Board::from_fen(&args.fen).expect("Invalid FEN string"),
        ai: None,
        args,
    };
    let commands = state::all_commands();

    'mainloop: loop {
        // Read user input for a move
        if !state.args.no_output {
            print!("[chess] $ ");
            io::stdout().flush().expect("Failed to flush stdout");
        }
        let mut input = String::new();
        if let Err(e) = std::io::stdin().read_line(&mut input) {
            eprintln!("Error reading input: {}", e);
            std::process::exit(1);
        };

        // Process the input move
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Parse the input to arguments similarly to how we parse arguments for bash
        let args = match shell_words::split(input) {
            Ok(args) => args,
            Err(e) => {
                eprintln!("Error parsing input: {}", e);
                std::process::exit(1);
            }
        };

        // Handle the input move
        for command in &commands {
            if command.name() == args[0] {
                command.execute(&mut state, &args);
                continue 'mainloop;
            }
        }

        // Help command
        if args[0] == "help" {
            println!("Available commands:");
            for command in &commands {
                println!(" - {}: {}", command.name(), command.description());
            }
            println!(" - help: Show this help message");
            continue;
        }

        // If we reach this point, the command is unknown
        if !state.args.no_output {
            eprintln!(
                "Unknown command: {}, type 'help' for a list of commands",
                args[0]
            );
        } else {
            std::process::exit(1);
        }
    }
}
