#![deny(warnings)]

use std::env;
use std::path::Path;

use env_logger::{Builder, Target};
use log::LevelFilter;

use cnx::text::*;
use cnx::widgets::*;
use cnx::*;

fn init_log() -> Result<()> {
    let mut builder = Builder::new();
    builder.filter(Some("cnx"), LevelFilter::Trace);
    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse_filters(&rust_log);
    }
    builder.target(Target::Stdout);
    builder.try_init()?;
    Ok(())
}

fn main() -> Result<()> {
    init_log()?;

    let attr = Attributes {
        font: Font::new("Noto Sans Mono"),
        fg_color: Color::white(),
        bg_color: Some(Color::grey()),
        padding: Padding::new(5.0, 5.0, 0.0, 0.0),
    };
    let mut active_attr = attr.clone();
    active_attr.bg_color = Some(Color::orange());

    let mut cnx = Cnx::new(Position::Bottom)?;

    cnx_add_widget!(cnx, Pager::new(&cnx, active_attr.clone(), attr.clone()));
    if Path::new("/sys/class/power_supply/BAT0/present").exists() {
        cnx_add_widget!(cnx, Battery::new(&cnx, active_attr.clone(), Color::red()));
    }
    cnx_add_widget!(cnx, ActiveWindowTitle::new(&cnx, attr.clone()));
    cnx_add_widget!(cnx, Volume::new(&cnx, active_attr.clone()));
    cnx_add_widget!(cnx, Clock::new(&cnx, attr.clone()));

    cnx.run()?;

    Ok(())
}
