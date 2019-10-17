#![deny(warnings)]

//! A simple X11 status bar for use with simple WMs.
//!
//! Cnx is written to be customisable, simple and fast. Where possible, it
//! prefers to asynchronously wait for changes in the underlying data sources
//! (and uses [`mio`]/[`tokio`] to achieve this), rather than periodically
//! calling out to external programs.
//!
//! # How to use
//!
//! Cnx is a library that allows you to make your own status bar.
//!
//! In normal usage, you will create a new binary project that relies on the
//! `cnx` crate, and customize it through options passed to the main [`Cnx`]
//! object and its widgets. (It's inspired by [`QTile`] and [`dwm`], in that the
//! configuration is done entirely in code, allowing greater extensibility
//! without needing complex configuration handling).
//!
//! An simple example of a binary using Cnx is:
//!
//! ```no_run
//! #[macro_use]
//! extern crate cnx;
//!
//! use cnx::*;
//! use cnx::text::*;
//! use cnx::widgets::*;
//!
//! fn main() -> Result<()> {
//!     let attr = Attributes {
//!         font: Font::new("SourceCodePro 21"),
//!         fg_color: Color::white(),
//!         bg_color: None,
//!         padding: Padding::new(8.0, 8.0, 0.0, 0.0),
//!     };
//!
//!     let mut cnx = Cnx::new(Position::Top)?;
//!     cnx_add_widget!(cnx, ActiveWindowTitle::new(&cnx, attr.clone()));
//!     cnx_add_widget!(cnx, Clock::new(&cnx, attr.clone()));
//!     Ok(cnx.run()?)
//! }
//! ```
//!
//! A more complex example is given in [`src/bin/cnx.rs`] alongside the project.
//! (This is the default `[bin]` target for the crate, so you _could_ use it by
//! either executing `cargo run` from the crate root, or even running `cargo
//! install cnx; cnx`. However, neither of these are recommended as options for
//! customizing Cnx are then limited).
//!
//! Before running Cnx, you'll need to make sure your system has the required
//! dependencies, which are described in the [`README`][readme-deps].
//!
//! # Built-in widgets
//!
//! There are currently these widgets available:
//!
//! - [`Active Window Title`] — Shows the title ([`EWMH`]'s `_NET_WM_NAME`) for
//!   the currently focused window ([`EWMH`]'s `_NEW_ACTIVE_WINDOW`).
//! - [`Pager`] — Shows the WM's workspaces/groups, highlighting whichever is
//!   currently active. (Uses [`EWMH`]'s `_NET_DESKTOP_NAMES`,
//!   `_NET_NUMBER_OF_DESKTOPS` and `_NET_CURRENT_DESKTOP`).
//! - [`Sensors`] — Periodically parses and displays the output of the
//!   [`lm_sensors`] utility, allowing CPU temperature to be displayed.
//! - [`Volume`] — Uses `alsa-lib` to show the current volume/mute status of the
//!   default output device. (Disable by removing default feature
//!   `volume-control`).
//! - [`Battery`] — Uses `/sys/class/power_supply/` to show details on the
//!   remaining battery and charge status.
//! - [`Clock`] — Shows the time.
//!
//! # Dependencies
//!
//! In addition to the Rust dependencies in `Cargo.toml`, Cnx also depends on
//! these system libraries:
//!
//!  - `xcb-util`: `xcb-ewmh` / `xcb-icccm` / `xcb-keysyms`
//!  - `x11-xcb`
//!  - `pango`
//!  - `cairo`
//!  - `pangocairo`
//!
//! Some widgets have additional dependencies:
//!
//!  - [`Volume`] widget relies on `alsa-lib`
//!  - [`Sensors`] widget relies on [`lm_sensors`] being installed.
//!
//! # Creating new widgets
//!
//! Cnx is designed such that thirdparty widgets can be written in external
//! crates and used with the main [`Cnx`] instance. However, this API is
//! currently very unstable and isn't recommended.
//!
//! The adventurous may choose to ignore this warning and look into the
//! documentation of the [`Widget`] trait. The built-in [`widgets`] should give you
//! some examples on which to base your work.
//!
//! [`mio`]: https://docs.rs/mio
//! [`tokio`]: https://tokio.rs/
//! [`Cnx`]: struct.Cnx.html
//! [`dwm`]: http://dwm.suckless.org/
//! [readme-deps]: https://github.com/mjkillough/cnx/blob/master/README.md#dependencies
//! [`src/bin/cnx.rs`]: https://github.com/mjkillough/cnx/blob/master/src/bin/cnx.rs
//! [`Active Window Title`]: widgets/struct.ActiveWindowTitle.html
//! [`EWMH`]: https://specifications.freedesktop.org/wm-spec/wm-spec-latest.html
//! [`Pager`]: widgets/struct.Pager.html
//! [`Sensors`]: widgets/struct.Sensors.html
//! [`lm_sensors`]: https://wiki.archlinux.org/index.php/lm_sensors
//! [`Volume`]: widgets/struct.Volume.html
//! [`Battery`]: widgets/struct.Battery.html
//! [`Clock`]: widgets/struct.Clock.html
//! [`Widget`]: widgets/trait.Widget.html
//! [`widgets`]: widgets/index.html

// new(...) -> Result<T> is used in a lot of places:
#![allow(clippy::new_ret_no_self)]

mod bar;
pub mod text;
pub mod widgets;

use failure::ResultExt;
use tokio_core::reactor::{Core, Handle};
use tokio_timer::Timer;

use crate::bar::Bar;

pub use crate::bar::Position;
pub use crate::widgets::Widget;

pub type Result<T> = std::result::Result<T, failure::Error>;

/// The main object, used to instantiate an instance of Cnx.
///
/// The [`cnx_add_widget!()`] macro can be used to add widgets to the Cnx
/// instance. Once configured, the [`run()`] method will take ownership of the
/// instance and run it until the process is killed or an error is returned.
///
/// [`cnx_add_widget!()`]: macro.cnx_add_widget.html
/// [`run()`]: #method.run
///
/// # Examples
///
/// ```no_run
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
/// cnx_add_widget!(cnx, ActiveWindowTitle::new(&cnx, attr.clone()));
/// cnx_add_widget!(cnx, Clock::new(&cnx, attr.clone()));
/// cnx.run()?;
/// # Ok(())
/// # }
/// # fn main() { run().unwrap(); }
/// ```
pub struct Cnx {
    core: Core,
    timer: Timer,
    bar: Bar,
    widgets: Vec<Box<dyn Widget>>,
}

impl Cnx {
    /// Creates a new `Cnx` instance.
    ///
    /// This creates a new `Cnx` instance at either the top or bottom of the
    /// screen, depending on the value of the [`Position`] enum.
    ///
    /// [`Position`]: enum.Position.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use cnx::{Cnx, Position};
    /// let mut cnx = Cnx::new(Position::Top);
    /// ```
    /// ```
    /// # use cnx::{Cnx, Position};
    /// let mut cnx = Cnx::new(Position::Bottom);
    /// ```
    pub fn new(position: Position) -> Result<Cnx> {
        Ok(Cnx {
            core: Core::new().context("Could not create Tokio Core")?,
            timer: Timer::default(),
            bar: Bar::new(position)?,
            widgets: Vec::new(),
        })
    }

    fn handle(&self) -> Handle {
        self.core.handle()
    }

    fn timer(&self) -> Timer {
        self.timer.clone()
    }

    /// Adds a widget to the Cnx instance.
    ///
    /// This method takes a [`Widget`] and adds it to the current Cnx instance,
    /// to the right of any existing widgets.
    ///
    /// It is recommended that you instead use the [`cnx_add_widget!()`] macro,
    /// as this will eventually grow to have a more flexible syntax for
    /// configuring widget attributes.
    ///
    /// [`Widget`]: widgets/trait.Widget.html
    /// [`cnx_add_widget!()`]: macro.cnx_add_widget.html
    pub fn add_widget<W>(&mut self, widget: W)
    where
        W: Widget + 'static,
    {
        self.widgets.push(Box::new(widget) as Box<dyn Widget>);
    }

    /// Runs the Cnx instance.
    ///
    /// This method takes ownership of the Cnx instance and runs it until either
    /// the process is terminated, or an internal error is returned.
    pub fn run(mut self) -> Result<()> {
        let handle = self.handle();
        self.core
            .run(self.bar.run_event_loop(&handle, self.widgets)?)
    }
}
