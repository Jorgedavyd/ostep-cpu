use std::time::Duration;

use scheduler::{engine::Engine, sched::mlfq::MultiLevelFeedbackQueue};

fn main() {
    // tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // create the engine
    let mut engine = Engine::new(MultiLevelFeedbackQueue::new(Duration::from_millis(500), Duration::from_millis(10), Duration::from_millis(20)));

    // create 10 tasks without parents
    for _ in 0..1000 {
        engine.create(None);
    }

    // run until there's no more tasks
    while engine.run_tick().is_some() {}
}
