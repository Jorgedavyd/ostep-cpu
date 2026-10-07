use std::{collections::VecDeque, time::Duration};

use crate::{engine::ProcessTable, process::{Pid, Process}, sched::scheduler::Scheduler};

pub struct RoundRobin {
    tasks: VecDeque<(Pid, Duration)>,

    // Round robin configuration
    time_slice: Duration,

    // last task state tracking
    last_task: Option<(Pid, Duration)>,
}

impl Scheduler for RoundRobin {
    fn next(&mut self, _: &ProcessTable) -> Option<Pid> {
        let mut last_task = self.tasks.pop_front()?;
        last_task.1 += self.time_slice;
        self.last_task = Some(last_task);
        Some(last_task.0)
    }

    fn enqueue(&mut self, process: &Process) {
        if let Some((last_task_pid, allotment)) = self.last_task
            && process.id == last_task_pid
                && allotment < self.time_slice
        {
            self.tasks.push_front((process.id, allotment));
        } else {
            self.tasks.push_back((process.id, Duration::ZERO));
        }
    }
}

impl RoundRobin {
    pub fn new(interrupt: Duration, slice_mult: u32) -> Self {
        Self {
            tasks: VecDeque::new(),
            time_slice: interrupt * slice_mult,
            last_task: None,
        }
    }
}
