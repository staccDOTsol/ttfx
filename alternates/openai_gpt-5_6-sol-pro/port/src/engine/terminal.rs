use std::io::{self, IsTerminal, Write};
use std::thread;
use std::time::Duration;

use crossterm::{
    cursor,
    execute,
    terminal::{self, ClearType},
};

use crate::engine::canvas::Canvas;
use crate::engine::character::EffectCharacter;

#[derive(Debug)]
pub struct Terminal {
    pub canvas: Canvas,
    characters: Vec<EffectCharacter>,
    frame_rate: f64,
}

impl Terminal {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            canvas: Canvas::new(width, height),
            characters: Vec::new(),
            frame_rate: 30.0,
        }
    }

    pub fn from_terminal_size() -> Self {
        let (width, height) = terminal::size().unwrap_or((80, 24));
        Self::new(width as usize, height as usize)
    }

    pub fn set_frame_rate(&mut self, frame_rate: f64) {
        self.frame_rate = frame_rate.max(0.0);
    }

    pub fn add_character(&mut self, character: EffectCharacter) {
        self.characters.push(character);
    }

    pub fn characters(&self) -> &[EffectCharacter] {
        &self.characters
    }

    pub fn characters_mut(&mut self) -> &mut [EffectCharacter] {
        &mut self.characters
    }

    pub fn step_frame(&mut self) -> String {
        self.canvas.clear();

        for character in &mut self.characters {
            character.step();
            self.canvas.draw_character(character);
        }

        self.canvas.render()
    }

    pub fn has_active_characters(&self) -> bool {
        self.characters
            .iter()
            .any(EffectCharacter::is_active)
    }

    pub fn run(&mut self, max_frames: usize) -> Vec<String> {
        let mut frames = Vec::new();

        for _ in 0..max_frames {
            frames.push(self.step_frame());

            if !self.has_active_characters() {
                break;
            }
        }

        frames
    }

    pub fn play(&mut self, max_frames: usize) -> io::Result<()> {
        let frames = self.run(max_frames);
        Self::play_frames(&frames, self.frame_rate)
    }

    pub fn play_frames(frames: &[String], frame_rate: f64) -> io::Result<()> {
        if frames.is_empty() {
            return Ok(());
        }

        let stdout = io::stdout();
        let interactive = stdout.is_terminal();
        let mut stdout = stdout.lock();
        let frame_delay = if frame_rate > 0.0 {
            Some(Duration::from_secs_f64(1.0 / frame_rate))
        } else {
            None
        };

        if interactive {
            execute!(stdout, cursor::Hide)?;
        }

        for (index, frame) in frames.iter().enumerate() {
            if interactive {
                execute!(
                    stdout,
                    cursor::MoveTo(0, 0),
                    terminal::Clear(ClearType::All)
                )?;
            }

            stdout.write_all(frame.as_bytes())?;

            if !interactive || index + 1 == frames.len() {
                stdout.write_all(b"\n")?;
            }

            stdout.flush()?;

            if index + 1 < frames.len() {
                if let Some(delay) = frame_delay {
                    thread::sleep(delay);
                }
            }
        }

        if interactive {
            execute!(stdout, cursor::Show)?;
        }

        Ok(())
    }
}
