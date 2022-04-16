use crossterm::style::Color as CrosstermColor;
use embedded_graphics_core::pixelcolor::{
    Bgr555, Bgr565, Bgr888, BinaryColor, Gray2, Gray4, Gray8, Rgb555, Rgb565, Rgb888,
};
use embedded_graphics_core::prelude::*;

/// A color which can be rendered to a terminal.
///
/// Basically a clone of [`crossterm::style::Color`], which can't be used
/// directly because of the orphan rule.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    /// The default background color of the terminal.
    BgColor,

    /// The default foreground color of the terminal.
    FgColor,

    /// Black color.
    Black,

    /// Dark grey color.
    DarkGrey,

    /// Light red color.
    Red,

    /// Dark red color.
    DarkRed,

    /// Light green color.
    Green,

    /// Dark green color.
    DarkGreen,

    /// Light yellow color.
    Yellow,

    /// Dark yellow color.
    DarkYellow,

    /// Light blue color.
    Blue,

    /// Dark blue color.
    DarkBlue,

    /// Light magenta color.
    Magenta,

    /// Dark magenta color.
    DarkMagenta,

    /// Light cyan color.
    Cyan,

    /// Dark cyan color.
    DarkCyan,

    /// White color.
    White,

    /// Grey color.
    Grey,

    /// An RGB color. See [RGB color model](https://en.wikipedia.org/wiki/RGB_color_model) for more info.
    ///
    /// Most UNIX terminals and Windows 10 supported only.
    /// See [Platform-specific notes](enum.Color.html#platform-specific-notes)
    /// for more info.
    Rgb(Rgb888),

    /// An ANSI color. See [256 colors - cheat sheet](https://jonasjacek.github.io/colors/) for more info.
    ///
    /// Most UNIX terminals and Windows 10 supported only.
    /// See [Platform-specific notes](enum.Color.html#platform-specific-notes)
    /// for more info.
    AnsiValue(u8),
}

impl Color {
    pub(crate) fn to_crossterm_color(self) -> CrosstermColor {
        match self {
            Color::BgColor | Color::FgColor => CrosstermColor::Reset,
            Color::Black => CrosstermColor::Black,
            Color::DarkGrey => CrosstermColor::DarkGrey,
            Color::Red => CrosstermColor::Red,
            Color::DarkRed => CrosstermColor::DarkRed,
            Color::Green => CrosstermColor::Green,
            Color::DarkGreen => CrosstermColor::DarkGreen,
            Color::Yellow => CrosstermColor::Yellow,
            Color::DarkYellow => CrosstermColor::DarkYellow,
            Color::Blue => CrosstermColor::Blue,
            Color::DarkBlue => CrosstermColor::DarkBlue,
            Color::Magenta => CrosstermColor::Magenta,
            Color::DarkMagenta => CrosstermColor::DarkMagenta,
            Color::Cyan => CrosstermColor::Cyan,
            Color::DarkCyan => CrosstermColor::DarkCyan,
            Color::White => CrosstermColor::White,
            Color::Grey => CrosstermColor::Grey,
            Color::Rgb(color) => CrosstermColor::Rgb {
                r: color.r(),
                g: color.g(),
                b: color.b(),
            },
            Color::AnsiValue(n) => CrosstermColor::AnsiValue(n),
        }
    }
}

impl PixelColor for Color {
    type Raw = ();
}

impl Default for Color {
    fn default() -> Self {
        Color::BgColor
    }
}

impl From<BinaryColor> for Color {
    fn from(color: BinaryColor) -> Self {
        match color {
            BinaryColor::Off => Self::BgColor,
            BinaryColor::On => Self::FgColor,
        }
    }
}

impl From<Rgb888> for Color {
    fn from(color: Rgb888) -> Self {
        Self::Rgb(color)
    }
}

impl From<Bgr555> for Color {
    fn from(color: Bgr555) -> Self {
        Self::Rgb(color.into())
    }
}

impl From<Bgr565> for Color {
    fn from(color: Bgr565) -> Self {
        Self::Rgb(color.into())
    }
}

impl From<Bgr888> for Color {
    fn from(color: Bgr888) -> Self {
        Self::Rgb(color.into())
    }
}

impl From<Gray2> for Color {
    fn from(color: Gray2) -> Self {
        Self::Rgb(color.into())
    }
}

impl From<Gray4> for Color {
    fn from(color: Gray4) -> Self {
        Self::Rgb(color.into())
    }
}

impl From<Gray8> for Color {
    fn from(color: Gray8) -> Self {
        Self::Rgb(color.into())
    }
}

impl From<Rgb555> for Color {
    fn from(color: Rgb555) -> Self {
        Self::Rgb(color.into())
    }
}

impl From<Rgb565> for Color {
    fn from(color: Rgb565) -> Self {
        Self::Rgb(color.into())
    }
}
