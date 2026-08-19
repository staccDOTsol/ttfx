use std::error::Error;
use std::io::{self, Read};

use clap::Parser;
use ttfx::effects;
use ttfx::engine::terminal::Terminal;

#[derive(Debug, Parser)]
#[command(
    name = "ttfx",
    version,
    about = "Terminal text effects, implemented in Rust"
)]
struct Cli {
    /// Name of the effect to run.
    effect: String,

    /// Frames rendered per second. Use 0 to disable pacing.
    #[arg(long, default_value_t = 30.0)]
    frame_rate: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let effect = effects::registry()
        .into_iter()
        .find(|effect| effect.name() == cli.effect)
        .ok_or_else(|| {
            let available = effects::registry()
                .into_iter()
                .map(|effect| effect.name().to_owned())
                .collect::<Vec<_>>();

            let message = if available.is_empty() {
                format!(
                    "unknown effect {:?}; no effects are registered yet",
                    cli.effect
                )
            } else {
                format!(
                    "unknown effect {:?}; available effects: {}",
                    cli.effect,
                    available.join(", ")
                )
            };

            io::Error::new(io::ErrorKind::InvalidInput, message)
        })?;

    let frames = effect.frames(&input);
    Terminal::play_frames(&frames, cli.frame_rate)?;

    Ok(())
}
