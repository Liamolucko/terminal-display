use std::io::{self, Write};

use crossterm::style::{Color, Colors};
use crossterm::{cursor, style, terminal, QueueableCommand};
use embedded_graphics_core::pixelcolor::Rgb888;
use embedded_graphics_core::prelude::*;
use embedded_graphics_core::primitives::Rectangle;

/// An implementation of `embedded_graphics::DrawTarget` for the terminal using crossterm.
///
/// A pixel is half of a character in the terminal, since they're usually about 1x2.
pub struct TerminalDisplay {
    /// A tuple of the (foreground_color, background_color) of every cell.
    ///
    /// This is needed because it's impossible to get back the color of a cell,
    /// and we need to preserve the color of the other half of the cell when
    /// writing a single pixel.
    buffer: Vec<Vec<(Rgb888, Rgb888)>>,
}

impl TerminalDisplay {
    pub fn new() -> io::Result<Self> {
        let mut this = Self { buffer: Vec::new() };
        this.resize()?;
        Ok(this)
    }

    /// Resizes the buffer to the correct size if it's changed,
    /// and return the current bounding box (in pixels, not rows/columns).
    fn resize(&mut self) -> io::Result<Rectangle> {
        let (width, height) = terminal::size()?;
        if self.buffer.get(0).map_or(0, |row| row.len()) != width.into() {
            for row in &mut self.buffer {
                row.resize(width.into(), (Rgb888::BLACK, Rgb888::BLACK));
            }
        }
        if self.buffer.len() != height.into() {
            self.buffer.resize_with(height.into(), || {
                vec![(Rgb888::BLACK, Rgb888::BLACK); width.into()]
            })
        }

        Ok(Rectangle {
            top_left: Point::zero(),
            size: Size::new(u32::from(width), 2 * u32::from(height)),
        })
    }
}

impl OriginDimensions for TerminalDisplay {
    fn size(&self) -> Size {
        let (width, height) = terminal::size().expect("failed to get terminal size");
        Size::new(u32::from(width), 2 * u32::from(height))
    }
}

impl DrawTarget for TerminalDisplay {
    type Color = Rgb888;

    type Error = io::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bounding_box = self.resize()?;

        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        for Pixel(point, color) in pixels {
            if bounding_box.contains(point) {
                // We've just checked that these coordinates fall within the bounds of the terminal,
                // so they must fit within a u16.
                let column = point.x as u16;
                let row = (point.y / 2) as u16;
                stdout.queue(cursor::MoveTo(column, row))?;

                let (foreground, background) =
                    &mut self.buffer[usize::from(row)][usize::from(column)];
                if point.y % 2 == 0 {
                    // Since we're using '▄' as our character, the color of the top half of the pixel
                    // is the background color.
                    *background = color;
                } else {
                    *foreground = color;
                }
                stdout.queue(style::SetColors(Colors {
                    foreground: Some(Color::Rgb {
                        r: foreground.r(),
                        g: foreground.g(),
                        b: foreground.b(),
                    }),
                    background: Some(Color::Rgb {
                        r: background.r(),
                        g: background.g(),
                        b: background.b(),
                    }),
                }))?;

                stdout.write_all("▄".as_bytes())?;
            }
        }
        Ok(())
    }
}
