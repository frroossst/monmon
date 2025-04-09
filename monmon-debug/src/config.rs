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
                num_producer: 5,
                per_producer: 50,
                _mode: ConfigKind::Fast,
            },
            ConfigKind::Medium => Config {
                num_producer: 50,
                per_producer: 500,
                _mode: ConfigKind::Medium,
            },
            ConfigKind::Slow => Config {
                num_producer: 101,
                per_producer: 1000,
                _mode: ConfigKind::Slow,
            },
        }
    }
}
