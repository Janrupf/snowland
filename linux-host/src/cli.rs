use clap::{App, IntoApp, Parser};

use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, Parser)]
pub struct Cli {
    /// The window to render to, defaults to the root window if not given
    ///
    /// This should be the X window id of the window.
    /// Example: `--window 0x42069`
    #[clap(short, long, parse(try_from_str = parse_maybe_hex))]
    pub window: Option<u64>,
}

fn parse_maybe_hex(input: &str) -> Result<u64, ParseIntError> {
    if let Some(stripped) = input.strip_prefix("0x") {
        u64::from_str_radix(stripped, 16)
    } else {
        u64::from_str(input)
    }
}

#[allow(dead_code)]
pub fn as_app() -> App<'static> {
    Cli::into_app()
}

pub fn parse() -> Cli {
    Cli::parse()
}
