use std::sync::Arc;

use monmon_impl::semaphore::SemaphoreMonitor;

use monmon_debug::config::{Config, ConfigKind};



// static accumulator 



fn benchmark() {

}


fn main() {
    println!("Hello, world!");

    let config: Config = Config::new(ConfigKind::Fast);

    let mut mon = Arc::new(SemaphoreMonitor::new(2));
}
