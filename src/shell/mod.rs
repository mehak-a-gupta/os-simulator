// OS Simulator Interactive Shell
//
// This module provides a command-line interface for interacting with the OS simulator.
// It consists of three main layers:
//
// 1. Command Parsing (parse_command)
//    - Converts user input strings to strongly-typed Command enum
//    - Type-safe argument parsing
//
// 2. Command Execution (Shell::execute)
//    - Dispatches commands to handlers
//    - Returns formatted output strings
//
// 3. System Management (Shell state)
//    - Maintains ProcessManager, MLFQScheduler, SchedulerStats
//    - Tracks overall OS state

use crate::process::{ProcessManager, ProcessState};
use crate::scheduler::MLFQScheduler;

/// Command enum for shell commands
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Create a new process: fork [ppid]
    Fork { ppid: u32 },

    /// List all processes: ps
    Ps,

    /// Run a process: run <pid>
    Run { pid: u32 },

    /// Block a process: block <pid>
    Block { pid: u32 },

    /// Unblock a process: unblock <pid>
    Unblock { pid: u32 },

    /// Kill a process: kill <pid>
    Kill { pid: u32 },

    /// Get process info: info <pid>
    Info { pid: u32 },

    /// Show scheduler queue state: queues
    Queues,

    /// Simulate N scheduling cycles: schedule <n>
    Schedule { cycles: u32 },

    /// Show help: help
    Help,

    /// Exit shell: exit
    Exit,
}

/// Parse command from user input
pub fn parse_command(input: &str) -> Option<Command> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    if parts.is_empty() {
        return None;
    }

    match parts[0] {
        "fork" => {
            if parts.len() >= 2 {
                parts[1].parse::<u32>().ok().map(|ppid| Command::Fork { ppid })
            } else {
                Some(Command::Fork { ppid: 1 }) // Default to init
            }
        }
        "ps" => Some(Command::Ps),
        "run" => {
            parts.get(1)?.parse::<u32>().ok().map(|pid| Command::Run { pid })
        }
        "block" => {
            parts.get(1)?.parse::<u32>().ok().map(|pid| Command::Block { pid })
        }
        "unblock" => {
            parts.get(1)?.parse::<u32>().ok().map(|pid| Command::Unblock { pid })
        }
        "kill" => {
            parts.get(1)?.parse::<u32>().ok().map(|pid| Command::Kill { pid })
        }
        "info" => {
            parts.get(1)?.parse::<u32>().ok().map(|pid| Command::Info { pid })
        }
        "queues" => Some(Command::Queues),
        "schedule" => {
            parts.get(1)?.parse::<u32>().ok().map(|cycles| Command::Schedule { cycles })
        }
        "help" => Some(Command::Help),
        "exit" | "quit" => Some(Command::Exit),
        _ => None,
    }
}

/// OS Shell for interactive process management
pub struct Shell {
    manager: ProcessManager,
    scheduler: MLFQScheduler,
    running: bool,
}

impl Shell {
    /// Create new shell with initialized OS
    pub fn new() -> Self {
        let mut manager = ProcessManager::new();
        let mut scheduler = MLFQScheduler::new();

        // Create init process (PID 1)
        let init_pid = manager.create_process(0);
        scheduler.add_process(init_pid);

        Shell {
            manager,
            scheduler,
            running: true,
        }
    }

    /// Execute a shell command
    pub fn execute(&mut self, cmd: Command) -> String {
        match cmd {
            Command::Fork { ppid } => self.cmd_fork(ppid),
            Command::Ps => self.cmd_ps(),
            Command::Run { pid } => self.cmd_run(pid),
            Command::Block { pid } => self.cmd_block(pid),
            Command::Unblock { pid } => self.cmd_unblock(pid),
            Command::Kill { pid } => self.cmd_kill(pid),
            Command::Info { pid } => self.cmd_info(pid),
            Command::Queues => self.cmd_queues(),
            Command::Schedule { cycles } => self.cmd_schedule(cycles),
            Command::Help => self.cmd_help(),
            Command::Exit => {
                self.running = false;
                "Exiting OS simulator...".to_string()
            }
        }
    }

    /// Fork command: create new child process
    fn cmd_fork(&mut self, ppid: u32) -> String {
        // Verify parent exists
        if self.manager.get_process(ppid).is_none() && ppid != 1 {
            return format!("Error: Parent process {} does not exist", ppid);
        }

        let new_pid = self.manager.create_process(ppid);
        self.scheduler.add_process(new_pid);

        format!("✓ Process created: PID {} (parent: {})", new_pid, ppid)
    }

    /// PS command: list all processes
    fn cmd_ps(&self) -> String {
        let mut output = String::from(
            "PID  PPID STATE       PRIORITY QUEUE TOTAL_TIME\n\
             ─────────────────────────────────────────────────\n"
        );

        for process in self.manager.all_processes() {
            let queue = self.scheduler
                .get_process_queue(process.pid)
                .map_or("N/A".to_string(), |q| format!("Q{}", q));

            output.push_str(&format!(
                "{:<4} {:<4} {:<11?} {:<8} {:<6} {:<10}\n",
                process.pid,
                process.ppid,
                process.state,
                process.priority,
                queue,
                process.total_time
            ));
        }

        output
    }

    /// Run command: transition process to running state
    fn cmd_run(&mut self, pid: u32) -> String {
        match self.manager.get_process_mut(pid) {
            Some(process) => {
                if process.state == ProcessState::Terminated {
                    return format!("Error: Cannot run terminated process {}", pid);
                }
                process.set_state(ProcessState::Running);
                self.manager.set_running_process(pid);
                format!("✓ Process {} is now running", pid)
            }
            None => format!("Error: Process {} not found", pid),
        }
    }

    /// Block command: block a process (e.g., waiting for I/O)
    fn cmd_block(&mut self, pid: u32) -> String {
        match self.manager.get_process_mut(pid) {
            Some(process) => {
                process.set_state(ProcessState::Blocked);
                format!("✓ Process {} blocked (waiting for I/O)", pid)
            }
            None => format!("Error: Process {} not found", pid),
        }
    }

    /// Unblock command: unblock a process
    fn cmd_unblock(&mut self, pid: u32) -> String {
        match self.manager.get_process_mut(pid) {
            Some(process) => {
                if process.state == ProcessState::Blocked {
                    process.set_state(ProcessState::Ready);
                    self.scheduler.process_yielded_early(pid);
                    format!("✓ Process {} unblocked (promoted in scheduler)", pid)
                } else {
                    format!("Error: Process {} is not blocked", pid)
                }
            }
            None => format!("Error: Process {} not found", pid),
        }
    }

    /// Kill command: terminate a process
    fn cmd_kill(&mut self, pid: u32) -> String {
        if pid == 1 {
            return "Error: Cannot kill init process (PID 1)".to_string();
        }

        if self.manager.terminate_process(pid) {
            self.scheduler.remove_process(pid);
            format!("✓ Process {} terminated", pid)
        } else {
            format!("Error: Process {} not found", pid)
        }
    }

    /// Info command: detailed process information
    fn cmd_info(&self, pid: u32) -> String {
        match self.manager.get_process(pid) {
            Some(process) => {
                let queue = self.scheduler
                    .get_process_queue(pid)
                    .map_or("N/A".to_string(), |q| format!("Q{}", q));

                let turnaround = process.turnaround_time();
                let waiting = process.waiting_time();

                format!(
                    "Process Information (PID: {})\n\
                     ────────────────────────────────────\n\
                     Parent PID (PPID):    {}\n\
                     State:                {:?}\n\
                     Priority:             {}\n\
                     Scheduler Queue:      {}\n\
                     Program Counter:      0x{:x}\n\
                     Total Execution Time: {}ms\n\
                     Turnaround Time:      {}ms\n\
                     Waiting Time:         {}ms\n\
                     Stack Pointer:        0x{:x}\n\
                     Heap Start:           0x{:x}\n",
                    process.pid,
                    process.ppid,
                    process.state,
                    process.priority,
                    queue,
                    process.program_counter,
                    process.total_time,
                    turnaround,
                    waiting,
                    process.registers.rsp,
                    process.memory_context.heap_start
                )
            }
            None => format!("Error: Process {} not found", pid),
        }
    }

    /// Queues command: show scheduler queue state
    fn cmd_queues(&self) -> String {
        let lengths = self.scheduler.queue_lengths();
        let current = self.scheduler.current_process();

        let mut output = String::from(
            "MLFQ Scheduler Queue State\n\
             ────────────────────────────────────\n"
        );

        output.push_str(&format!("Q0 (8ms):   {} processes\n", lengths[0]));
        output.push_str(&format!("Q1 (16ms):  {} processes\n", lengths[1]));
        output.push_str(&format!("Q2 (32ms):  {} processes\n", lengths[2]));
        output.push_str(&format!("Q3 (64ms):  {} processes\n", lengths[3]));
        output.push_str(&format!(
            "Currently Running: {}\n",
            current.map_or("None".to_string(), |p| p.to_string())
        ));
        output.push_str(&format!(
            "Time Remaining:   {}ms\n",
            self.scheduler.time_remaining()
        ));

        output
    }

    /// Schedule command: simulate N scheduling cycles
    fn cmd_schedule(&mut self, cycles: u32) -> String {
        let mut output = format!("Simulating {} scheduling cycles:\n\n", cycles);

        for cycle in 1..=cycles {
            if let Some((pid, quantum)) = self.scheduler.next_process() {
                if let Some(process) = self.manager.get_process_mut(pid) {
                    process.set_state(ProcessState::Running);
                    process.total_time = process.total_time.saturating_add(quantum);

                    output.push_str(&format!("Cycle {}: PID {} ran for {}ms\n", cycle, pid, quantum));

                    // Alternate behavior for demo
                    if cycle % 3 == 0 {
                        self.scheduler.process_used_full_quantum(pid);
                        let new_queue = self.scheduler.get_process_queue(pid).unwrap_or(3);
                        output.push_str(&format!("         → Demoted to Q{}\n", new_queue));
                    } else {
                        self.scheduler.process_yielded_early(pid);
                        let new_queue = self.scheduler.get_process_queue(pid).unwrap_or(0);
                        output.push_str(&format!("         → Promoted to Q{}\n", new_queue));
                    }

                    process.set_state(ProcessState::Ready);
                }
            }
        }

        output
    }

    /// Help command: display available commands
    fn cmd_help(&self) -> String {
        String::from(
            "Available Commands:\n\
             ────────────────────────────────────────────────────\n\
             fork [ppid]          - Create new process (child of ppid)\n\
             ps                   - List all processes\n\
             run <pid>            - Transition process to running state\n\
             block <pid>          - Block process (waiting for I/O)\n\
             unblock <pid>        - Unblock process\n\
             kill <pid>           - Terminate process\n\
             info <pid>           - Show detailed process information\n\
             queues               - Show scheduler queue state\n\
             schedule <cycles>    - Simulate N scheduling cycles\n\
             help                 - Show this help message\n\
             exit                 - Exit simulator\n"
        )
    }

    /// Check if shell is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get process count
    pub fn process_count(&self) -> usize {
        self.manager.process_count()
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fork() {
        let cmd = parse_command("fork 1").unwrap();
        assert_eq!(cmd, Command::Fork { ppid: 1 });
    }

    #[test]
    fn test_parse_ps() {
        let cmd = parse_command("ps").unwrap();
        assert_eq!(cmd, Command::Ps);
    }

    #[test]
    fn test_parse_run() {
        let cmd = parse_command("run 2").unwrap();
        assert_eq!(cmd, Command::Run { pid: 2 });
    }

    #[test]
    fn test_parse_kill() {
        let cmd = parse_command("kill 2").unwrap();
        assert_eq!(cmd, Command::Kill { pid: 2 });
    }

    #[test]
    fn test_parse_schedule() {
        let cmd = parse_command("schedule 5").unwrap();
        assert_eq!(cmd, Command::Schedule { cycles: 5 });
    }

    #[test]
    fn test_shell_creation() {
        let shell = Shell::new();
        assert!(shell.is_running());
        assert_eq!(shell.process_count(), 1); // Init process
    }

    #[test]
    fn test_shell_fork_process() {
        let mut shell = Shell::new();
        let result = shell.execute(Command::Fork { ppid: 1 });

        assert!(result.contains("✓"));
        assert_eq!(shell.process_count(), 2);
    }

    #[test]
    fn test_shell_kill_process() {
        let mut shell = Shell::new();
        shell.execute(Command::Fork { ppid: 1 });
        assert_eq!(shell.process_count(), 2);

        let result = shell.execute(Command::Kill { pid: 2 });

        // Kill should succeed
        assert!(result.contains("✓"));

        // Process count stays same (process is terminated, not removed)
        // But we can verify it's in Terminated state via info command
        let info = shell.execute(Command::Info { pid: 2 });
        assert!(info.contains("Terminated"));
    }

    #[test]
    fn test_shell_cannot_kill_init() {
        let mut shell = Shell::new();
        let result = shell.execute(Command::Kill { pid: 1 });

        assert!(result.contains("Error"));
    }

    #[test]
    fn test_shell_run_process() {
        let mut shell = Shell::new();
        shell.execute(Command::Fork { ppid: 1 });
        let result = shell.execute(Command::Run { pid: 2 });

        assert!(result.contains("✓"));
    }

    #[test]
    fn test_shell_block_unblock() {
        let mut shell = Shell::new();
        shell.execute(Command::Fork { ppid: 1 });

        let block_result = shell.execute(Command::Block { pid: 2 });
        assert!(block_result.contains("✓"));

        let unblock_result = shell.execute(Command::Unblock { pid: 2 });
        assert!(unblock_result.contains("✓"));
    }

    #[test]
    fn test_parse_invalid_command() {
        let cmd = parse_command("invalid");
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parse_empty_input() {
        let cmd = parse_command("");
        assert!(cmd.is_none());
    }
}