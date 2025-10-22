// src/shell/mod.rs

use crate::process::{ProcessManager, ProcessState};
use crate::scheduler::MLFQScheduler;

/// Command enum for shell commands
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    // Process Management
    Fork { ppid: u32 },
    Ps,
    Run { pid: u32 },
    Block { pid: u32 },
    Unblock { pid: u32 },
    Kill { pid: u32 },
    Info { pid: u32 },

    // Scheduler Operations
    Queues,
    Schedule { cycles: u32 },

    // Scheduler Control
    Nice { pid: u32, priority: u8 },
    SchedStats,

    // Programs
    Programs,
    RunProgram { program_name: String },

    // Statistics
    Stats,
    Metrics { pid: u32 },
    ResetStats,

    // System
    Help,
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
                Some(Command::Fork { ppid: 1 })
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
        "nice" => {
            let pid = parts.get(1)?.parse::<u32>().ok()?;
            let priority = parts.get(2)?.parse::<u8>().ok()?;
            Some(Command::Nice { pid, priority })
        }
        "sched_stats" => Some(Command::SchedStats),
        "programs" => Some(Command::Programs),
        "run_program" => {
            parts.get(1).map(|s| Command::RunProgram { program_name: s.to_string() })
        }
        "stats" => Some(Command::Stats),
        "metrics" => {
            parts.get(1)?.parse::<u32>().ok().map(|pid| Command::Metrics { pid })
        }
        "reset_stats" => Some(Command::ResetStats),
        "help" => Some(Command::Help),
        "exit" | "quit" => Some(Command::Exit),
        _ => None,
    }
}

/// OS Shell
pub struct Shell {
    manager: ProcessManager,
    scheduler: MLFQScheduler,
    stats: crate::scheduler::metrics::SchedulerStats,
    running: bool,
}

impl Shell {
    pub fn new() -> Self {
        let mut manager = ProcessManager::new();
        let mut scheduler = MLFQScheduler::new();
        let mut stats = crate::scheduler::metrics::SchedulerStats::new();

        let init_pid = manager.create_process(0);
        scheduler.add_process(init_pid);
        stats.record_process_created(init_pid);

        Shell {
            manager,
            scheduler,
            stats,
            running: true,
        }
    }

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
            Command::Nice { pid, priority } => self.cmd_nice(pid, priority),
            Command::SchedStats => self.cmd_sched_stats(),
            Command::Programs => self.cmd_programs(),
            Command::RunProgram { program_name } => self.cmd_run_program(&program_name),
            Command::Stats => self.cmd_stats(),
            Command::Metrics { pid } => self.cmd_metrics(pid),
            Command::ResetStats => self.cmd_reset_stats(),
            Command::Help => self.cmd_help(),
            Command::Exit => {
                self.running = false;
                "Exiting OS simulator...".to_string()
            }
        }
    }

    // ========================================================================
    // PROCESS MANAGEMENT COMMANDS
    // ========================================================================

    fn cmd_fork(&mut self, ppid: u32) -> String {
        if self.manager.get_process(ppid).is_none() && ppid != 1 {
            return format!("Error: Parent process {} does not exist", ppid);
        }

        let new_pid = self.manager.create_process(ppid);
        self.scheduler.add_process(new_pid);
        self.stats.record_process_created(new_pid);

        format!("✓ Process created: PID {} (parent: {})", new_pid, ppid)
    }

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

    fn cmd_run(&mut self, pid: u32) -> String {
        match self.manager.get_process_mut(pid) {
            Some(process) => {
                if process.state == ProcessState::Terminated {
                    return format!("Error: Cannot run terminated process {}", pid);
                }
                process.set_state(ProcessState::Running);
                self.manager.set_running_process(pid);
                self.stats.record_context_switch(pid);
                format!("✓ Process {} is now running", pid)
            }
            None => format!("Error: Process {} not found", pid),
        }
    }

    fn cmd_block(&mut self, pid: u32) -> String {
        match self.manager.get_process_mut(pid) {
            Some(process) => {
                process.set_state(ProcessState::Blocked);
                format!("✓ Process {} blocked (waiting for I/O)", pid)
            }
            None => format!("Error: Process {} not found", pid),
        }
    }

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

    fn cmd_kill(&mut self, pid: u32) -> String {
        if pid == 1 {
            return "Error: Cannot kill init process (PID 1)".to_string();
        }

        if let Some(process) = self.manager.get_process(pid) {
            let turnaround = process.turnaround_time();
            let response = process.response_time().unwrap_or(0);
            let execution = process.total_time as u64;

            self.stats.record_execution_time(pid, execution);
            self.stats.record_process_terminated(pid, turnaround, response);
        }

        if self.manager.terminate_process(pid) {
            self.scheduler.remove_process(pid);
            format!("✓ Process {} terminated", pid)
        } else {
            format!("Error: Process {} not found", pid)
        }
    }

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

    // ========================================================================
    // SCHEDULER COMMANDS
    // ========================================================================

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

    fn cmd_schedule(&mut self, cycles: u32) -> String {
        let mut output = format!("Simulating {} scheduling cycles:\n\n", cycles);

        for cycle in 1..=cycles {
            if let Some((pid, quantum)) = self.scheduler.next_process() {
                if let Some(process) = self.manager.get_process_mut(pid) {
                    process.set_state(ProcessState::Running);
                    process.total_time = process.total_time.saturating_add(quantum);

                    self.stats.record_context_switch(pid);
                    self.stats.record_execution_time(pid, quantum as u64);
                    self.stats.record_tick();

                    output.push_str(&format!("Cycle {}: PID {} ran for {}ms in Q{}\n",
                                             cycle,
                                             pid,
                                             quantum,
                                             self.scheduler.get_process_queue(pid).unwrap_or(3)
                    ));

                    let use_full_quantum = rand::random::<f32>() < 0.7;

                    if use_full_quantum {
                        self.scheduler.process_used_full_quantum(pid);
                        self.stats.record_queue_change(pid);
                        let new_queue = self.scheduler.get_process_queue(pid).unwrap_or(3);
                        output.push_str(&format!("         • Used full quantum → Demoted to Q{}\n", new_queue));
                    } else {
                        self.scheduler.process_yielded_early(pid);
                        self.stats.record_queue_change(pid);
                        let new_queue = self.scheduler.get_process_queue(pid).unwrap_or(0);
                        output.push_str(&format!("         • Yielded early → Promoted to Q{}\n", new_queue));
                    }

                    process.set_state(ProcessState::Ready);
                }
            }
        }

        output
    }

    // ========================================================================
    // SCHEDULER CONTROL COMMANDS
    // ========================================================================

    fn cmd_nice(&mut self, pid: u32, priority: u8) -> String {
        if priority > 3 {
            return "Error: Priority must be 0-3 (0=highest, 3=lowest)".to_string();
        }

        match self.manager.get_process_mut(pid) {
            Some(process) => {
                let old_priority = process.priority;
                process.priority = priority;

                if let Some(_old_queue) = self.scheduler.get_process_queue(pid) {
                    self.scheduler.remove_process(pid);
                    self.scheduler.add_process_to_queue(pid, priority as usize);
                    self.stats.record_queue_change(pid);
                }

                format!(
                    "✓ Process {} priority changed from {} to {}",
                    pid, old_priority, priority
                )
            }
            None => format!("Error: Process {} not found", pid),
        }
    }

    fn cmd_sched_stats(&self) -> String {
        let mut output = String::from(
            "╔════════════════════════════════════════════════════════════════╗\n\
             ║           DETAILED SCHEDULER STATISTICS                       ║\n\
             ╚════════════════════════════════════════════════════════════════╝\n\n"
        );

        output.push_str("System Summary:\n");
        output.push_str("────────────────────────────────────────────────────────────\n");
        output.push_str(&format!("Total Processes:          {}\n", self.manager.process_count()));
        output.push_str(&format!("Scheduler State:          Running\n"));
        output.push_str(&format!("Current Process:          {}\n\n",
                                 self.scheduler.current_process().map_or("None".to_string(), |p| p.to_string())));

        let lengths = self.scheduler.queue_lengths();
        output.push_str("Queue Status:\n");
        output.push_str("────────────────────────────────────────────────────────────\n");
        output.push_str(&format!("Q0 (8ms):   {} processes\n", lengths[0]));
        output.push_str(&format!("Q1 (16ms):  {} processes\n", lengths[1]));
        output.push_str(&format!("Q2 (32ms):  {} processes\n", lengths[2]));
        output.push_str(&format!("Q3 (64ms):  {} processes\n\n", lengths[3]));

        output.push_str("Performance Metrics:\n");
        output.push_str("────────────────────────────────────────────────────────────\n");
        output.push_str(&format!("CPU Utilization:          {:.2}%\n", self.stats.cpu_utilization()));
        output.push_str(&format!("Context Switch Rate:      {:.4} per tick\n", self.stats.context_switch_rate()));
        output.push_str(&format!("Total Context Switches:   {}\n", self.stats.total_context_switches));
        output.push_str(&format!("Total Execution Time:     {}ms\n\n", self.stats.total_execution_time));

        output.push_str("Queue Distribution:\n");
        output.push_str("────────────────────────────────────────────────────────────\n");
        for (idx, &len) in lengths.iter().enumerate() {
            output.push_str(&format!("Q{}: ", idx));
            for _ in 0..len {
                output.push('■');
            }
            output.push_str(&format!(" ({})\n", len));
        }

        output
    }

    fn cmd_programs(&self) -> String {
        let registry = crate::scheduler::programs::ProgramRegistry::new();
        registry.print_catalog()
    }

    fn cmd_run_program(&mut self, program_name: &str) -> String {
        let registry = crate::scheduler::programs::ProgramRegistry::new();

        match registry.get_program(program_name) {
            Some(program) => {
                let pid = self.manager.create_process(1);
                self.scheduler.add_process(pid);
                self.stats.record_process_created(pid);

                format!(
                    "✓ Program '{}' started as PID {}\n\
                     Description: {}\n\
                     Behavior: {}\n\
                     Expected Priority: Q{}",
                    program.name,
                    pid,
                    program.description,
                    program.behavior_description(),
                    program.expected_priority
                )
            }
            None => {
                format!("Error: Program '{}' not found. Type 'programs' to see available programs.", program_name)
            }
        }
    }

    // ========================================================================
    // STATISTICS COMMANDS
    // ========================================================================

    fn cmd_stats(&self) -> String {
        self.stats.summary_report()
    }

    fn cmd_metrics(&self, pid: u32) -> String {
        match self.stats.get_process_metrics(pid) {
            Some(metrics) => {
                format!(
                    "Process Metrics (PID: {})\n\
                     ════════════════════════════════════════════════════════════\n\
                     Turnaround Time:     {}ms\n\
                     Response Time:       {}ms\n\
                     Waiting Time:        {}ms\n\
                     Execution Time:      {}ms\n\
                     Context Switches:    {}\n\
                     Queue Changes:       {}\n",
                    metrics.pid,
                    metrics.turnaround_time,
                    metrics.response_time,
                    metrics.waiting_time,
                    metrics.execution_time,
                    metrics.context_switches,
                    metrics.queue_changes,
                )
            }
            None => format!("Error: No metrics found for process {}", pid),
        }
    }

    fn cmd_reset_stats(&mut self) -> String {
        self.stats.reset();
        "✓ All statistics have been reset".to_string()
    }

    // ========================================================================
    // SYSTEM COMMANDS
    // ========================================================================

    fn cmd_help(&self) -> String {
        String::from(
            "Available Commands:\n\
             ────────────────────────────────────────────────────\n\
             Process Management:\n\
               fork [ppid]          - Create new process\n\
               ps                   - List all processes\n\
               kill <pid>           - Terminate process\n\
               run <pid>            - Transition to running\n\
             \n\
             Process State:\n\
               block <pid>          - Block process (I/O)\n\
               unblock <pid>        - Unblock process\n\
               info <pid>           - Process information\n\
             \n\
             Scheduler Control:\n\
               nice <pid> <prio>    - Change priority (0-3)\n\
               schedule <cycles>    - Simulate N cycles\n\
               queues               - Show queue state\n\
               sched_stats          - Detailed statistics\n\
             \n\
             Programs:\n\
               programs             - List available programs\n\
               run_program <n>      - Execute a program\n\
             \n\
             Statistics:\n\
               stats                - Show metrics\n\
               metrics <pid>        - Process metrics\n\
               reset_stats          - Clear statistics\n\
             \n\
             System:\n\
               help                 - Show this help\n\
               exit                 - Exit simulator\n"
        )
    }

    // ========================================================================
    // UTILITY METHODS
    // ========================================================================

    pub fn is_running(&self) -> bool {
        self.running
    }

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
    fn test_parse_nice() {
        let cmd = parse_command("nice 2 0").unwrap();
        assert_eq!(cmd, Command::Nice { pid: 2, priority: 0 });
    }

    #[test]
    fn test_parse_sched_stats() {
        let cmd = parse_command("sched_stats").unwrap();
        assert_eq!(cmd, Command::SchedStats);
    }

    #[test]
    fn test_parse_programs() {
        let cmd = parse_command("programs").unwrap();
        assert_eq!(cmd, Command::Programs);
    }

    #[test]
    fn test_parse_run_program() {
        let cmd = parse_command("run_program video_encoder").unwrap();
        assert_eq!(cmd, Command::RunProgram { program_name: "video_encoder".to_string() });
    }

    #[test]
    fn test_parse_stats() {
        let cmd = parse_command("stats").unwrap();
        assert_eq!(cmd, Command::Stats);
    }

    #[test]
    fn test_parse_metrics() {
        let cmd = parse_command("metrics 2").unwrap();
        assert_eq!(cmd, Command::Metrics { pid: 2 });
    }

    #[test]
    fn test_shell_creation() {
        let shell = Shell::new();
        assert!(shell.is_running());
        assert_eq!(shell.process_count(), 1);
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

        assert!(result.contains("✓"));

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