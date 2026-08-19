use clap::Parser;
use std::io::{self, Read};
use ttfx::effects::{registry, Effect};

#[derive(Parser, Debug)]
#[command(name = "ttfx", about = "Terminal text effects")]
struct Cli {
    /// Effect name (none registered in this skeleton)
    #[arg(default_value = "")]
    effect: String,
}

fn main() {
    let cli = Cli::parse();
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);

    let effects: Vec<Box<dyn Effect>> = registry();
    if let Some(eff) = effects.iter().find(|e| e.name() == cli.effect) {
        for frame in eff.frames(&input) {
            print!("{frame}");
        }
    } else if !cli.effect.is_empty() {
        eprintln!("unknown effect: {}", cli.effect);
        std::process::exit(1);
    } else {
        print!("{input}");
    }
}
