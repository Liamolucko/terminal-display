//! Draw a line using `draw_iter` to test that implementation (as opposed to
//! `draw_contiguous`).

use std::io;

use embedded_graphics_core::prelude::*;
use terminal_display::{Color, TerminalDisplay};

fn main() -> io::Result<()> {
    let mut display = TerminalDisplay::new()?;

    let len = u32::min(display.size().width, display.size().height);

    display.clear(Color::BgColor)?;
    display.draw_iter((0..len as i32).map(|i| Pixel(Point::new(i, i), Color::FgColor)))?;

    display.flush()?;
    loop {}
}
