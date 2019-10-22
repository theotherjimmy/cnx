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
        fg_color: "3c3836".parse().unwrap(),
        bg_color: Some("fbf1c7".parse().unwrap()),
        padding: Padding::new(5.0, 5.0, 0.0, 0.0),
    };
    let mut active_attr = attr.clone();
    active_attr.bg_color = Some("d65d0e".parse().unwrap());

    let mut cnx = Cnx::new(Position::Bottom)?;

    if Path::new("/sys/class/power_supply/BAT0/present").exists() {
        cnx.add_widget(Battery::new(
            &cnx,
            active_attr.clone(),
            "cc241d".parse().unwrap(),
        ));
    }
    cnx.add_widget(ActiveWindowTitle::new(&cnx, attr.clone()));
    cnx.add_widget(Pager::new(&cnx, active_attr.clone(), attr.clone()));
    cnx.add_widget(Clock::new(
        &cnx,
        String::from("%Y-%m-%d %a %I:%M"),
        attr.clone(),
    ));

    cnx.run()?;

    Ok(())
}
