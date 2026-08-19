use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "ttfx",
    version,
    about = "TerminalTextEffects Rust port"
)]
pub struct Cli {
    /// Path to input file. Reads from stdin when omitted.
    #[arg(long = "input-file")]
    pub input_file: Option<PathBuf>,

    /// Randomly select an effect.
    #[arg(long = "random-effect")]
    pub random_effect: bool,

    /// Effects to include when randomly selecting an effect.
    #[arg(long = "include-effects", num_args = 1..)]
    pub include_effects: Vec<String>,

    /// Effects to exclude when randomly selecting an effect.
    #[arg(long = "exclude-effects", num_args = 1..)]
    pub exclude_effects: Vec<String>,

    /// Print shell completion script for the given shell.
    #[arg(long = "print-completion", value_parser = ["bash", "zsh"])]
    pub print_completion: Option<String>,

    /// Seed for deterministic random selection and effect behavior.
    #[arg(long = "seed")]
    pub seed: Option<u64>,

    /// Name of the effect to apply.
    #[arg(value_name = "EFFECT")]
    pub effect: Option<String>,
}
