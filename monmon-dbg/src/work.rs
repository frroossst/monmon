use rand::Rng;
use std::thread;

pub fn do_something() {
    let _: () = {
        // either randomly sleep, busy wait, or do nothing
        let mut rng = rand::rng();
        let random_number = rng.random_range(0..3);
        match random_number {
            0 => {
                let sleep_duration = rng.random_range(1..50);
                thread::sleep(std::time::Duration::from_millis(sleep_duration));
            }
            1 => {
                let busy_wait_duration = rng.random_range(1..50);
                let start_time = std::time::Instant::now();
                while start_time.elapsed().as_millis() < busy_wait_duration as u128 {}
            }
            _ => {
                // Do nothing
            }
        }
    };
    std::hint::black_box(());
}
