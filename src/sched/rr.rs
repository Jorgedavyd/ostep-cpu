use std::collections::VecDeque;

use crate::{engine::ProcessTable, process::Pid, sched::scheduler::Scheduler};

pub struct RoundRobin {
    tasks: VecDeque<Pid>,

    // Round robin configuration
    time_slice: u32,
    interrupt: u32,

    // last task state tracking
    last_task: Option<Pid>,
    cum_time: u32,
}

impl Scheduler for RoundRobin {
    fn next(&mut self, _: &ProcessTable) -> Option<Pid> {
        self.last_task = self.tasks.pop_front();
        self.last_task
    }

    fn enqueue(&mut self, pid: Pid) {
        self.cum_time += self.interrupt;
        if let Some(last_task_pid) = self.last_task && pid == last_task_pid {
            if self.cum_time == self.time_slice {
                self.last_task = None;
                self.cum_time = 0;
                self.tasks.push_back(pid);
            } else {
                self.tasks.push_front(pid);
            }
        } else {
            self.tasks.push_back(pid);
        }
    }
}

impl RoundRobin {
    pub fn new(interrupt: u32, slice_mult: u32) -> Self {
        Self {
            tasks: VecDeque::new(),
            time_slice: interrupt * slice_mult,
            interrupt,
            last_task: None,
            cum_time: 0,
        }
    }
}
