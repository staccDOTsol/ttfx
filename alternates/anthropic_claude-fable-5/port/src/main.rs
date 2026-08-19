use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use ttfx::effects::registry;

/// ttfx — terminal text effects (Rust port of TerminalTextEffects).
#[derive(Parser, Debug)]
#[command(name = "ttfx", version, about)]
struct Cli {
    /// Name of the effect to run. Omit to list available effects.
    effect: Option<String>,

    /// Milliseconds between frames.
    #[arg(long, default_value_t = 40)]
    frame_delay: u64,

    /// Print frames without clearing the screen between them.
    #[arg(long, default_value_t = false)]
    no_clear: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() || input.trim().is_empty() {
        input = String::from("ttfx");
    }

    let effects = registry();

    let Some(name) = cli.effect else {
        if effects.is_empty() {
            eprintln!("no effects registered yet");
        } else {
            eprintln!("available effects:");
            for effect in &effects {
                eprintln!("  {}", effect.name());
            }
        }
        std::process::exit(1);
    };

    let Some(effect) = effects.iter().find(|e| e.name() == name) else {
        eprintln!("unknown effect: {name}");
        if effects.is_empty() {
            eprintln!("(no effects registered yet)");
        } else {
            eprintln!("available effects:");
            for effect in &effects {
                eprintln!("  {}", effect.name());
            }
        }
        std::process::exit(1);
    };

    let frames = effect.frames(&input);
    let mut stdout = io::stdout();
    for frame in frames {
        if !cli.no_clear {
            let _ = execute!(stdout, Clear(ClearType::All), MoveTo(0, 0));
        }
        let _ = write!(stdout, "{frame}");
        if cli.no_clear {
            let _ = writeln!(stdout);
        }
        let _ = stdout.flush();
        thread::sleep(Duration::from_millis(cli.frame_delay));
    }
    let _ = writeln!(stdout);
}
