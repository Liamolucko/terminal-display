use std::io::{self, BufWriter, Stdout, Write};
use std::ops::Range;

use crossterm::style::Color as CrosstermColor;
use crossterm::{cursor, style, terminal, QueueableCommand};
use embedded_graphics_core::prelude::*;
use embedded_graphics_core::primitives::Rectangle;

mod color;

pub use color::Color;

/// Get the size of the terminal in pixels from its size in rows/columns.
fn size(width: u16, height: u16) -> Size {
    Size::new(u32::from(width), 2 * u32::from(height))
}

/// Get the bounding box of the terminal in pixels from its size in
/// rows/columns.
fn bounding_box(width: u16, height: u16) -> Rectangle {
    Rectangle {
        top_left: Point::zero(),
        size: size(width, height),
    }
}

fn write_cell(mut stdout: impl Write, top_color: Color, bottom_color: Color) -> io::Result<()> {
    match (top_color, bottom_color) {
        (Color::BgColor, Color::BgColor) => {
            stdout.queue(style::SetBackgroundColor(CrosstermColor::Reset))?;
            stdout.write_all(" ".as_bytes())
        }
        (Color::FgColor, Color::FgColor) => {
            stdout.queue(style::SetForegroundColor(CrosstermColor::Reset))?;
            stdout.write_all("█".as_bytes())
        }
        (top_color, bottom_color)
            if top_color != Color::FgColor && bottom_color != Color::BgColor =>
        {
            stdout.queue(style::SetBackgroundColor(top_color.to_crossterm_color()))?;
            stdout.queue(style::SetForegroundColor(bottom_color.to_crossterm_color()))?;
            stdout.write_all("▄".as_bytes())
        }
        (top_color, bottom_color) => {
            stdout.queue(style::SetBackgroundColor(bottom_color.to_crossterm_color()))?;
            stdout.queue(style::SetForegroundColor(top_color.to_crossterm_color()))?;
            stdout.write_all("▀".as_bytes())
        }
    }
}

/// An implementation of `embedded_graphics::DrawTarget` for the terminal using
/// crossterm.
///
/// A pixel is half of a character in the terminal, since they're usually about
/// 1x2.
///
/// To show the rendered image, the buffer must be flushed by calling
/// [`TerminalDisplay::flush`].
///
/// [`TerminalDisplay::flush`]: crate::TerminalDisplay::flush
pub struct TerminalDisplay {
    /// A tuple of the (top_color, bottom_color) of every cell.
    ///
    /// This is needed because it's impossible to get back the color of a cell,
    /// and we need to preserve the color of the other half of the cell when
    /// writing a single pixel.
    buffer: Vec<Vec<(Color, Color)>>,
    /// We need to store this between runs so that
    stdout: BufWriter<Stdout>,
}

impl TerminalDisplay {
    pub fn new() -> io::Result<Self> {
        let mut this = Self {
            buffer: Vec::new(),
            stdout: BufWriter::new(io::stdout()),
        };
        this.resize()?;
        Ok(this)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }

    /// Resize the buffer to the correct size if it's changed, and return the
    /// current size of the terminal as (width, height).
    fn resize(&mut self) -> io::Result<(u16, u16)> {
        let (width, height) = terminal::size()?;
        if self.buffer.get(0).map_or(0, |row| row.len()) != width.into() {
            for row in &mut self.buffer {
                row.resize(width.into(), (Color::BgColor, Color::BgColor));
            }
        }
        if self.buffer.len() != height.into() {
            self.buffer.resize_with(height.into(), || {
                vec![(Color::BgColor, Color::BgColor); width.into()]
            })
        }

        Ok((width, height))
    }

    fn fill_solid_aligned(
        &mut self,
        columns: Range<u16>,
        rows: Range<u16>,
        color: Color,
    ) -> io::Result<()> {
        // Update the color buffer
        self.buffer[usize::from(rows.start)..usize::from(rows.end)]
            .iter_mut()
            .for_each(|row| {
                row[usize::from(columns.start)..usize::from(columns.end)].fill((color, color))
            });

        if color == Color::FgColor {
            self.stdout
                .queue(style::SetForegroundColor(CrosstermColor::Reset))?;
        } else {
            self.stdout
                .queue(style::SetBackgroundColor(color.to_crossterm_color()))?;
        }

        for row in rows {
            self.stdout.queue(cursor::MoveTo(columns.start, row))?;

            for _ in columns.clone() {
                if color == Color::FgColor {
                    self.stdout.write_all("█".as_bytes())?;
                } else {
                    self.stdout.write_all(" ".as_bytes())?;
                }
            }
        }

        Ok(())
    }
}

impl OriginDimensions for TerminalDisplay {
    fn size(&self) -> Size {
        let (width, height) = terminal::size().expect("failed to get terminal size");
        size(width, height)
    }
}

impl DrawTarget for TerminalDisplay {
    type Color = Color;

    type Error = io::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> io::Result<()>
    where
        I: IntoIterator<Item = Pixel<Color>>,
    {
        let (width, height) = self.resize()?;
        let bounding_box = bounding_box(width, height);

        for Pixel(point, color) in pixels {
            if bounding_box.contains(point) {
                // We've just checked that these coordinates fall within the bounds of the
                // terminal, so they must fit within a u16.
                let column = point.x as u16;
                let row = (point.y / 2) as u16;
                self.stdout.queue(cursor::MoveTo(column, row))?;

                let (top_color, bottom_color) =
                    &mut self.buffer[usize::from(row)][usize::from(column)];
                if point.y % 2 == 0 {
                    *top_color = color;
                } else {
                    *bottom_color = color;
                }
                write_cell(&mut self.stdout, *top_color, *bottom_color)?;
            }
        }
        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> io::Result<()>
    where
        I: IntoIterator<Item = Color>,
    {
        let (width, height) = self.resize()?;
        let bounding_box = bounding_box(width, height);

        // Clamp the passed area to the size of the terminal.
        let clamped_area = bounding_box.intersection(area);

        // Compute all of the dimensions we need.
        let (left_padding, right_padding, top_padding, start_y, end_y) =
            match (area.bottom_right(), clamped_area.bottom_right()) {
                (Some(bottom_right), Some(clamped_bottom_right)) => {
                    // The clamped area will only ever be the same size or smaller than the original
                    // area, which means the results of these subtractions will
                    // always be positive; so, if they don't fit in a `usize`,
                    // it must be because they're too big, so saturate them to `usize::MAX`.
                    let left_padding = usize::try_from(clamped_area.top_left.x - area.top_left.x)
                        .unwrap_or(usize::MAX);
                    let right_padding = usize::try_from(bottom_right.x - clamped_bottom_right.x)
                        .unwrap_or(usize::MAX);
                    let top_padding = usize::try_from(clamped_area.top_left.y - area.top_left.y)
                        .unwrap_or(usize::MAX);

                    let start_y = clamped_area.top_left.y;
                    let end_y = clamped_bottom_right.y;

                    (left_padding, right_padding, top_padding, start_y, end_y)
                }
                // If either of those boxes is zero-sized (which causes `bottom_right` to return
                // `None`), we've got nothing to draw.
                _ => return Ok(()),
            };

        let mut colors = colors
            .into_iter()
            .skip(area.size.width.try_into().unwrap_or(usize::MAX) * top_padding);

        // TODO: replace with `Iterator::advance_by` once it's stabilised.
        fn advance_by<T>(iterator: &mut impl Iterator<Item = T>, n: usize) {
            if let Some(i) = n.checked_sub(1) {
                let _ = iterator.nth(i);
            }
        }

        for y in clamped_area.rows() {
            let is_top_half = y % 2 == 0;

            // Move the cursor to the start of the row.
            // We know these will fit in `u16`s because they have to be within
            // our bounding box of the terminal.
            let column = clamped_area.top_left.x as u16;
            let row = (y / 2) as u16;
            self.stdout.queue(cursor::MoveTo(column, row))?;

            // Skip the out-of-bounds part at the start of this row.
            advance_by(&mut colors, left_padding);

            for x in clamped_area.columns() {
                let column = x as u16;

                let color = colors.next();

                let (top_color, bottom_color) =
                    &mut self.buffer[usize::from(row)][usize::from(column)];

                if let Some(color) = color {
                    if is_top_half {
                        *top_color = color;
                    } else {
                        *bottom_color = color;
                    }
                } else if is_top_half || y == start_y {
                    // Return early, as long as we don't still need to draw for the sake of the top
                    // half.
                    return Ok(());
                }

                // Wait until the bottom half of the cell to write it, unless this is the last
                // row and there won't be a bottom half. Our main bottleneck is actually writing
                // to the tty, so the less we write the better.
                if !is_top_half || y == end_y {
                    write_cell(&mut self.stdout, *top_color, *bottom_color)?;
                }
            }

            // Now skip the out-of-bounds part at the end of this row.
            advance_by(&mut colors, right_padding);
        }
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Color) -> io::Result<()> {
        let (width, height) = self.resize()?;
        let bounding_box = bounding_box(width, height);

        // Clamp the passed area to the size of the terminal.
        let clamped_area = bounding_box.intersection(area);

        let top_left = clamped_area.top_left;
        let bottom_right = match clamped_area.bottom_right() {
            Some(bottom_right) => bottom_right,
            // If the box is zero sized, we don't need to draw anything.
            _ => return Ok(()),
        };

        let start_column = top_left.x as u16;
        let end_column = start_column + clamped_area.size.width as u16;

        if top_left.y % 2 == 1 {
            // We need to draw the first row normally, since we still need to change the
            // color of the top half as we go.
            let row = (top_left.y / 2) as u16;
            self.stdout.queue(cursor::MoveTo(start_column, row))?;

            for (top_color, bottom_color) in &mut self.buffer[usize::from(row)]
                [usize::from(start_column)..usize::from(end_column)]
            {
                *bottom_color = color;
                write_cell(&mut self.stdout, *top_color, *bottom_color)?;
            }
        }

        // Figure out the start and end row of the solidly-filled part.
        let mut start_row = (top_left.y / 2) as u16;
        if top_left.y % 2 == 1 {
            // If we start in the second half of a row, that row gets filled normally; so start the solidly-filled part one row later.
            start_row += 1;
        }

        // We're building an exclusive range, so the end point is one after the last row.
        let mut end_row = (bottom_right.y / 2) as u16 + 1;
        if bottom_right.y % 2 == 0 {
            // If we're ending on the top half of a row, that row gets filled normally; so end the solidly-filled part one row earlier.
            end_row -= 1;
        }

        self.fill_solid_aligned(start_column..end_column, start_row..end_row, color)?;

        if bottom_right.y % 2 == 0 {
            // We need to draw the last row normally, since we still need to
            // change the color of the bottom half as we go.
            let row = (bottom_right.y / 2) as u16;
            self.stdout.queue(cursor::MoveTo(start_column, row))?;
            for (top_color, bottom_color) in &mut self.buffer[usize::from(row)]
                [usize::from(start_column)..usize::from(end_column)]
            {
                *top_color = color;
                write_cell(&mut self.stdout, *top_color, *bottom_color)?;
            }
        }

        Ok(())
    }

    fn clear(&mut self, color: Color) -> io::Result<()> {
        let (width, height) = self.resize()?;
        self.fill_solid_aligned(0..width, 0..height, color)
    }
}
