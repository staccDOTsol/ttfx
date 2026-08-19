use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn lerp(self, other: Self, progress: f64) -> Self {
        let progress = progress.clamp(0.0, 1.0);

        let interpolate = |start: u8, end: u8| {
            (start as f64 + (end as f64 - start as f64) * progress)
                .round()
                .clamp(0.0, 255.0) as u8
        };

        Self::new(
            interpolate(self.red, other.red),
            interpolate(self.green, other.green),
            interpolate(self.blue, other.blue),
        )
    }

    pub fn foreground_ansi(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.red, self.green, self.blue)
    }

    pub fn background_ansi(self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.red, self.green, self.blue)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:02x}{:02x}{:02x}",
            self.red, self.green, self.blue
        )
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.strip_prefix('#').unwrap_or(value);

        if value.len() != 6 {
            return Err("colors must contain exactly six hexadecimal digits".to_owned());
        }

        let red = u8::from_str_radix(&value[0..2], 16)
            .map_err(|_| format!("invalid color {value:?}"))?;
        let green = u8::from_str_radix(&value[2..4], 16)
            .map_err(|_| format!("invalid color {value:?}"))?;
        let blue = u8::from_str_radix(&value[4..6], 16)
            .map_err(|_| format!("invalid color {value:?}"))?;

        Ok(Self::new(red, green, blue))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorPair {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}

impl ColorPair {
    pub const fn new(
        foreground: Option<Color>,
        background: Option<Color>,
    ) -> Self {
        Self {
            foreground,
            background,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strike: bool,
}

impl Style {
    pub fn with_colors(colors: ColorPair) -> Self {
        Self {
            foreground: colors.foreground,
            background: colors.background,
            ..Self::default()
        }
    }

    pub fn ansi_prefix(self) -> String {
        let mut output = String::new();

        if let Some(color) = self.foreground {
            output.push_str(&color.foreground_ansi());
        }
        if let Some(color) = self.background {
            output.push_str(&color.background_ansi());
        }
        if self.bold {
            output.push_str("\x1b[1m");
        }
        if self.italic {
            output.push_str("\x1b[3m");
        }
        if self.underline {
            output.push_str("\x1b[4m");
        }
        if self.blink {
            output.push_str("\x1b[5m");
        }
        if self.reverse {
            output.push_str("\x1b[7m");
        }
        if self.hidden {
            output.push_str("\x1b[8m");
        }
        if self.strike {
            output.push_str("\x1b[9m");
        }

        output
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gradient {
    stops: Vec<Color>,
    steps: usize,
}

impl Gradient {
    pub fn new(stops: impl IntoIterator<Item = Color>, steps: usize) -> Self {
        Self {
            stops: stops.into_iter().collect(),
            steps: steps.max(1),
        }
    }

    pub fn colors(&self) -> Vec<Color> {
        match self.stops.as_slice() {
            [] => Vec::new(),
            [color] => vec![*color; self.steps],
            stops => {
                let mut colors = Vec::with_capacity(self.steps);

                for index in 0..self.steps {
                    let progress = if self.steps == 1 {
                        0.0
                    } else {
                        index as f64 / (self.steps - 1) as f64
                    };

                    let scaled = progress * (stops.len() - 1) as f64;
                    let segment = (scaled.floor() as usize).min(stops.len() - 2);
                    let segment_progress = scaled - segment as f64;

                    colors.push(
                        stops[segment].lerp(stops[segment + 1], segment_progress),
                    );
                }

                colors
            }
        }
    }
}
