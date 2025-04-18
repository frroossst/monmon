use core::fmt;

use colored::Colorize;

#[derive(Debug)]
pub struct Config {
    _mode: ConfigKind,
    pub num_producer: usize,
    pub per_producer: usize,
}

#[derive(Debug)]
pub enum ConfigKind {
    Fast,
    Medium,
    Slow,
}

impl Config {
    pub fn new(config: ConfigKind) -> Self {
        match config {
            ConfigKind::Fast => Config {
                num_producer: 4,
                per_producer: 50,
                _mode: ConfigKind::Fast,
            },
            ConfigKind::Medium => Config {
                num_producer: 64,
                per_producer: 50,
                _mode: ConfigKind::Medium,
            },
            ConfigKind::Slow => Config {
                num_producer: 1024,
                per_producer: 50,
                _mode: ConfigKind::Slow,
            },
        }
    }
}

pub struct RaceCondition {
    pub expected: usize,
    pub actual: usize,
}

impl RaceCondition {
    pub fn new(expected: usize, actual: usize) -> Self {
        RaceCondition { expected, actual }
    }
}

impl fmt::Debug for RaceCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.expected != self.actual {
            writeln!(f, "{}", "[RACE CONDITION]".red().bold().blink())?;
            writeln!(f, "Expected: {}, Actual: {}", self.expected, self.actual)?;
            writeln!(
                f,
                "Missing items: {}",
                format!("{}", self.expected.saturating_sub(self.actual))
                    .bright_white()
                    .italic()
            )?;
        } else {
            writeln!(f, "{}", "[NO RACE]".bright_green().bold())?;
        }
        Ok(())
    }
}
