use std::{collections::VecDeque, time::Duration};

use crate::{engine::INTERRUPT_INTERVAL, process::Pid, sched::scheduler::Scheduler};

const LEVEL_COUNT: usize = 3;

fn duration_rem(lhs: Duration, rhs: Duration) -> Duration {
    Duration::from_nanos((lhs.as_nanos() % rhs.as_nanos()) as u64)
}

pub struct MultiLevelFeedbackQueue {
    levels: [VecDeque<Pid>; LEVEL_COUNT],
    rr_slice: Duration,
    demote_slice: Duration,

    // last task running
    running_task: Option<(usize, Pid)>,

    s: Duration,
    scheduler_timer: Duration,
}

impl Scheduler for MultiLevelFeedbackQueue {
    fn next(&mut self, _: &crate::engine::ProcessTable) -> Option<Pid> {
        self.scheduler_timer += INTERRUPT_INTERVAL;

        // 5. To avoid task starvation, we put all tasks in the most priority queue every s
        if duration_rem(self.scheduler_timer, self.s) == Duration::ZERO {
            let (sources, destinations) = self.levels.split_at_mut(LEVEL_COUNT - 1);
            let destination = &mut destinations[0];
            for source in sources {
                destination.append(source);
            }
        }

        // 1. if priority(A) > priority(B) -> run A
        // 2. if priority(A) = priority(B) -> RR
        for (level, queue) in self.levels.iter_mut().enumerate().rev() {
            if let Some(selected_task) = queue.pop_front() {
                self.running_task = Some((level, selected_task));
                return self.running_task.map(|(_, pid)| pid);
            }
        }

        None
    }


    fn enqueue(&mut self, proc: &crate::process::Process) {
        if let Some((level, pid)) = self.running_task && pid == proc.id {
            if duration_rem(proc.runtime, self.demote_slice) == Duration::ZERO {
                // 4. if depleted time allotment, demoted to lower queue
                self.levels[level.saturating_sub(1)].push_back(pid)
            } else if duration_rem(proc.runtime - self.offset(level), self.rr_slice) == Duration::ZERO {
                // 2. if priority(A) = priority(B) -> RR
                self.levels[level].push_back(pid);
            } else {
                self.levels[level].push_front(pid);
            }
        } else {
            // 3. if a job cleanly enters the queue, it is placed in the upmost queue
            self.levels[LEVEL_COUNT - 1].push_back(proc.id);
        }
    }
}

impl MultiLevelFeedbackQueue {
    pub fn new(s: Duration, rr_slice: Duration, demote_slice: Duration) -> Self {
        Self {
            levels: Default::default(),
            s,
            rr_slice,
            demote_slice,
            running_task: None,
            scheduler_timer: Duration::ZERO
        }
    }

    fn offset(&self, level: usize) -> Duration {
        let levels_remaining = LEVEL_COUNT - level - 1;
        let slice_ms = self.demote_slice.as_millis() % self.rr_slice.as_millis();

        Duration::from_millis(
            (levels_remaining as u128 * slice_ms)
            .try_into()
            .unwrap(),
        )
    }
}
