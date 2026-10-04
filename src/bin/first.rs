use std::time::Duration;

use scheduler::{engine::Engine, sched::rr::RoundRobin};

fn main() {
    // create the engine
    let mut engine = Engine::new(RoundRobin::new(Duration::from_millis(10), 3));

    // create 10 tasks without parents
    for _ in 0..1000 {
        engine.create(None);
    }

    // run until there's no more tasks
    while engine.run_tick().is_some() {}
}
