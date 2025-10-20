// src/scheduler/mod.rs

use crate::process::{Process, ProcessManager, ProcessState};
use std::collections::VecDeque;

/// Multi-Level Feedback Queue (MLFQ) Scheduler
///
/// A sophisticated CPU scheduler that uses multiple priority queues.
/// Processes start at low priority and move up based on behavior.
///
/// Key features:
/// - 4 priority levels (0-3, 0 is highest)
/// - Each level has different time quantum
/// - Processes move down if they use full quantum
/// - Processes move up if they block/yield early
/// - Prevents starvation with periodic boosting

#[derive(Debug, Clone)]
pub struct MLFQScheduler {
    // Four priority queues: Q0 (highest) to Q3 (lowest)
    queues: [VecDeque<u32>; 4],

    // Time quantum for each queue (in ms)
    // Higher priority queues get shorter quantums for responsiveness
    time_quantums: [u32; 4],

    // Track which queue each process is in
    process_queue_map: std::collections::HashMap<u32, usize>,

    // For starvation prevention: boost all to Q0 every N ticks
    boost_interval: u32,
    current_ticks: u32,

    // Current running process
    current_pid: Option<u32>,
    time_remaining: u32,
}

impl MLFQScheduler {
    /// Create a new MLFQ scheduler
    pub fn new() -> Self {
        MLFQScheduler {
            queues: [VecDeque::new(), VecDeque::new(), VecDeque::new(), VecDeque::new()],
            time_quantums: [8, 16, 32, 64],    // ms per queue
            process_queue_map: std::collections::HashMap::new(),
            boost_interval: 100,               // Boost every 100 ticks
            current_ticks: 0,
            current_pid: None,
            time_remaining: 0,
        }
    }

    /// Add a process to the scheduler (starts at lowest priority)
    pub fn add_process(&mut self, pid: u32) {
        // New processes start in Q3 (lowest priority)
        self.queues[3].push_back(pid);
        self.process_queue_map.insert(pid, 3);
    }

    /// Add process to specific queue (mainly for system processes)
    pub fn add_process_to_queue(&mut self, pid: u32, queue: usize) {
        if queue < 4 {
            self.queues[queue].push_back(pid);
            self.process_queue_map.insert(pid, queue);
        }
    }

    /// Remove process from scheduler (when terminated)
    pub fn remove_process(&mut self, pid: u32) {
        if let Some(queue_idx) = self.process_queue_map.remove(&pid) {
            // Find and remove from the queue
            self.queues[queue_idx].retain(|&p| p != pid);
        }
    }

    /// Move process to a different priority queue
    fn move_process_to_queue(&mut self, pid: u32, new_queue: usize) {
        if new_queue < 4 {
            // Remove from current queue
            if let Some(old_queue) = self.process_queue_map.remove(&pid) {
                self.queues[old_queue].retain(|&p| p != pid);
            }
            // Add to new queue
            self.queues[new_queue].push_back(pid);
            self.process_queue_map.insert(pid, new_queue);
        }
    }

    /// Perform priority boost (anti-starvation mechanism)
    /// Move all processes to Q0
    fn priority_boost(&mut self) {
        for queue_idx in 1..4 {
            while let Some(pid) = self.queues[queue_idx].pop_front() {
                self.queues[0].push_back(pid);
                self.process_queue_map.insert(pid, 0);
            }
        }
    }

    /// Get next process to run
    /// Returns (PID, time_quantum_ms)
    pub fn next_process(&mut self) -> Option<(u32, u32)> {
        // Increment ticks first
        self.current_ticks = self.current_ticks.wrapping_add(1);

        // Check if we need to boost all processes (starvation prevention)
        if self.current_ticks > 0 && self.current_ticks % self.boost_interval == 0 {
            self.priority_boost();
        }

        // Find first non-empty queue (highest priority)
        for queue_idx in 0..4 {
            if let Some(pid) = self.queues[queue_idx].pop_front() {
                let quantum = self.time_quantums[queue_idx];
                self.current_pid = Some(pid);
                self.time_remaining = quantum;
                return Some((pid, quantum));
            }
        }

        // No process available
        self.current_pid = None;
        None
    }

    /// Handle process using its full time quantum
    /// (Process was preempted, didn't yield)
    /// -> Move to lower priority (if not already at Q3)
    pub fn process_used_full_quantum(&mut self, pid: u32) {
        if let Some(&current_queue) = self.process_queue_map.get(&pid) {
            if current_queue < 3 {
                // Move down to lower priority
                self.move_process_to_queue(pid, current_queue + 1);
            } else {
                // Already at Q3, just re-queue
                self.queues[3].push_back(pid);
            }
        }
    }

    /// Handle process yielding/blocking early
    /// (Process finished early or blocked on I/O)
    /// -> Move to higher priority (if not already at Q0)
    pub fn process_yielded_early(&mut self, pid: u32) {
        if let Some(&current_queue) = self.process_queue_map.get(&pid) {
            if current_queue > 0 {
                // Move up to higher priority
                self.move_process_to_queue(pid, current_queue - 1);
            } else {
                // Already at Q0, just re-queue
                self.queues[0].push_back(pid);
            }
        }
    }

    /// Simulate process execution for a given number of ticks
    pub fn tick(&mut self, ticks: u32) {
        self.time_remaining = self.time_remaining.saturating_sub(ticks);
    }

    /// Check if current process's quantum has expired
    pub fn is_quantum_expired(&self) -> bool {
        self.time_remaining == 0
    }

    /// Get current running process PID
    pub fn current_process(&self) -> Option<u32> {
        self.current_pid
    }

    /// Get total queue lengths (for diagnostics)
    pub fn queue_lengths(&self) -> [usize; 4] {
        [
            self.queues[0].len(),
            self.queues[1].len(),
            self.queues[2].len(),
            self.queues[3].len(),
        ]
    }

    /// Get which queue a process is in
    pub fn get_process_queue(&self, pid: u32) -> Option<usize> {
        self.process_queue_map.get(&pid).copied()
    }

    /// Get time remaining for current quantum
    pub fn time_remaining(&self) -> u32 {
        self.time_remaining
    }

    /// Reset scheduler (clear all queues)
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

        assert_eq!(scheduler.get_process_queue(1), Some(3)); // Q3 (lowest)
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
        assert_eq!(quantum, 64); // Q3 quantum

        assert_eq!(scheduler.current_process(), Some(1));
    }

    #[test]
    fn test_priority_levels() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process_to_queue(1, 0); // Q0 (highest)
        scheduler.add_process_to_queue(2, 3); // Q3 (lowest)

        let (pid, _) = scheduler.next_process().expect("Should have process");
        assert_eq!(pid, 1); // Q0 process should run first
    }

    #[test]
    fn test_process_used_full_quantum() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process_to_queue(1, 0); // Start at Q0

        scheduler.process_used_full_quantum(1);
        assert_eq!(scheduler.get_process_queue(1), Some(1)); // Moved to Q1

        scheduler.process_used_full_quantum(1);
        assert_eq!(scheduler.get_process_queue(1), Some(2)); // Moved to Q2
    }

    #[test]
    fn test_process_yielded_early() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process_to_queue(1, 3); // Start at Q3

        scheduler.process_yielded_early(1);
        assert_eq!(scheduler.get_process_queue(1), Some(2)); // Moved to Q2

        scheduler.process_yielded_early(1);
        assert_eq!(scheduler.get_process_queue(1), Some(1)); // Moved to Q1
    }

    #[test]
    fn test_quantum_expiration() {
        let mut scheduler = MLFQScheduler::new();
        scheduler.add_process(1);

        scheduler.next_process();
        assert!(!scheduler.is_quantum_expired());

        scheduler.tick(64); // Quantum is 64ms for Q3
        assert!(scheduler.is_quantum_expired());
    }

    #[test]
    fn test_priority_boost_prevents_starvation() {
        let mut scheduler = MLFQScheduler::new();

        // Add processes to different queues
        scheduler.add_process_to_queue(1, 3); // Low priority
        scheduler.add_process_to_queue(2, 3); // Low priority
        scheduler.add_process_to_queue(3, 0); // High priority

        let original_q1 = scheduler.get_process_queue(1);
        assert_eq!(original_q1, Some(3));

        // Manually set ticks to 99, then call next_process once more
        // This should trigger the boost (at tick 100)
        scheduler.current_ticks = 99;

        // Add a dummy process that will be selected
        scheduler.add_process_to_queue(4, 0);
        let _ = scheduler.next_process(); // This increments to tick 100 and triggers boost

        // After boost at tick 100, low priority process should be back in Q0
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