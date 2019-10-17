use std::time::Duration;

use chrono::prelude::*;
use failure::ResultExt;
use futures::{stream, Future, Stream};
use tokio_timer::Timer;

use super::{Widget, WidgetStream};
use crate::text::{Attributes, Text};
use crate::{Cnx, Result};

/// Shows the current time and date.
///
/// This widget shows the current time and date, in the form `%Y-%m-%d %a %I:%M
/// %p`, e.g. `2017-09-01 Fri 12:51 PM`.
pub struct Clock {
    format: String,
    timer: Timer,
    attr: Attributes,
}

impl Clock {
    /// Creates a new Clock widget.
    ///
    /// Creates a new `Clock` widget, whose text will be displayed with the
    /// given [`Attributes`].
    ///
    /// The [`Cnx`] instance is borrowed during construction in order to get
    /// access to handles of its event loop. However, it is not borrowed for the
    /// lifetime of the widget. See the [`cnx_add_widget!()`] for more
    /// discussion about the lifetime of the borrow.
    ///
    /// [`Attributes`]: ../text/struct.Attributes.html
    /// [`Cnx`]: ../struct.Cnx.html
    /// [`cnx_add_widget!()`]: ../macro.cnx_add_widget.html
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate cnx;
    /// #
    /// # use cnx::*;
    /// # use cnx::text::*;
    /// # use cnx::widgets::*;
    /// #
    /// # fn run() -> ::cnx::Result<()> {
    /// let attr = Attributes {
    ///     font: Font::new("SourceCodePro 21"),
    ///     fg_color: Color::white(),
    ///     bg_color: None,
    ///     padding: Padding::new(8.0, 8.0, 0.0, 0.0),
    /// };
    ///
    /// let mut cnx = Cnx::new(Position::Top)?;
    /// cnx.add_widget(String::from("%Y-%m-%d %a %I:%M %p")) Clock::new(&cnx, attr.clone()));
    /// # Ok(())
    /// # }
    /// # fn main() { run().unwrap(); }
    /// ```
    pub fn new(cnx: &Cnx, format: String, attr: Attributes) -> Clock {
        Clock {
            format,
            timer: cnx.timer(),
            attr,
        }
    }
}

impl Widget for Clock {
    fn stream(self: Box<Self>) -> Result<WidgetStream> {
        // As we're not showing seconds, we can sleep for however long it takes
        // until the minutes changes between updates. Initially sleep for 0 seconds
        // so that our `self.timer.sleep()` expires immediately.
        let sleep_for = Duration::from_secs(0);
        let stream = stream::unfold(sleep_for, move |sleep_for| {
            // Avoid having to move self into the .map() closure.
            let attr = self.attr.clone();
            let format_str = self.format.clone();
            Some(self.timer.sleep(sleep_for).map(move |()| {
                let now = Local::now();
                let formatted = now.format(&format_str).to_string();
                let texts = vec![Text {
                    attr: attr,
                    text: formatted,
                    stretch: false,
                }];

                let sleep_for = Duration::from_secs(60 - u64::from(now.second()));
                (texts, sleep_for)
            }))
        })
        .then(|r| r.context("Error in tokio_timer stream"))
        .map_err(|e| e.into());

        Ok(Box::new(stream))
    }
}
