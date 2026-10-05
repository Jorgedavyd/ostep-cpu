use std::{
    collections::{BTreeMap, VecDeque},
    time::Duration,
};

use crate::{
    process::{Pid},
    sched::scheduler::Scheduler,
};

#[derive(Default)]
pub struct ShortestTimeCompletionFirst {
    tasks: BTreeMap<Duration, VecDeque<Pid>>,
    last_task: Option<Pid>
}

impl Scheduler for ShortestTimeCompletionFirst {
    fn next(&mut self, _proc_tbl: &crate::engine::ProcessTable) -> Option<crate::process::Pid> {
        if let Some(mut entry) = self.tasks.first_entry() {
            let task_queue = entry.get_mut();
            self.last_task = task_queue.pop_front();
            if task_queue.is_empty() { entry.remove_entry(); }
            self.last_task
        } else {
            None
        }
    }

    fn enqueue(&mut self, proc: &crate::process::Process) {
        let tasks = self.tasks.entry(proc.cpu_remaining).or_default();
        if let Some(pid) = self.last_task && pid == proc.id {
            tasks.push_front(proc.id);
        } else {
            tasks.push_back(proc.id);
        }
    }
}


impl ShortestTimeCompletionFirst {
    pub fn new() -> Self {
        Self::default()
    }
}
