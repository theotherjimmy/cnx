use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use cairo::{Context, Surface};
use failure::format_err;
use pango::{Alignment, EllipsizeMode, FontDescription, LayoutExt};
use pangocairo;

use crate::Result;

#[derive(Clone, Debug, PartialEq)]
pub struct Color {
    red: f64,
    green: f64,
    blue: f64,
}

#[derive(Debug)]
pub enum ParseColorError {
    Red(ParseIntError),
    Green(ParseIntError),
    Blue(ParseIntError),
}

impl Color {
    pub fn apply_to_context(&self, cr: &Context) {
        cr.set_source_rgb(self.red, self.green, self.blue);
    }
}

impl Default for Color {
    fn default() -> Self {
        Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        }
    }
}

impl FromStr for Color {
    type Err = ParseColorError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Color {
            red: (u8::from_str_radix(&s[0..2], 16).map_err(ParseColorError::Red)?) as f64 / 256.0,
            green: (u8::from_str_radix(&s[2..4], 16).map_err(ParseColorError::Green)?) as f64
                / 256.0,
            blue: (u8::from_str_radix(&s[4..6], 16).map_err(ParseColorError::Blue)?) as f64 / 256.0,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Padding {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

impl Padding {
    pub fn new(left: f64, right: f64, top: f64, bottom: f64) -> Padding {
        Padding {
            left,
            right,
            top,
            bottom,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Font(FontDescription);

impl Font {
    pub fn new(name: &str) -> Font {
        Font(FontDescription::from_string(name))
    }
}

impl fmt::Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attributes {
    pub font: Font,
    pub fg_color: Color,
    pub bg_color: Option<Color>,
    pub padding: Padding,
}

fn create_pango_layout(cairo_context: &cairo::Context) -> Result<pango::Layout> {
    let layout = pangocairo::functions::create_layout(cairo_context)
        .ok_or_else(|| format_err!("Failed to create Pango layout"))?;
    Ok(layout)
}

fn show_pango_layout(cairo_context: &cairo::Context, layout: &pango::Layout) {
    pangocairo::functions::show_layout(cairo_context, layout);
}

#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    pub attr: Attributes,
    pub text: String,
    pub stretch: bool,
}

impl Text {
    pub(crate) fn compute(self, surface: &Surface) -> Result<ComputedText> {
        let (width, height) = {
            let context = Context::new(&surface);
            let layout = create_pango_layout(&context)?;
            layout.set_text(&self.text);
            layout.set_font_description(Some(&self.attr.font.0));

            let padding = &self.attr.padding;
            let (text_width, text_height) = layout.get_pixel_size();
            let width = f64::from(text_width) + padding.left + padding.right;
            let height = f64::from(text_height) + padding.top + padding.bottom;
            (width, height)
        };

        Ok(ComputedText {
            attr: self.attr,
            text: self.text,
            stretch: self.stretch,
            x: 0.0,
            y: 0.0,
            width,
            height,
        })
    }
}

// This impl allows us to see whether a widget's text has changed without
// having to call the (relatively) expensive .compute().
impl PartialEq<ComputedText> for Text {
    fn eq(&self, other: &ComputedText) -> bool {
        self.attr == other.attr && self.text == other.text && self.stretch == other.stretch
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ComputedText {
    pub attr: Attributes,
    pub text: String,
    pub stretch: bool,

    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ComputedText {
    pub fn render(&self, surface: &Surface) -> Result<()> {
        let context = Context::new(&surface);
        let layout = create_pango_layout(&context)?;
        layout.set_text(&self.text);
        layout.set_font_description(Some(&self.attr.font.0));

        context.translate(self.x, self.y);

        // Set the width/height on the Pango layout so that it word-wraps/ellipises.
        let padding = &self.attr.padding;
        let text_width = self.width - padding.left - padding.right;
        let text_height = self.height - padding.top - padding.bottom;
        layout.set_ellipsize(EllipsizeMode::End);
        layout.set_width(text_width as i32 * pango::SCALE);
        layout.set_height(text_height as i32 * pango::SCALE);
        if self.stretch {
            layout.set_alignment(Alignment::Center)
        }

        let bg_color = &self.attr.bg_color.clone().unwrap_or_default();
        bg_color.apply_to_context(&context);
        // FIXME: The use of `height` isnt' right here: we want to do the
        // full height of the bar, not the full height of the text. It
        // would be useful if we could do Surface.get_height(), but that
        // doesn't seem to be available in cairo-rs for some reason?
        context.rectangle(0.0, 0.0, self.width, self.height);
        context.fill();

        self.attr.fg_color.apply_to_context(&context);
        context.translate(padding.left, padding.top);
        show_pango_layout(&context, &layout);

        Ok(())
    }
}
