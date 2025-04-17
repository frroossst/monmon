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

#[derive(Debug)]
pub struct RaceCondition {
    pub expected: usize,
    pub actual: usize,
}

impl RaceCondition {
    pub fn new(expected: usize, actual: usize) -> Self {
        RaceCondition { expected, actual }
    }
}
