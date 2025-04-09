pub struct Config {
    num_producers: usize,
    num_consumers: usize,
    per_producer: usize,
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
                num_producers: 10,
                num_consumers: 10,
                per_producer: 100,
            },
            ConfigKind::Medium => Config {
                num_producers: 25,
                num_consumers: 25,
                per_producer: 100,
            },
            ConfigKind::Slow => Config {
                num_producers: 101,
                num_consumers: 101,
                per_producer: 100,
            },
        }
    }

}
