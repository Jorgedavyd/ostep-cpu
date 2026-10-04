use std::{collections::VecDeque, time::Duration};

use crate::{engine::ProcessTable, process::{Pid, Process}, sched::scheduler::Scheduler};

pub struct RoundRobin {
    tasks: VecDeque<Pid>,

    // Round robin configuration
    time_slice: Duration,

    // last task state tracking
    last_task: Option<Pid>,
}

impl Scheduler for RoundRobin {
    fn next(&mut self, _: &ProcessTable) -> Option<Pid> {
        self.last_task = self.tasks.pop_front();
        self.last_task
    }

    fn enqueue(&mut self, process: &Process) {
        if let Some(last_task_pid) = self.last_task
            && process.id == last_task_pid
                && process.runtime < self.time_slice
        {
            self.tasks.push_front(process.id);
        } else {
            self.tasks.push_back(process.id);
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
