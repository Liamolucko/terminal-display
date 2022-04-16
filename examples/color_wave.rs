use std::io;
use std::time::Instant;

use crossterm::{ExecutableCommand, cursor};
use embedded_graphics_core::pixelcolor::Rgb888;
use embedded_graphics_core::prelude::{Dimensions, DrawTarget, PointsIter};
use palette::{Hsv, IntoColor, IntoComponent, Srgb};
use terminal_display::TerminalDisplay;

fn main() -> io::Result<()> {
    let mut display = TerminalDisplay::new()?;

    io::stdout().execute(cursor::Hide)?;

    let start = Instant::now();
    loop {
        let bounding_box = display.bounding_box();
        let elapsed = start.elapsed();
        display.fill_contiguous(
            &bounding_box,
            bounding_box.points().map(|point| {
                let hue = point.x as f64 - point.y as f64 + elapsed.as_secs_f64() * 200.0;
                let color: Srgb<f64> = Hsv::new(hue, 1.0, 1.0).into_color();
                Rgb888::new(
                    color.red.into_component(),
                    color.green.into_component(),
                    color.blue.into_component(),
                )
            }),
        )?;
    }
}
