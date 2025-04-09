pub struct Config {
    pub num_producer: usize,
    pub per_producer: usize,
}

pub enum ConfigKind {
    Fast,
    Medium,
    Slow,
}

impl Config {

    pub fn new(config: ConfigKind) -> Self {
        match config {
            ConfigKind::Fast => Config {
                num_producer: 10,
                per_producer: 100,
            },
            ConfigKind::Medium => Config {
                num_producer: 25,
                per_producer: 100,
            },
            ConfigKind::Slow => Config {
                num_producer: 101,
                per_producer: 100,
            },
        }
    }

}
