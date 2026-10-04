use std::{collections::BTreeMap, time::Duration};

use rand::RngExt;
use tracing::{debug, info, instrument, trace, warn};

use crate::{
    process::{Pid, Process, ProcessState, Syscall, Trap},
    sched::scheduler::Scheduler,
};

const INTERRUPT_INTERVAL: Duration = Duration::from_millis(10);

pub type ProcessTable = BTreeMap<Pid, Process>;

pub struct Engine<S: Scheduler> {
    scheduler: S,

    process_table: ProcessTable,
    current_process: Option<Pid>,

    interrupt_interval: Duration,

    rng: Box<dyn rand::Rng>,
}

impl<S: Scheduler> Engine<S> {
    pub fn new(scheduler: S) -> Self {
        Self {
            scheduler,
            process_table: BTreeMap::new(),
            current_process: None,
            interrupt_interval: INTERRUPT_INTERVAL,
            rng: Box::new(rand::rng()),
        }
    }

    fn dispatch(&mut self) -> Option<Pid> {
        let pid = self.scheduler.next(&self.process_table)?;
        let process = self
            .process_table
            .get_mut(&pid)
            .expect("scheduler returned invalid PID, couldn't find in Process Table");

        process.state = ProcessState::Running;
        self.current_process = Some(pid);
        self.current_process
    }

    #[instrument(
        name = "interrupt",
        skip(self),
        fields(
            current_pid = ?self.current_process,
            process_count = self.process_table.len(),
        )
    )]
    pub fn run_tick(&mut self) -> Option<Pid> {
        if self.current_process.is_none() {
            return self.dispatch();
        }

        let current_pid = self
            .current_process
            .expect("called trap on non-existent process");

        trace!(pid = current_pid, "executing process");

        let process = self
            .process_table
            .get_mut(&current_pid)
            .expect("current process missing from process table");

        let trap_response = process.response(&mut self.rng);

        let elapsed = match &trap_response {
            Trap::Syscall { kind, elapsed } => {
                debug!(
                    pid = current_pid,
                    syscall = ?kind,
                    elapsed_ms = elapsed.as_millis(),
                    "syscall trap"
                );

                *elapsed
            }

            Trap::TimerInterrupt => {
                debug!(
                    pid = current_pid,
                    interval_ms = self.interrupt_interval.as_millis(),
                    "timer interrupt"
                );

                self.interrupt_interval
            }
        };

        process.consumed_cpu(elapsed);

        trace!(
            pid = current_pid,
            elapsed_ms = elapsed.as_millis(),
            "CPU time consumed"
        );

        process.state = if process.done() {
            ProcessState::Finished
        } else {
            ProcessState::Ready
        };

        debug!(
            pid = current_pid,
            state = ?process.state,
            "process state transition"
        );

        match process.state {
            ProcessState::Running => unreachable!(),

            ProcessState::Ready => {
                trace!(pid = process.id, "returning process to scheduler");

                self.scheduler.enqueue(process);
            }

            ProcessState::Finished => {
                info!(pid = current_pid, "process finished");

                self.process_table.remove_entry(&current_pid);
            }

            ProcessState::Blocked => {
                debug!(pid = current_pid, "process blocked");
            }
        }

        if let Trap::Syscall {
            kind: Syscall::Fork,
            ..
        } = trap_response
        {
            info!(parent_pid = current_pid, "fork requested");

            self.create(Some(current_pid));
        }

        self.dispatch()
    }

    #[instrument(skip(self), fields(?parent))]
    pub fn create(&mut self, parent: Option<Pid>) {
        let new_pid = self
            .process_table
            .last_key_value()
            .map(|(&key, _)| key + 1)
            .unwrap_or(0);

        let cpu_time = self.rng.random_range(0..10) * INTERRUPT_INTERVAL;

        let process = Process::new(new_pid, ProcessState::Ready, parent, cpu_time);

        self.scheduler.enqueue(&process);
        self.process_table.insert(new_pid, process);

        info!(
            pid = new_pid,
            ?parent,
            cpu_time_ms = cpu_time.as_millis(),
            "process created"
        );
    }

    #[instrument(skip(self))]
    pub fn destroy(&mut self, pid: Pid) {
        match self.process_table.remove_entry(&pid) {
            Some(_) => {
                info!(pid, "process destroyed");
            }

            None => {
                warn!(pid, "attempted to destroy nonexistent process");
            }
        }
    }
}
