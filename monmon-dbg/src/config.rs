use core::fmt;

use colored::Colorize;

pub enum RaceKind {
    UnsafeAccum,
    UnsafeBuffer,
    StdlibMutexAccum,
    StdlibMutexBuffer,
    BinarySemaphoreAccum,
    BinarySemaphoreBuffer,
    HappyLockAccum,
    HappyLockBuffer,
    SemaphoreMonitorAccum,
    SemaphoreMonitorBuffer,
    FutexMonitorAccum,
    FutexMonitorBuffer,
}

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

use num_traits::{SaturatingSub, Zero};

pub struct RaceCondition<T> {
    pub expected: T,
    pub actual: T,
}

impl<T> RaceCondition<T>
where
    T: Copy,
{
    pub fn new(expected: T, actual: T) -> Self {
        RaceCondition { expected, actual }
    }
}

impl<T> fmt::Debug for RaceCondition<T>
where
    T: fmt::Display + PartialEq + SaturatingSub + Copy + Zero,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.expected != self.actual {
            writeln!(f, "{}", "[RACE CONDITION]".red().bold().blink())?;
            writeln!(f, "Expected: {}, Actual: {}", self.expected, self.actual)?;
            writeln!(
                f,
                "Missing items: {}",
                self.expected
                    .saturating_sub(&self.actual)
                    .to_string()
                    .bright_white()
                    .italic()
            )?;
        } else {
            writeln!(f, "{}", "[NO RACE]".bright_green().bold())?;
        }
        Ok(())
    }
}
