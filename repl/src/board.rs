use chessoteric_core::bitboard::Bitboard;
use ratatui::prelude::*;
use ratatui::widgets::Widget;

use crate::skin::{DisplayMode, display_from_str};

pub struct BoardWidget<'a> {
    pub board: &'a chessoteric_core::board::SquareCentricBoard,
    pub highlighted: Bitboard,
    pub selected: Option<u8>,
    pub screen: Rect,
}

impl<'a> BoardWidget<'a> {
    pub const ASPECT_RATIO: u16 = 2; // The board should have an aspect ratio of 2:1, so it fits nicely in the left half of the screen
}

impl<'a> Widget for BoardWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // buf.set_style(area, Style::default().bg(Color::Red));

        // First of all build a rectangle with an aspect ratio of 2:1, centered in the area
        let size = area.height.min(area.width / BoardWidget::ASPECT_RATIO).max(8);

        // Determine the display mode based on the size of the area
        let display_mode = DisplayMode::from_size(size as usize / 8);
        let display_size = display_mode.size() as u16;

        // Find the closest multiple of 8 that is less than or equal to size, to ensure the squares are perfectly square
        let size = size - (size % 8);
        let x_offset = ((self.screen.width - size * BoardWidget::ASPECT_RATIO) / 2)
            .min(area.width - size * BoardWidget::ASPECT_RATIO);
        let y_offset = (area.height.saturating_sub(size)) / 2;
        let board_area = Rect {
            x: area.x + x_offset,
            y: area.y + y_offset,
            width: size * BoardWidget::ASPECT_RATIO,
            height: size,
        };

        // Then render the board squares
        for rank in 0..8 {
            for file in 0..8 {
                let is_light_square = (rank + file) % 2 == 0;
                let piece_index = ((7 - rank) * 8 + file) as usize;

                let mut square_color = if is_light_square {
                    Color::Rgb(122, 133, 147)
                } else {
                    Color::Rgb(64, 81, 181)
                };

                if self.selected == Some(piece_index as u8) {
                    square_color = Color::Rgb(186, 202, 68); // Selected square color for selection B
                } else if self.highlighted.0 & (1 << piece_index) != 0 {
                    if is_light_square {
                        square_color = Color::Rgb(136, 153, 186); // Light square color
                    } else {
                        square_color = Color::Rgb(73, 83, 145);
                    }
                }

                let is_white_piece = self.board.squares[piece_index]
                    .map(|piece| piece.color() == chessoteric_core::board::Color::White)
                    .unwrap_or(false);
                let fg_color = if is_white_piece {
                    Color::Rgb(255, 255, 255) // White piece color
                } else {
                    Color::Rgb(0, 0, 0) // Black piece color
                };

                let square_rect = Rect {
                    x: board_area.x + file * (board_area.width / 8),
                    y: board_area.y + rank * (board_area.height / 8),
                    width: board_area.width / 8,
                    height: board_area.height / 8,
                };
                buf.set_style(square_rect, Style::default().bg(square_color).fg(fg_color));

                if let Some(piece) = self.board.squares[rank as usize * 8 + file as usize] {
                    let piece_char = display_from_str(piece, display_mode);
                    let x = square_rect.x + (square_rect.width.saturating_sub(display_size * 2)) / 2;
                    let mut y = square_rect.y + (square_rect.height.saturating_sub(display_size)) / 2;

                    for line in piece_char.lines() {
                        buf.set_string(x, y, line, Style::default());
                        y += 1;
                    }
                }
            }
        }
    }
}
