// src/process/mod.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Process state enum representing the different states a process can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// Simulated CPU registers
#[derive(Debug, Clone)]
pub struct Registers {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0x1000, // Stack pointer starts at high address
        }
    }
}

/// Memory context for a process
#[derive(Debug, Clone)]
pub struct MemoryContext {
    pub page_table_base: u64,
    pub heap_start: u64,
    pub heap_size: usize,
    pub stack_start: u64,
    pub stack_size: usize,
}

impl Default for MemoryContext {
    fn default() -> Self {
        MemoryContext {
            page_table_base: 0,
            heap_start: 0x2000,
            heap_size: 0x1000,
            stack_start: 0x10000,
            stack_size: 0x2000,
        }
    }
}

/// Process Control Block (PCB)
#[derive(Debug, Clone)]
pub struct Process {
    pub pid: u32,
    pub ppid: u32, // Parent PID
    pub state: ProcessState,
    pub priority: u8, // 0-3, where 0 is highest priority
    pub program_counter: u64,
    pub registers: Registers,
    pub memory_context: MemoryContext,
    pub time_allocated: u32, // Time allocated to this quantum (ms)
    pub time_used: u32, // Time used in current quantum (ms)
    pub total_time: u32, // Total execution time (ms)
    pub creation_time: DateTime<Utc>,
    pub termination_time: Option<DateTime<Utc>>,
    pub queue_entry_time: DateTime<Utc>,
}

impl Process {
    /// Create a new process with given PID and parent PID
    pub fn new(pid: u32, ppid: u32) -> Self {
        let now = Utc::now();
        Process {
            pid,
            ppid,
            state: ProcessState::Ready,
            priority: 3, // Start at lowest priority
            program_counter: 0,
            registers: Registers::default(),
            memory_context: MemoryContext::default(),
            time_allocated: 0,
            time_used: 0,
            total_time: 0,
            creation_time: now,
            termination_time: None,
            queue_entry_time: now,
        }
    }

    /// Transition process to a new state
    pub fn set_state(&mut self, new_state: ProcessState) {
        self.state = new_state;
        if new_state == ProcessState::Terminated {
            self.termination_time = Some(Utc::now());
        }
    }

    /// Get the turnaround time (total time from creation to termination)
    pub fn turnaround_time(&self) -> u64 {
        match self.termination_time {
            Some(term_time) => {
                (term_time.timestamp_millis() - self.creation_time.timestamp_millis()) as u64
            }
            None => (Utc::now().timestamp_millis() - self.creation_time.timestamp_millis()) as u64,
        }
    }

    /// Get the response time (time until first execution)
    pub fn response_time(&self) -> Option<u64> {
        if self.total_time > 0 {
            Some((self.queue_entry_time.timestamp_millis() - self.creation_time.timestamp_millis()) as u64)
        } else {
            None
        }
    }

    /// Get waiting time (turnaround time - total execution time)
    pub fn waiting_time(&self) -> u64 {
        self.turnaround_time().saturating_sub(self.total_time as u64)
    }

    /// Check if process has used its time quantum
    pub fn quantum_expired(&self) -> bool {
        self.time_used >= self.time_allocated
    }

    /// Reset time counters for new quantum
    pub fn reset_quantum(&mut self) {
        self.time_used = 0;
    }
}

/// Process Manager for managing all processes
pub struct ProcessManager {
    processes: HashMap<u32, Process>,
    next_pid: u32,
    current_process_id: Option<u32>,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new() -> Self {
        ProcessManager {
            processes: HashMap::new(),
            next_pid: 1,
            current_process_id: None,
        }
    }

    /// Create a new process
    pub fn create_process(&mut self, ppid: u32) -> u32 {
        let pid = self.next_pid;
        self.next_pid += 1;
        let process = Process::new(pid, ppid);
        self.processes.insert(pid, process);
        pid
    }

    /// Get a process by PID
    pub fn get_process(&self, pid: u32) -> Option<&Process> {
        self.processes.get(&pid)
    }

    /// Get a mutable reference to a process
    pub fn get_process_mut(&mut self, pid: u32) -> Option<&mut Process> {
        self.processes.get_mut(&pid)
    }

    /// Terminate a process
    pub fn terminate_process(&mut self, pid: u32) -> bool {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.set_state(ProcessState::Terminated);
            return true;
        }
        false
    }

    /// Get all processes
    pub fn all_processes(&self) -> Vec<&Process> {
        self.processes.values().collect()
    }

    /// Get all active (non-terminated) processes
    pub fn active_processes(&self) -> Vec<&Process> {
        self.processes
            .values()
            .filter(|p| p.state != ProcessState::Terminated)
            .collect()
    }

    /// Set the currently running process
    pub fn set_running_process(&mut self, pid: u32) {
        self.current_process_id = Some(pid);
        if let Some(process) = self.processes.get_mut(&pid) {
            process.set_state(ProcessState::Running);
        }
    }

    /// Get the currently running process
    pub fn get_running_process(&self) -> Option<&Process> {
        self.current_process_id
            .and_then(|pid| self.processes.get(&pid))
    }

    /// Get mutable reference to currently running process
    pub fn get_running_process_mut(&mut self) -> Option<&mut Process> {
        let pid = self.current_process_id?;
        self.processes.get_mut(&pid)
    }

    /// Clear current process
    pub fn clear_running_process(&mut self) {
        self.current_process_id = None;
    }

    /// Get process count
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_creation() {
        let process = Process::new(1, 0);
        assert_eq!(process.pid, 1);
        assert_eq!(process.ppid, 0);
        assert_eq!(process.state, ProcessState::Ready);
        assert_eq!(process.priority, 3);
    }

    #[test]
    fn test_process_state_transition() {
        let mut process = Process::new(1, 0);
        assert_eq!(process.state, ProcessState::Ready);

        process.set_state(ProcessState::Running);
        assert_eq!(process.state, ProcessState::Running);

        process.set_state(ProcessState::Terminated);
        assert_eq!(process.state, ProcessState::Terminated);
        assert!(process.termination_time.is_some());
    }

    #[test]
    fn test_process_manager() {
        let mut manager = ProcessManager::new();

        let pid1 = manager.create_process(0);
        let pid2 = manager.create_process(0);

        assert_eq!(pid1, 1);
        assert_eq!(pid2, 2);
        assert_eq!(manager.process_count(), 2);
    }

    #[test]
    fn test_process_quantum_tracking() {
        let mut process = Process::new(1, 0);
        process.time_allocated = 8;
        process.time_used = 5;

        assert!(!process.quantum_expired());
        process.time_used = 8;
        assert!(process.quantum_expired());
    }

    #[test]
    fn test_process_metrics() {
        let process = Process::new(1, 0);

        // Just verify turnaround_time method doesn't panic and returns a value
        let turnaround = process.turnaround_time();
        assert!(turnaround >= 0); // Should always be non-negative
    }

    #[test]
    fn test_process_manager_operations() {
        let mut manager = ProcessManager::new();

        let pid = manager.create_process(0);
        manager.set_running_process(pid);

        let running = manager.get_running_process();
        assert!(running.is_some());
        assert_eq!(running.unwrap().pid, pid);
        assert_eq!(running.unwrap().state, ProcessState::Running);
    }
}