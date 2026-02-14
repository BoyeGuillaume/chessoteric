use std::time::Duration;
use ratatui::{
    DefaultTerminal, Frame, layout::{Constraint, Direction, Layout, Offset}, style::{Color, Style}, text::Span, widgets::{Block, Padding}
};
use crate::board::BoardWidget;


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
    buffer: String,
    cursor_position: u8,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            board: chessoteric_core::board::SquareCentricBoard::default_position(),
            moves: Vec::new(),
            buffer: String::new(),
            cursor_position: 0,
        }
    }
}

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut state = AppState::default();
    state.moves.push("e4".to_string());
    state.moves.push("e5".to_string());
    state.moves.push("Nf3".to_string());

    loop {
        terminal.draw(|frame| render(frame, &mut state))?;

        if crossterm::event::poll(Duration::from_millis(20))? {
            let event = crossterm::event::read()?;

            match event {
                crossterm::event::Event::Key(key_event) => {
                    if key_event.code == crossterm::event::KeyCode::Char('c')
                        && key_event
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                    {
                        break Ok(());
                    }

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
                            crossterm::event::KeyCode::Enter => {}
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

    frame.render_widget(BoardWidget {
        board: &state.board,
        selected: Some(state.cursor_position),
        screen: frame.area(),
        highlighted: chessoteric_core::bitboard::Bitboard(0x0),
    }, middle_layout[0]);

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
    let input_area = layout[2];
    let input_block = Block::default()
        .title("Input")
        .padding(Padding::symmetric(3, 0))
        .borders(ratatui::widgets::Borders::ALL);
    let input_block_area = input_block.inner(input_area);

    frame.render_widget(input_block, input_area);
    frame.render_widget(
        ratatui::widgets::Paragraph::new(state.buffer.as_str()),
        input_block_area,
    );
    frame.set_cursor_position((
        input_block_area.x + state.buffer.len() as u16,
        input_block_area.y,
    ));
}
