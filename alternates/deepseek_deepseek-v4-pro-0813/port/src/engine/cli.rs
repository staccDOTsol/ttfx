//! Command-line interface for ttfx.

use clap::{Arg, ArgAction, Command, ValueHint};

/// Build the root CLI command.
///
/// Effect subcommands are supplied by the engine effect registry so this
/// module does not need to know every bundled effect at compile time.
pub fn build_cli(subcommands: Vec<Command>) -> Command {
    Command::new("ttfx")
        .version(env!("CARGO_PKG_VERSION"))
        .about("TerminalTextEffects reimplementation in Rust")
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Input file to read. Reads stdin when omitted.")
                .value_hint(ValueHint::FilePath),
        )
        .arg(
            Arg::new("seed")
                .long("seed")
                .value_name("SEED")
                .help("Random seed for deterministic runs.")
                .value_parser(clap::value_parser!(u64)),
        )
        .arg(
            Arg::new("ignore_terminal_dimensions")
                .long("ignore-terminal-dimensions")
                .action(ArgAction::SetTrue)
                .help("Use input dimensions instead of detected terminal dimensions."),
        )
        .arg(
            Arg::new("terminal_width")
                .long("terminal-width")
                .value_name("COLS")
                .help("Override terminal width.")
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("terminal_height")
                .long("terminal-height")
                .value_name("ROWS")
                .help("Override terminal height.")
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("frame_rate")
                .long("frame-rate")
                .value_name("FPS")
                .help("Frames per second for time-based effects.")
                .default_value("15")
                .value_parser(clap::value_parser!(f64)),
        )
        .subcommands(subcommands)
}
