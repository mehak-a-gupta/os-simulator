// src/scheduler/mod.rs - Restructured with Metrics and Test Suite

pub mod metrics;
pub mod test_suite;

pub use metrics::{SchedulerStats, ProcessMetrics};
pub use test_suite::TestResults;

use std::collections::VecDeque;

/// Multi-Level Feedback Queue (MLFQ) Scheduler
///
/// A sophisticated CPU scheduler that uses multiple priority queues.
/// Processes start at low priority and move up based on behavior.

#[derive(Debug, Clone)]
pub struct MLFQScheduler {
    queues: [VecDeque<u32>; 4],
    time_quantums: [u32; 4],
    process_queue_map: std::collections::HashMap<u32, usize>,
    boost_interval: u32,
    current_ticks: u32,
    current_pid: Option<u32>,
    time_remaining: u32,
}

impl MLFQScheduler {
    pub fn new() -> Self {
        MLFQScheduler {
            queues: [VecDeque::new(), VecDeque::new(), VecDeque::new(), VecDeque::new()],
            time_quantums: [8, 16, 32, 64],
            process_queue_map: std::collections::HashMap::new(),
            boost_interval: 100,
            current_ticks: 0,
            current_pid: None,
            time_remaining: 0,
        }
    }

    pub fn add_process(&mut self, pid: u32) {
        self.queues[3].push_back(pid);
        self.process_queue_map.insert(pid, 3);
    }

    pub fn add_process_to_queue(&mut self, pid: u32, queue: usize) {
        if queue < 4 {
            self.queues[queue].push_back(pid);
            self.process_queue_map.insert(pid, queue);
        }
    }

    pub fn remove_process(&mut self, pid: u32) {
        if let Some(queue_idx) = self.process_queue_map.remove(&pid) {
            self.queues[queue_idx].retain(|&p| p != pid);
        }
    }

    fn move_process_to_queue(&mut self, pid: u32, new_queue: usize) {
        if new_queue < 4 {
            if let Some(old_queue) = self.process_queue_map.remove(&pid) {
                self.queues[old_queue].retain(|&p| p != pid);
            }
            self.queues[new_queue].push_back(pid);
            self.process_queue_map.insert(pid, new_queue);
        }
    }

    fn priority_boost(&mut self) {
        for queue_idx in 1..4 {
            while let Some(pid) = self.queues[queue_idx].pop_front() {
                self.queues[0].push_back(pid);
                self.process_queue_map.insert(pid, 0);
            }
        }
    }

    pub fn next_process(&mut self) -> Option<(u32, u32)> {
        self.current_ticks = self.current_ticks.wrapping_add(1);

        if self.current_ticks > 0 && self.current_ticks % self.boost_interval == 0 {
            self.priority_boost();
        }

        for queue_idx in 0..4 {
            if let Some(pid) = self.queues[queue_idx].pop_front() {
                let quantum = self.time_quantums[queue_idx];
                self.current_pid = Some(pid);
                self.time_remaining = quantum;
                return Some((pid, quantum));
            }
        }

        self.current_pid = None;
        None
    }

    pub fn process_used_full_quantum(&mut self, pid: u32) {
        if let Some(&current_queue) = self.process_queue_map.get(&pid) {
            if current_queue < 3 {
                self.move_process_to_queue(pid, current_queue + 1);
            } else {
                self.queues[3].push_back(pid);
            }
        }
    }

    pub fn process_yielded_early(&mut self, pid: u32) {
        if let Some(&current_queue) = self.process_queue_map.get(&pid) {
            if current_queue > 0 {
                self.move_process_to_queue(pid, current_queue - 1);
            } else {
                self.queues[0].push_back(pid);
            }
        }
    }

    pub fn tick(&mut self, ticks: u32) {
        self.time_remaining = self.time_remaining.saturating_sub(ticks);
    }

    pub fn is_quantum_expired(&self) -> bool {
        self.time_remaining == 0
    }

    pub fn current_process(&self) -> Option<u32> {
        self.current_pid
    }

    pub fn queue_lengths(&self) -> [usize; 4] {
        [
            self.queues[0].len(),
            self.queues[1].len(),
            self.queues[2].len(),
            self.queues[3].len(),
        ]
    }

    pub fn get_process_queue(&self, pid: u32) -> Option<usize> {
        self.process_queue_map.get(&pid).copied()
    }

    pub fn time_remaining(&self) -> u32 {
        self.time_remaining
    }

    pub fn reset(&mut self) {
        for queue in &mut self.queues {
            queue.clear();
        }
        self.process_queue_map.clear();
        self.current_pid = None;
        self.time_remaining = 0;
        self.current_ticks = 0;
    }
}

impl Default for MLFQScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = MLFQScheduler::new();
        assert_eq!(scheduler.time_quantums, [8, 16, 32, 64]);
        assert_eq!(scheduler.queue_lengths(), [0, 0, 0, 0]);
    }

    #[test]
    fn test_add_process() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process(1);
        scheduler.add_process(2);

        assert_eq!(scheduler.get_process_queue(1), Some(3));
        assert_eq!(scheduler.get_process_queue(2), Some(3));
        assert_eq!(scheduler.queue_lengths(), [0, 0, 0, 2]);
    }

    #[test]
    fn test_next_process() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process(1);
        scheduler.add_process(2);

        let (pid, quantum) = scheduler.next_process().expect("Should have process");
        assert_eq!(pid, 1);
        assert_eq!(quantum, 64);

        assert_eq!(scheduler.current_process(), Some(1));
    }

    #[test]
    fn test_priority_levels() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process_to_queue(1, 0);
        scheduler.add_process_to_queue(2, 3);

        let (pid, _) = scheduler.next_process().expect("Should have process");
        assert_eq!(pid, 1);
    }

    #[test]
    fn test_process_used_full_quantum() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process_to_queue(1, 0);

        scheduler.process_used_full_quantum(1);
        assert_eq!(scheduler.get_process_queue(1), Some(1));

        scheduler.process_used_full_quantum(1);
        assert_eq!(scheduler.get_process_queue(1), Some(2));
    }

    #[test]
    fn test_process_yielded_early() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process_to_queue(1, 3);

        scheduler.process_yielded_early(1);
        assert_eq!(scheduler.get_process_queue(1), Some(2));

        scheduler.process_yielded_early(1);
        assert_eq!(scheduler.get_process_queue(1), Some(1));
    }

    #[test]
    fn test_quantum_expiration() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process(1);

        scheduler.next_process();
        assert!(!scheduler.is_quantum_expired());

        scheduler.tick(64);
        assert!(scheduler.is_quantum_expired());
    }

    #[test]
    fn test_priority_boost_prevents_starvation() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process_to_queue(1, 3);
        scheduler.add_process_to_queue(2, 3);
        scheduler.add_process_to_queue(3, 0);

        let original_q1 = scheduler.get_process_queue(1);
        assert_eq!(original_q1, Some(3));

        scheduler.current_ticks = 99;
        scheduler.add_process_to_queue(4, 0);
        let _ = scheduler.next_process();

        let queue_1_after = scheduler.get_process_queue(1);
        assert_eq!(queue_1_after, Some(0), "Process 1 should be boosted to Q0");
    }

    #[test]
    fn test_remove_process() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process(1);
        scheduler.add_process(2);

        assert_eq!(scheduler.queue_lengths(), [0, 0, 0, 2]);

        scheduler.remove_process(1);
        assert_eq!(scheduler.queue_lengths(), [0, 0, 0, 1]);
        assert_eq!(scheduler.get_process_queue(1), None);
    }

    #[test]
    fn test_multiple_processes_fifo_order() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process(1);
        scheduler.add_process(2);
        scheduler.add_process(3);

        let (pid1, _) = scheduler.next_process().unwrap();
        let (pid2, _) = scheduler.next_process().unwrap();
        let (pid3, _) = scheduler.next_process().unwrap();

        assert_eq!(pid1, 1);
        assert_eq!(pid2, 2);
        assert_eq!(pid3, 3);
    }

    #[test]
    fn test_scheduler_reset() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process(1);
        scheduler.add_process(2);

        scheduler.reset();
        assert_eq!(scheduler.queue_lengths(), [0, 0, 0, 0]);
        assert_eq!(scheduler.next_process(), None);
    }
}