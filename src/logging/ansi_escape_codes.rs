#![allow(unused, reason = "copypasta")]

use std::ptr;
use std::fmt;
use std::fmt::{Display, Formatter, Write as _};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectGraphicRendition {
    Reset,
    IntensityBold,
    IntensityDim,
    ItalicOn,
    UnderlineOn,
    BlinkSlow,
    BlinkFast,
    InverseOn,
    HiddenOn,
    StrikethroughOn,
    FontPrimary,
    FontAlternative1,
    FontAlternative2,
    FontAlternative3,
    FontAlternative4,
    FontAlternative5,
    FontAlternative6,
    FontAlternative7,
    FontAlternative8,
    FontAlternative9,
    FontFraktur,
    DoublyUnderlined,//sometimes BoldOff
    IntensityOff,
    ItalicOff,
    UnderlineOff,
    BlinkOff,
    ProportionalSpacingOn,
    InverseOff,
    HiddenOff,
    StrikethroughOff,
    ForegroundBlack,
    ForegroundRed,
    ForegroundGreen,
    ForegroundYellow,
    ForegroundBlue,
    ForegroundMagenta,
    ForegroundCyan,
    ForegroundWhite,
    ForegroundColor(Color),
    ForegroundDefault,
    BackgroundBlack,
    BackgroundRed,
    BackgroundGreen,
    BackgroundYellow,
    BackgroundBlue,
    BackgroundMagenta,
    BackgroundCyan,
    BackgroundWhite,
    BackgroundColor(Color),
    BackgroundDefault,
    ProportionalSpacingOff,
    BorderFrame,
    BorderEncircle,
    OverlinedOn,
    BorderNone,
    OverlinedOff,
    //56
    //57
    UnderlineColor(Color) = 58,// 58;5;n / 58;2;r;g;b
    UnderlineColorDefault,
    IdeogramUnderlineOrRightSide,
    IdeogramUnderlineOrRightSideDouble,
    IdeogramOverlineOrLeftSide,
    IdeogramOverlineOrLeftSideDouble,
    IdeogramStressMarking,
    IdeogramOff,
    //66
    //67
    //68
    //69 - nice
    //70
    //71
    //72
    Superscript = 73,
    Subscript,
    SuperSubscriptOff,
    //76..=89
    ForegroundBrightBlack = 90,
    ForegroundBrightRed,
    ForegroundBrightGreen,
    ForegroundBrightYellow,
    ForegroundBrightBlue,
    ForegroundBrightMagenta,
    ForegroundBrightCyan,
    ForegroundBrightWhite,
    //98
    //99
    BackgroundBrightBlack = 100,
    BackgroundBrightRed,
    BackgroundBrightGreen,
    BackgroundBrightYellow,
    BackgroundBrightBlue,
    BackgroundBrightMagenta,
    BackgroundBrightCyan,
    BackgroundBrightWhite
}

impl Display for SelectGraphicRendition {
    fn fmt(&self, fmtr: &mut Formatter<'_>)
 -> fmt::Result {
        fmtr.write_char('')?; //ANSI escape
        fmtr.write_char('\x5b')?; //control sequence introducer (CSI) - '[' in ascii

        let discriminant: &u8 = unsafe { &*ptr::from_ref(self).cast() };
        discriminant.fmt(fmtr)?;

        match *self {
            Self::ForegroundColor(color) |
            Self::BackgroundColor(color) |
            Self::UnderlineColor(color) => color.fmt(fmtr)?,
            _ => ()
        }

        fmtr.write_char('\x6d')?; //control sequence final byte - 'm' in ascii - supposedly anything 0x40–0x7E (ASCII @A–Z[\]^_`a–z{|}~) should work, but somehow only this one actually does

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Palette(u8),
    Rgb(u8,u8,u8),
}

impl Display for Color {
    fn fmt(&self, fmtr: &mut Formatter<'_>)
 -> fmt::Result {
        fmtr.write_str(";")?;
        match *self {
            Self::Palette(index) => {
                5.fmt(fmtr)?;
                fmtr.write_char(';')?;
                index.fmt(fmtr)?;
            },
            Self::Rgb(red, green, blue) => {
                2.fmt(fmtr)?;
                fmtr.write_char(';')?;
                red.fmt(fmtr)?;
                fmtr.write_char(';')?;
                green.fmt(fmtr)?;
                fmtr.write_char(';')?;
                blue.fmt(fmtr)?;
            }
        }

        Ok(())
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::Palette(0)
    }
}