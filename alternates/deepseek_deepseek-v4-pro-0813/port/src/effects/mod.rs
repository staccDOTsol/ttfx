pub mod beams;
pub mod binarypath;
pub mod bouncyballs;
pub mod bubbles;
pub mod burn;
pub mod colorshift;
pub mod crumble;
pub mod decrypt;
pub mod errorcorrect;
pub mod expand;
pub mod fireworks;
pub mod highlight;
pub mod laseretch;
pub mod matrix;
pub mod middleout;
pub mod overflow;
pub mod pour;
pub mod print;
pub mod rain;
pub mod random_sequence;
pub mod rings;
pub mod smoke;
pub mod spotlights;
pub mod spray;
pub mod swarm;
pub mod sweep;
pub mod synthgrid;
pub mod thunderstorm;
pub mod unstable;
pub mod vhstape;
pub mod waves;
pub mod wipe;
pub mod blackhole;
pub mod orbittingvolley;
pub mod scattered;
pub mod slice;
pub mod slide;

pub trait Effect {
    fn name(&self) -> &str;
    fn frames(&self, input: &str) -> Vec<String>;
}

pub fn registry() -> Vec<Box<dyn Effect>> {
    vec![
        Box::new(beams::Beams::new()),
        Box::new(binarypath::Binarypath::new()),
        Box::new(bouncyballs::Bouncyballs::new()),
        Box::new(bubbles::Bubbles::new()),
        Box::new(burn::Burn::new()),
        Box::new(colorshift::Colorshift::new()),
        Box::new(crumble::Crumble::new()),
        Box::new(decrypt::Decrypt::new()),
        Box::new(errorcorrect::Errorcorrect::new()),
        Box::new(expand::Expand::new()),
        Box::new(fireworks::Fireworks::new()),
        Box::new(highlight::Highlight::new()),
        Box::new(laseretch::Laseretch::new()),
        Box::new(matrix::Matrix::new()),
        Box::new(middleout::Middleout::new()),
        Box::new(overflow::Overflow::new()),
        Box::new(pour::Pour::new()),
        Box::new(print::Print::new()),
        Box::new(rain::Rain::new()),
        Box::new(random_sequence::RandomSequence::new()),
        Box::new(rings::Rings::new()),
        Box::new(smoke::Smoke::new()),
        Box::new(spotlights::Spotlights::new()),
        Box::new(spray::Spray::new()),
        Box::new(swarm::Swarm::new()),
        Box::new(sweep::Sweep::new()),
        Box::new(synthgrid::Synthgrid::new()),
        Box::new(thunderstorm::Thunderstorm::new()),
        Box::new(unstable::Unstable::new()),
        Box::new(vhstape::Vhstape::new()),
        Box::new(waves::Waves::new()),
        Box::new(wipe::Wipe::new()),
        Box::new(blackhole::Blackhole::new()),
        Box::new(orbittingvolley::Orbittingvolley::new()),
        Box::new(scattered::Scattered::new()),
        Box::new(slice::Slice::new()),
        Box::new(slide::Slide::new())
    ]
}

/// Look one effect up by name.
///
/// The harness owns this file, so a core that reasonably expects a lookup here
/// gets one. MEASURED on ds: its cli.rs called `effects::get_effect(...)`, the
/// generated mod.rs offered only `registry()`, and EVERY effect therefore
/// failed to compile against a core that could not build — 3 effects requeued,
/// $0.74 spent, 0/37 kept, with the real error (`cannot find function
/// get_effect`) never surfacing because it looked like ordinary effect churn.
/// Providing both shapes costs one unused-function warning to models that
/// prefer `registry()`.
pub fn get_effect(name: &str) -> Option<Box<dyn Effect>> {
    registry().into_iter().find(|e| e.name() == name)
}

/// Every effect name, for `--list` and CLI validation.
pub fn effect_names() -> Vec<&'static str> {
    vec!["beams", "binarypath", "bouncyballs", "bubbles", "burn", "colorshift", "crumble", "decrypt", "errorcorrect", "expand", "fireworks", "highlight", "laseretch", "matrix", "middleout", "overflow", "pour", "print", "rain", "random_sequence", "rings", "smoke", "spotlights", "spray", "swarm", "sweep", "synthgrid", "thunderstorm", "unstable", "vhstape", "waves", "wipe", "blackhole", "orbittingvolley", "scattered", "slice", "slide"]
}
