use std::io::{self, BufWriter, Write};

use crossterm::style::Color;
use crossterm::{cursor, style, terminal, QueueableCommand};
use embedded_graphics_core::pixelcolor::Rgb888;
use embedded_graphics_core::prelude::*;
use embedded_graphics_core::primitives::Rectangle;

/// An implementation of `embedded_graphics::DrawTarget` for the terminal using
/// crossterm.
///
/// A pixel is half of a character in the terminal, since they're usually about
/// 1x2.
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

    /// Resize the buffer to the correct size if it's changed, and return the
    /// current bounding box (in pixels, not rows/columns).
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

    fn write_pixel(
        &mut self,
        mut stdout: impl Write,
        row: u16,
        column: u16,
        foreground_color: Option<Rgb888>,
        background_color: Option<Rgb888>,
    ) -> io::Result<()> {
        let (foreground, background) = &mut self.buffer[usize::from(row)][usize::from(column)];
        if let Some(color) = foreground_color {
            *foreground = color;
        }
        if let Some(color) = background_color {
            *background = color;
        }

        stdout.queue(style::SetForegroundColor(Color::Rgb {
            r: foreground.r(),
            g: foreground.g(),
            b: foreground.b(),
        }))?;
        stdout.queue(style::SetBackgroundColor(Color::Rgb {
            r: background.r(),
            g: background.g(),
            b: background.b(),
        }))?;

        stdout.write_all("▄".as_bytes())
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
        let mut stdout = BufWriter::new(stdout.lock());
        for Pixel(point, color) in pixels {
            if bounding_box.contains(point) {
                // We've just checked that these coordinates fall within the bounds of the
                // terminal, so they must fit within a u16.
                let column = point.x as u16;
                let row = (point.y / 2) as u16;
                stdout.queue(cursor::MoveTo(column, row))?;

                let mut foreground_color = None;
                let mut background_color = None;
                if point.y % 2 == 0 {
                    // Since we're using '▄' as our character, the color of the top half of the
                    // pixel is the background color.
                    background_color = Some(color);
                } else {
                    foreground_color = Some(color);
                }
                self.write_pixel(&mut stdout, row, column, foreground_color, background_color)?;
            }
        }
        stdout.flush()?;
        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let bounding_box = self.resize()?;
        let drawn_box = bounding_box.intersection(area);

        if drawn_box.is_zero_sized() {
            return Ok(());
        }

        let mut colors = colors.into_iter().skip(
            area.size.width.try_into().unwrap_or(usize::MAX)
                // `drawn_box.top_left.y` will always be >= `area.top_left.y`,
                // so if this result doesn't fit in a usize, it must be because
                // it's too big; in that case, round down to `usize::MAX`.
                * usize::try_from(drawn_box.top_left.y - area.top_left.y).unwrap_or(usize::MAX),
        );

        // TODO: replace with `Iterator::advance_by` once it's stabilised.
        fn advance_by<T>(iterator: &mut impl Iterator<Item = T>, n: usize) {
            if n != 0 {
                let _ = iterator.nth(n - 1);
            }
        }

        let stdout = io::stdout();
        let mut stdout = BufWriter::new(stdout.lock());
        for row in drawn_box.rows() {
            let is_top_half = row % 2 == 0;

            // Move the cursor to the start of the row.
            // We know these will fit in `u16`s because they have to be within
            // our bounding box of the terminal.
            let column = drawn_box.top_left.x as u16;
            let row = (row / 2) as u16;
            stdout.queue(cursor::MoveTo(column, row))?;

            // Skip the out-of-bounds part at the start of this row.
            advance_by(
                &mut colors,
                usize::try_from(drawn_box.top_left.x - area.top_left.x).unwrap_or(usize::MAX),
            );

            for column in drawn_box.columns() {
                let column = column as u16;

                let color = colors.next();
                if color.is_none()
                    && (is_top_half
                        || i32::from(row) == drawn_box.top_left.y
                        || Some(row.into()) == drawn_box.bottom_right().map(|point| point.y))
                {
                    // Return early, as long as we don't still need to draw for the sake of the top
                    // half.
                    stdout.flush()?;
                    return Ok(());
                }

                // Wait until the bottom half of the pixel to write it (unless this is the last
                // row). Our main bottleneck is actually writing to the tty, so the less we
                // write the better.
                if !is_top_half || Some(row.into()) == drawn_box.bottom_right().map(|point| point.y)
                {
                    let mut foreground_color = None;
                    let mut background_color = None;
                    if is_top_half {
                        // Since we're using '▄' as our character, the color of the top half of the
                        // pixel is the background color.
                        background_color = color;
                    } else {
                        foreground_color = color;
                    }

                    self.write_pixel(&mut stdout, row, column, foreground_color, background_color)?;
                } else {
                    let (_, background) = &mut self.buffer[usize::from(row)][usize::from(column)];
                    // This has to be the top half of the pixel, otherwise we would have taken the
                    // other branch.
                    // Set this so that it will be read when writing the other half of this cell.
                    *background = color.expect(
                        "if color is none, we should have already returned early by this point",
                    );
                }
            }

            // Now skip the out-of-bounds part at the end of this row.
            advance_by(
                &mut colors,
                // `bottom_right` will never return `None` here, because we return early if
                // `drawn_box` is zero-sized, and if `drawn_box` isn't zero-sized `area` can't
                // be zero-sized either.
                (area.bottom_right().unwrap().x - drawn_box.bottom_right().unwrap().x)
                    .try_into()
                    // `area.bottom_right().x` will always be >= `drawn_box.bottom_right().x`,
                    // so if this result doesn't fit in a usize, it must be because it's too
                    // big; in that case, round down to `usize::MAX`.
                    .unwrap_or(usize::MAX),
            )
        }
        stdout.flush()?;
        Ok(())
    }
}
