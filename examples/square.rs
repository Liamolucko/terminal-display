//! Draws a square which specifically doesn't align nicely to the cell
//! boundaries to make sure our edge cases work properly.

use std::io;

use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use terminal_display::{Color, TerminalDisplay};

fn main() -> io::Result<()> {
    let mut display = TerminalDisplay::new()?;

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Red)
        .stroke_width(1)
        .fill_color(Color::Green)
        .build();

    Rectangle {
        top_left: Point::new(1, 1),
        size: Size::new(6, 6),
    }
    .into_styled(style)
    .draw(&mut display)?;

    display.flush()?;

    loop {}
}
