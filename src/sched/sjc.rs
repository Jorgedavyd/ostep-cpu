use std::{
    collections::{BTreeMap, VecDeque},
    time::Duration,
};

use crate::{process::Pid, sched::scheduler::Scheduler};

#[derive(Default)]
pub struct ShortestJobCompletion {
    tasks: BTreeMap<Duration, VecDeque<Pid>>,
    last_task: Option<Pid>,
}

impl Scheduler for ShortestJobCompletion {
    fn next(&mut self, _: &crate::engine::ProcessTable) -> Option<crate::process::Pid> {
        if self.last_task.is_some() { return self.last_task; }

        self.last_task = None;

        while let Some(mut entry) = self.tasks.first_entry() {
            let task_queue = entry.get_mut();
            let pid = task_queue.pop_front();
            if task_queue.is_empty() { entry.remove_entry(); }
            if pid.is_some() { return pid; }
        }

        None
    }

    fn enqueue(&mut self, proc: &crate::process::Process) {
        if self.last_task == Some(proc.id) { return; }

        let tasks = self.tasks.entry(proc.cpu_remaining).or_default();
        tasks.push_back(proc.id);
    }
}

impl ShortestJobCompletion {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::ProcessTable,
        process::{Process, ProcessState},
    };

    #[test]
    fn completed_task_is_not_returned_from_queue() {
        let mut scheduler = ShortestJobCompletion::new();
        let mut process = Process::new(1, ProcessState::Ready, None, Duration::from_millis(20));
        let mut process_table = ProcessTable::new();

        scheduler.enqueue(&process);
        process_table.insert(process.id, process);
        assert_eq!(scheduler.next(&process_table), Some(1));

        process = process_table.remove(&1).unwrap();
        process.consumed_cpu(Duration::from_millis(10));
        scheduler.enqueue(&process);
        process_table.insert(process.id, process);
        assert_eq!(scheduler.next(&process_table), Some(1));

        process_table.remove(&1);
        assert_eq!(scheduler.next(&process_table), None);
    }
}
