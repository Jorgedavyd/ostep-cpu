use std::{collections::BTreeMap, time::Duration};
use rand::RngExt;

use crate::{
    process::{Pid, Process, ProcessState, Syscall, Trap},
    sched::scheduler::Scheduler,
};

const INTERRUPT_INTERVAL: Duration = Duration::from_millis(10);

pub type ProcessTable = BTreeMap<Pid, Process>;

pub struct Engine<S: Scheduler> {
    // scheduler
    scheduler: S,

    // process state
    process_table: BTreeMap<Pid, Process>,
    current_process: Option<Pid>,

    // timer interrupt
    interrupt_interval: Duration,

    // randomness
    rng: Box<dyn rand::Rng>
}

impl<S: Scheduler> Engine<S> {
    pub fn new(scheduler: S) -> Self {
        Self {
            scheduler,
            process_table: BTreeMap::new(),
            current_process: None,
            interrupt_interval: INTERRUPT_INTERVAL,
            rng: Box::new(rand::rng())
        }
    }

    pub fn run_tick(&mut self) {
        let Some(current_process) = self.current_process else {
            return;
        };

        let current_process_id = current_process;
        let current_process = self.process_table.get_mut(&current_process).unwrap();

        // simulate response from the last task
        let trap = current_process.response(&mut self.rng);

        // update current process state
        current_process.consumed_cpu(match trap {
            Trap::Syscall { elapsed, .. } => elapsed,
            Trap::TimerInterrupt => self.interrupt_interval,
        });

        current_process.state = if current_process.done() {
            ProcessState::Finished
        } else {
            ProcessState::Ready
        };

        // enqueue it or let it die
        match current_process.state {
            ProcessState::Running => unreachable!(),
            ProcessState::Ready => self.scheduler.enqueue(current_process.id),
            ProcessState::Finished => {
                self.process_table.remove_entry(&current_process_id);
            }
            ProcessState::Blocked => (),
        }

        // Syscalls only fork
        if let Trap::Syscall { kind, .. } = trap && matches!(kind, Syscall::Fork) {
            self.create(Some(current_process_id));
        }


        // select the next current task
        self.current_process = self.scheduler.next(&self.process_table);

        if let Some(current_process) = self.current_process {
            let process = self.process_table.get_mut(&current_process).unwrap();
            process.state = ProcessState::Running;
        }
    }

    pub fn create(&mut self, parent: Option<Pid>) {
        let new_pid = self.process_table.last_key_value().map(|(&key, _)| key + 1).unwrap_or(0);
        self.process_table.insert(new_pid, Process::new(new_pid, ProcessState::Ready, parent, self.rng.random_range(0..10) * INTERRUPT_INTERVAL));
        self.scheduler.enqueue(new_pid);
    }

    pub fn destroy(&mut self, pid: Pid) {
        self.process_table.remove_entry(&pid);
    }
}
