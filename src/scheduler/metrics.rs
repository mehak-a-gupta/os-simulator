
use std::collections::HashMap;

/// Metrics for a single process
#[derive(Debug, Clone)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub turnaround_time: u64,      // Time from creation to termination (ms)
    pub response_time: u64,         // Time from creation to first run (ms)
    pub waiting_time: u64,          // Turnaround - execution time (ms)
    pub execution_time: u64,        // Total time actually running (ms)
    pub context_switches: u32,      // How many times this process was switched
    pub queue_changes: u32,         // How many times it moved between queues
}

impl ProcessMetrics {
    pub fn new(pid: u32) -> Self {
        ProcessMetrics {
            pid,
            turnaround_time: 0,
            response_time: 0,
            waiting_time: 0,
            execution_time: 0,
            context_switches: 0,
            queue_changes: 0,
        }
    }
}

/// System-wide scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    /// Per-process metrics
    pub process_metrics: HashMap<u32, ProcessMetrics>,

    /// Total number of context switches in system
    pub total_context_switches: u64,

    /// Total system time elapsed (ticks/cycles)
    pub total_ticks: u64,

    /// Number of processes that have been created
    pub processes_created: u32,

    /// Number of processes that have terminated
    pub processes_terminated: u32,

    /// Total time all processes spent executing
    pub total_execution_time: u64,

    /// Total time all processes spent waiting
    pub total_waiting_time: u64,

    /// Track queue depths over time (for analysis)
    pub queue_depth_samples: Vec<[usize; 4]>,

    /// Time when stats were started/reset
    pub start_time: std::time::Instant,
}

impl SchedulerStats {
    pub fn new() -> Self {
        SchedulerStats {
            process_metrics: HashMap::new(),
            total_context_switches: 0,
            total_ticks: 0,
            processes_created: 0,
            processes_terminated: 0,
            total_execution_time: 0,
            total_waiting_time: 0,
            queue_depth_samples: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Record a new process creation
    pub fn record_process_created(&mut self, pid: u32) {
        self.processes_created += 1;
        self.process_metrics.insert(pid, ProcessMetrics::new(pid));
    }

    /// Record a context switch
    pub fn record_context_switch(&mut self, pid: u32) {
        self.total_context_switches += 1;

        if let Some(metrics) = self.process_metrics.get_mut(&pid) {
            metrics.context_switches += 1;
        }
    }

    /// Record queue change for a process
    pub fn record_queue_change(&mut self, pid: u32) {
        if let Some(metrics) = self.process_metrics.get_mut(&pid) {
            metrics.queue_changes += 1;
        }
    }

    /// Record execution time for a process
    pub fn record_execution_time(&mut self, pid: u32, time: u64) {
        self.total_execution_time += time;

        if let Some(metrics) = self.process_metrics.get_mut(&pid) {
            metrics.execution_time += time;
        }
    }

    /// Record process termination with metrics
    pub fn record_process_terminated(&mut self, pid: u32, turnaround: u64, response: u64) {
        self.processes_terminated += 1;

        if let Some(metrics) = self.process_metrics.get_mut(&pid) {
            metrics.turnaround_time = turnaround;
            metrics.response_time = response;
            metrics.waiting_time = turnaround.saturating_sub(metrics.execution_time);
            self.total_waiting_time += metrics.waiting_time;
        }
    }

    /// Sample current queue depths
    pub fn sample_queue_depths(&mut self, depths: [usize; 4]) {
        self.queue_depth_samples.push(depths);
    }

    /// Record a tick
    pub fn record_tick(&mut self) {
        self.total_ticks += 1;
    }

    /// Get average turnaround time across all terminated processes
    pub fn avg_turnaround_time(&self) -> f64 {
        if self.processes_terminated == 0 {
            return 0.0;
        }

        let total: u64 = self.process_metrics
            .values()
            .filter(|m| m.turnaround_time > 0)
            .map(|m| m.turnaround_time)
            .sum();

        total as f64 / self.processes_terminated as f64
    }

    /// Get average response time
    pub fn avg_response_time(&self) -> f64 {
        if self.processes_terminated == 0 {
            return 0.0;
        }

        let total: u64 = self.process_metrics
            .values()
            .filter(|m| m.response_time > 0)
            .map(|m| m.response_time)
            .sum();

        total as f64 / self.processes_terminated as f64
    }

    /// Get average waiting time
    pub fn avg_waiting_time(&self) -> f64 {
        if self.processes_terminated == 0 {
            return 0.0;
        }

        self.total_waiting_time as f64 / self.processes_terminated as f64
    }

    /// Get CPU utilization (execution time / total time)
    pub fn cpu_utilization(&self) -> f64 {
        if self.total_ticks == 0 {
            return 0.0;
        }

        (self.total_execution_time as f64 / self.total_ticks as f64) * 100.0
    }

    /// Get context switch rate (switches per tick)
    pub fn context_switch_rate(&self) -> f64 {
        if self.total_ticks == 0 {
            return 0.0;
        }

        self.total_context_switches as f64 / self.total_ticks as f64
    }

    /// Get average queue depth for specific queue
    pub fn avg_queue_depth(&self, queue_idx: usize) -> f64 {
        if self.queue_depth_samples.is_empty() {
            return 0.0;
        }

        let total: usize = self.queue_depth_samples
            .iter()
            .map(|sample| sample[queue_idx])
            .sum();

        total as f64 / self.queue_depth_samples.len() as f64
    }

    /// Get process-specific metrics
    pub fn get_process_metrics(&self, pid: u32) -> Option<&ProcessMetrics> {
        self.process_metrics.get(&pid)
    }

    /// Generate summary report
    pub fn summary_report(&self) -> String {
        let mut report = String::from(
            "╔════════════════════════════════════════════════════════════════╗\n\
             ║             SCHEDULER METRICS AND STATISTICS                  ║\n\
             ╚════════════════════════════════════════════════════════════════╝\n\n"
        );

        // System Overview
        report.push_str("System Overview:\n");
        report.push_str("─────────────────────────────────────────────────────────────\n");
        report.push_str(&format!("Total Ticks:              {}\n", self.total_ticks));
        report.push_str(&format!("Processes Created:        {}\n", self.processes_created));
        report.push_str(&format!("Processes Terminated:     {}\n", self.processes_terminated));
        report.push_str(&format!("Total Context Switches:   {}\n\n", self.total_context_switches));

        // Performance Metrics
        report.push_str("Performance Metrics:\n");
        report.push_str("─────────────────────────────────────────────────────────────\n");
        report.push_str(&format!("CPU Utilization:          {:.2}%\n", self.cpu_utilization()));
        report.push_str(&format!("Context Switch Rate:      {:.4} per tick\n", self.context_switch_rate()));
        report.push_str(&format!("Total Execution Time:     {}ms\n", self.total_execution_time));
        report.push_str(&format!("Total Waiting Time:       {}ms\n\n", self.total_waiting_time));

        // Average Metrics
        report.push_str("Average Metrics (Terminated Processes):\n");
        report.push_str("─────────────────────────────────────────────────────────────\n");
        report.push_str(&format!("Avg Turnaround Time:      {:.2}ms\n", self.avg_turnaround_time()));
        report.push_str(&format!("Avg Response Time:        {:.2}ms\n", self.avg_response_time()));
        report.push_str(&format!("Avg Waiting Time:         {:.2}ms\n\n", self.avg_waiting_time()));

        // Queue Analysis
        report.push_str("Queue Depth Analysis:\n");
        report.push_str("─────────────────────────────────────────────────────────────\n");
        report.push_str(&format!("Avg Q0 Depth:             {:.2}\n", self.avg_queue_depth(0)));
        report.push_str(&format!("Avg Q1 Depth:             {:.2}\n", self.avg_queue_depth(1)));
        report.push_str(&format!("Avg Q2 Depth:             {:.2}\n", self.avg_queue_depth(2)));
        report.push_str(&format!("Avg Q3 Depth:             {:.2}\n\n", self.avg_queue_depth(3)));

        // Per-Process Metrics
        if !self.process_metrics.is_empty() {
            report.push_str("Per-Process Metrics:\n");
            report.push_str("─────────────────────────────────────────────────────────────\n");
            report.push_str("PID  Turnaround  Response  Waiting  Execution  Ctx-Sw  Q-Changes\n");
            report.push_str("─────────────────────────────────────────────────────────────\n");

            for pid in self.process_metrics.keys() {
                if let Some(metrics) = self.process_metrics.get(pid) {
                    report.push_str(&format!(
                        "{:<4} {:<10} {:<9} {:<8} {:<10} {:<7} {:<10}\n",
                        metrics.pid,
                        format!("{}ms", metrics.turnaround_time),
                        format!("{}ms", metrics.response_time),
                        format!("{}ms", metrics.waiting_time),
                        format!("{}ms", metrics.execution_time),
                        metrics.context_switches,
                        metrics.queue_changes,
                    ));
                }
            }
        }

        report.push_str("\n");
        report
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.process_metrics.clear();
        self.total_context_switches = 0;
        self.total_ticks = 0;
        self.processes_created = 0;
        self.processes_terminated = 0;
        self.total_execution_time = 0;
        self.total_waiting_time = 0;
        self.queue_depth_samples.clear();
        self.start_time = std::time::Instant::now();
    }
}

impl Default for SchedulerStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_creation() {
        let stats = SchedulerStats::new();
        assert_eq!(stats.processes_created, 0);
        assert_eq!(stats.total_context_switches, 0);
    }

    #[test]
    fn test_record_process_created() {
        let mut stats = SchedulerStats::new();
        stats.record_process_created(1);
        stats.record_process_created(2);

        assert_eq!(stats.processes_created, 2);
        assert!(stats.process_metrics.contains_key(&1));
        assert!(stats.process_metrics.contains_key(&2));
    }

    #[test]
    fn test_record_context_switch() {
        let mut stats = SchedulerStats::new();
        stats.record_process_created(1);

        stats.record_context_switch(1);
        stats.record_context_switch(1);

        assert_eq!(stats.total_context_switches, 2);
        assert_eq!(stats.process_metrics.get(&1).unwrap().context_switches, 2);
    }

    #[test]
    fn test_record_execution_time() {
        let mut stats = SchedulerStats::new();
        stats.record_process_created(1);

        stats.record_execution_time(1, 50);
        stats.record_execution_time(1, 30);

        assert_eq!(stats.total_execution_time, 80);
        assert_eq!(stats.process_metrics.get(&1).unwrap().execution_time, 80);
    }

    #[test]
    fn test_record_process_terminated() {
        let mut stats = SchedulerStats::new();
        stats.record_process_created(1);
        stats.record_execution_time(1, 100);

        stats.record_process_terminated(1, 200, 50);

        assert_eq!(stats.processes_terminated, 1);
        let metrics = stats.process_metrics.get(&1).unwrap();
        assert_eq!(metrics.turnaround_time, 200);
        assert_eq!(metrics.response_time, 50);
        assert_eq!(metrics.waiting_time, 100);
    }

    #[test]
    fn test_cpu_utilization() {
        let mut stats = SchedulerStats::new();
        stats.total_ticks = 100;
        stats.total_execution_time = 50;

        let utilization = stats.cpu_utilization();
        assert_eq!(utilization, 50.0);
    }

    #[test]
    fn test_avg_turnaround_time() {
        let mut stats = SchedulerStats::new();
        stats.record_process_created(1);
        stats.record_process_created(2);

        stats.record_process_terminated(1, 100, 0);
        stats.record_process_terminated(2, 200, 0);

        let avg = stats.avg_turnaround_time();
        assert_eq!(avg, 150.0);
    }

    #[test]
    fn test_avg_response_time() {
        let mut stats = SchedulerStats::new();
        stats.record_process_created(1);
        stats.record_process_created(2);

        stats.record_process_terminated(1, 100, 10);
        stats.record_process_terminated(2, 200, 20);

        let avg = stats.avg_response_time();
        assert_eq!(avg, 15.0);
    }

    #[test]
    fn test_avg_queue_depth() {
        let mut stats = SchedulerStats::new();
        stats.sample_queue_depths([1, 2, 3, 4]);
        stats.sample_queue_depths([2, 3, 4, 5]);

        let avg_q0 = stats.avg_queue_depth(0);
        assert_eq!(avg_q0, 1.5);
    }

    #[test]
    fn test_context_switch_rate() {
        let mut stats = SchedulerStats::new();
        stats.total_ticks = 100;
        stats.total_context_switches = 25;

        let rate = stats.context_switch_rate();
        assert_eq!(rate, 0.25);
    }

    #[test]
    fn test_record_queue_change() {
        let mut stats = SchedulerStats::new();
        stats.record_process_created(1);

        stats.record_queue_change(1);
        stats.record_queue_change(1);
        stats.record_queue_change(1);

        assert_eq!(stats.process_metrics.get(&1).unwrap().queue_changes, 3);
    }

    #[test]
    fn test_stats_reset() {
        let mut stats = SchedulerStats::new();
        stats.record_process_created(1);
        stats.total_ticks = 100;

        stats.reset();

        assert_eq!(stats.processes_created, 0);
        assert_eq!(stats.total_ticks, 0);
        assert!(stats.process_metrics.is_empty());
    }

    #[test]
    fn test_summary_report() {
        let mut stats = SchedulerStats::new();
        stats.record_process_created(1);
        stats.record_execution_time(1, 50);
        stats.record_process_terminated(1, 100, 10);
        stats.total_ticks = 100;

        let report = stats.summary_report();

        assert!(report.contains("SCHEDULER METRICS"));
        assert!(report.contains("Total Ticks"));
        assert!(report.contains("CPU Utilization"));
    }
}