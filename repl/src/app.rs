use crate::board::BoardWidget;
use chessoteric_core::{bitboard::Bitboard, moves::generate_moves};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Offset},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Padding},
};
use std::{env::args, time::Duration};

const TITLE_STR: &str = r"   █████████  █████                                       █████                        ███          
  ███░░░░░███░░███                                       ░░███                        ░░░           
 ███     ░░░  ░███████    ██████  █████  █████   ██████  ███████    ██████  ████████  ████   ██████ 
░███          ░███░░███  ███░░██████░░  ███░░   ███░░███░░░███░    ███░░███░░███░░███░░███  ███░░███
░███          ░███ ░███ ░███████░░█████░░█████ ░███ ░███  ░███    ░███████  ░███ ░░░  ░███ ░███ ░░░ 
░░███     ███ ░███ ░███ ░███░░░  ░░░░███░░░░███░███ ░███  ░███ ███░███░░░   ░███      ░███ ░███  ███
 ░░█████████  ████ █████░░██████ ██████ ██████ ░░██████   ░░█████ ░░██████  █████     █████░░██████ 
  ░░░░░░░░░  ░░░░ ░░░░░  ░░░░░░ ░░░░░░ ░░░░░░   ░░░░░░     ░░░░░   ░░░░░░  ░░░░░     ░░░░░  ░░░░░░  
";

struct AppState {
    board: chessoteric_core::board::SquareCentricBoard,
    moves: Vec<String>,
    highlighted_moves: Bitboard,
    buffer: String,
    cursor_position: u8,
    selected_position: Option<u8>,
    current_moves: Vec<chessoteric_core::moves::Move>,
    current_score: f32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            board: chessoteric_core::board::SquareCentricBoard::default_position(),
            moves: Vec::new(),
            highlighted_moves: Bitboard(0x0),
            buffer: String::new(),
            cursor_position: 0,
            current_moves: Vec::new(),
            selected_position: None,
            current_score: 0.0,
        }
    }
}

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<String> {
    // let mut ai = chessoteric_core::ai::get_ai("simple").unwrap();
    let mut state = AppState::default();

    if args().len() > 1 {
        state.board =
            chessoteric_core::board::SquareCentricBoard::parse_fen(&args().nth(1).unwrap())
                .unwrap();
    }

    let board = state.board.clone().into();
    let mut in_check = false;
    generate_moves(&board, &mut state.current_moves, &mut in_check);

    loop {
        terminal.draw(|frame| render(frame, &mut state))?;

        if crossterm::event::poll(Duration::from_millis(20))? {
            let event = crossterm::event::read()?;

            match event {
                crossterm::event::Event::Key(key_event)
                    if key_event.code == crossterm::event::KeyCode::Char('c')
                        && key_event
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    break Ok(format!("You entered: {}", state.board.fen()));
                }
                crossterm::event::Event::Key(key_event) => {
                    if key_event.is_press() || key_event.is_repeat() {
                        match key_event.code {
                            crossterm::event::KeyCode::Left => {
                                if state.cursor_position > 0 {
                                    state.cursor_position -= 1;
                                }
                            }
                            crossterm::event::KeyCode::Right => {
                                if state.cursor_position < 63 {
                                    state.cursor_position += 1;
                                }
                            }
                            crossterm::event::KeyCode::Down => {
                                if state.cursor_position >= 8 {
                                    state.cursor_position -= 8;
                                }
                            }
                            crossterm::event::KeyCode::Up => {
                                if state.cursor_position < 56 {
                                    state.cursor_position += 8;
                                }
                            }
                            crossterm::event::KeyCode::Esc => {
                                state.selected_position = None;
                                state.highlighted_moves = Bitboard::empty();
                            }
                            crossterm::event::KeyCode::Enter
                            | crossterm::event::KeyCode::Char(' ') => {
                                if let Some(selected_position) = state.selected_position
                                    && (state.highlighted_moves.0 & (1 << state.cursor_position))
                                        != 0
                                {
                                    state.highlighted_moves = Bitboard::empty();

                                    // Find the move that has the current cursor position as the destination square
                                    if let Some(mv) = state.current_moves.iter().find(|mv| {
                                        mv.to == state.cursor_position
                                            && mv.from == selected_position
                                    }) {
                                        state.moves.push(mv.to_string());

                                        let mut board = state.board.clone().into();
                                        mv.apply(&mut board);

                                        // Get the best move from the AI and apply it to the board
                                        // if let Some((ai_move, ai_score)) =
                                        //     ai.best_move(&board, std::time::Duration::from_secs(1))
                                        // {
                                        //     state.moves.push(ai_move.to_string());
                                        //     ai_move.apply(&mut board);
                                        //     state.current_score = ai_score;
                                        // }

                                        // Finally, get the best move from the AI and apply it to the board
                                        let new_board =
                                            chessoteric_core::board::SquareCentricBoard::from(
                                                board,
                                            );
                                        state.board = new_board;

                                        // Regenerate moves for the new board state
                                        state.current_moves.clear();
                                        let mut in_check = false;
                                        generate_moves(
                                            &board,
                                            &mut state.current_moves,
                                            &mut in_check,
                                        );
                                    }

                                    state.selected_position = None;
                                } else {
                                    state.selected_position = Some(state.cursor_position);

                                    // Filter all moves that have start position equal to the cursor position
                                    let mut bitboard = Bitboard::empty();
                                    for mv in state.current_moves.iter().filter_map(|mv| {
                                        if mv.from == state.cursor_position {
                                            Some(mv.to)
                                        } else {
                                            None
                                        }
                                    }) {
                                        bitboard.0 |= 1 << mv;
                                    }
                                    state.highlighted_moves = bitboard;
                                }
                            }
                            crossterm::event::KeyCode::Char(c) => state.buffer.push(c),
                            crossterm::event::KeyCode::Backspace => {
                                state.buffer.pop();
                            }
                            crossterm::event::KeyCode::Delete => {
                                state.buffer.clear();
                            }
                            _ => {}
                        }
                    }
                }
                crossterm::event::Event::Paste(_) => todo!(),
                _ => {}
            }
        }
    }
}

fn render(frame: &mut Frame, state: &mut AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Fill(1),
            Constraint::Length(5),
        ])
        .split(frame.area());

    // Display the title at the top of the layout
    let centered = layout[0].centered_horizontally(Constraint::Length(100));
    for (i, line) in TITLE_STR.lines().enumerate() {
        frame.render_widget(
            Span::styled(line, Style::default().fg(Color::DarkGray)),
            centered.offset(Offset::new(0, 1 + i as i32)),
        );
    }

    // Render the board in the top 80% of the layout
    let middle_area = layout[1];
    let middle_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(27)])
        .split(middle_area);

    frame.render_widget(
        BoardWidget {
            board: &state.board,
            selected: Some(state.cursor_position),
            screen: frame.area(),
            highlighted: state.highlighted_moves,
        },
        middle_layout[0],
    );

    // History of moves will go in the right half of the middle area, but we can leave it blank for now
    let history_block = Block::default()
        .title("History")
        .padding(Padding::symmetric(1, 0))
        .borders(ratatui::widgets::Borders::ALL);
    let history_block_area = history_block.inner(middle_layout[1]);
    frame.render_widget(history_block, middle_layout[1]);

    for move_counter in 0..(state.moves.len() + 1) / 2 {
        // Render the move number in the left half of the history block
        let string = format!("{}. ", move_counter + 1);
        frame.render_widget(
            Span::styled(string, Style::default().fg(Color::DarkGray)),
            history_block_area.offset(Offset::new(0, move_counter as i32)),
        );

        // Render the white move in the right half of the history block
        if let Some(mv) = state.moves.get(move_counter * 2) {
            frame.render_widget(
                Span::raw(mv.to_string()),
                history_block_area.offset(Offset::new(4, move_counter as i32)),
            );
        }

        // Render the black move in the right half of the history block
        if let Some(mv) = state.moves.get(move_counter * 2 + 1) {
            frame.render_widget(
                Span::raw(mv.to_string()),
                history_block_area.offset(Offset::new(14, move_counter as i32)),
            );
        }
    }

    // Render the commands in the bottom 20% of the layout
    fn title_block(title: &str) -> Block<'_> {
        let title = ratatui::text::Line::from(title).centered();
        Block::new()
            .borders(Borders::NONE)
            .padding(Padding::vertical(1))
            .title(title)
    }
    let ratio = ((state.current_score as f64 + 5.0) / 10.0).clamp(0.0, 1.0);

    frame.render_widget(
        Gauge::default()
            .block(title_block("title"))
            .ratio(ratio)
            .label(format!("{:.1}", state.current_score)),
        layout[2],
    );
}
