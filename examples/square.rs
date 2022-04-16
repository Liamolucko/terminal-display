use std::io;

use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Rectangle, PrimitiveStyleBuilder};
use terminal_display::TerminalDisplay;

fn main() -> io::Result<()> {
    let mut display = TerminalDisplay::new()?;

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb888::RED)
        .stroke_width(1)
        .fill_color(Rgb888::GREEN)
        .build();

    // Draw a square which specifically doesn't align nicely to the cell boundaries
    // to make sure our edge cases work properly.
    Rectangle {
        top_left: Point::new(1, 1),
        size: Size::new(6, 6),
    }
    .into_styled(style)
    .draw(&mut display)?;

    display.flush()?;

    loop {}
}
