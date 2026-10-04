use std::collections::VecDeque;

use crate::{process::{Pid, Process}, sched::scheduler::Scheduler};

#[derive(Default)]
pub struct Fifo {
    tasks: VecDeque<Pid>,
    last_task: Option<Pid>,
}

impl Scheduler for Fifo {
    fn next(&mut self, _: &crate::engine::ProcessTable) -> Option<Pid> {
        self.last_task = self.tasks.pop_front();
        self.last_task
    }

    fn enqueue(&mut self, proc: &Process) {
        if let Some(last_task) = self.last_task && last_task == proc.id {
            self.tasks.push_front(proc.id);
        } else {
            self.tasks.push_back(proc.id);
        }
    }
}

impl Fifo {
    pub fn new() -> Self {
        Self::default()
    }
}
