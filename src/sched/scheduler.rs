use crate::{
    engine::ProcessTable,
    process::{Pid, Process},
};

pub trait Scheduler {
    fn next(&mut self, proc_tbl: &ProcessTable) -> Option<Pid>;
    fn enqueue(&mut self, proc: &Process);
}
