use std::time::Duration;

use rand::RngExt;

pub type Pid = u64;
pub type Parent = Option<Pid>;

const CHILD_LIKELIHOOD: i32 = 5;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    #[default]
    Ready,
    Blocked,
    Finished,
}

#[derive(Debug)]
pub enum Syscall {
    Fork,
}

pub enum Trap {
    Syscall { kind: Syscall, elapsed: Duration },
    TimerInterrupt,
}

#[derive(Default)]
pub struct Process {
    pub id: Pid,
    pub state: ProcessState,
    pub parent: Parent,
    pub runtime: Duration,
    pub cpu_remaining: Duration,
}

impl Process {
    pub fn new(id: Pid, state: ProcessState, parent: Parent, cpu_remaining: Duration) -> Self {
        Self {
            id,
            state,
            runtime: Duration::default(),
            parent,
            cpu_remaining,
        }
    }

    pub fn consumed_cpu(&mut self, cpu_time: Duration) {
        self.runtime += cpu_time;
        self.cpu_remaining = self.cpu_remaining.saturating_sub(cpu_time);
    }

    pub fn done(&self) -> bool {
        self.cpu_remaining == Duration::ZERO
    }

    pub fn running(&self) -> bool {
        self.state == ProcessState::Running
    }

    pub fn blocked(&self) -> bool {
        self.state == ProcessState::Blocked
    }

    pub fn response<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Trap {
        if rng.random_range(0..100) < CHILD_LIKELIHOOD {
            Trap::Syscall {
                kind: Syscall::Fork,
                elapsed: Duration::ZERO,
            }
        } else {
            Trap::TimerInterrupt
        }
    }
}
