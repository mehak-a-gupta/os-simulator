
use os_simulator::process::{ProcessManager, ProcessState};
use os_simulator::scheduler::MLFQScheduler;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║          macOS Teaching OS Simulator v0.2                      ║");
    println!("║              MLFQ Scheduler Demo                               ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let mut manager = ProcessManager::new();
    let mut scheduler = MLFQScheduler::new();

    // Create init process (PID 1)
    let init_pid = manager.create_process(0);
    println!("✓ Init process created (PID: {})", init_pid);
    scheduler.add_process(init_pid);

    // Create child processes
    let pid2 = manager.create_process(init_pid);
    let pid3 = manager.create_process(init_pid);
    let pid4 = manager.create_process(init_pid);
    println!("✓ Child processes created (PIDs: {}, {}, {})\n", pid2, pid3, pid4);

    // Add to scheduler
    scheduler.add_process(pid2);
    scheduler.add_process(pid3);
    scheduler.add_process(pid4);

    // Display initial state
    println!("Initial Queue Lengths: Q0={}, Q1={}, Q2={}, Q3={}",
             scheduler.queue_lengths()[0],
             scheduler.queue_lengths()[1],
             scheduler.queue_lengths()[2],
             scheduler.queue_lengths()[3]
    );

    println!("\n{}", "─".repeat(70));
    println!("Simulating CPU Scheduling (5 scheduling cycles):");
    println!("{}\n", "─".repeat(70));

    // Simulate scheduling
    for cycle in 1..=5 {
        println!("Cycle {}:", cycle);

        if let Some((pid, quantum)) = scheduler.next_process() {
            if let Some(process) = manager.get_process_mut(pid) {
                process.set_state(ProcessState::Running);
                println!("  ➜ Running PID: {} | Quantum: {}ms", pid, quantum);

                // Simulate execution
                scheduler.tick(quantum);

                // Decide what happens next (for demo, alternate behaviors)
                if cycle % 2 == 0 {
                    // Process used full quantum -> demoted
                    scheduler.process_used_full_quantum(pid);
                    let new_queue = scheduler.get_process_queue(pid).unwrap_or(3);
                    println!("  • Process used full quantum → moved to Q{}", new_queue);
                } else {
                    // Process yielded early -> promoted
                    scheduler.process_yielded_early(pid);
                    let new_queue = scheduler.get_process_queue(pid).unwrap_or(0);
                    println!("  • Process yielded early → moved to Q{}", new_queue);
                }

                process.set_state(ProcessState::Ready);
            }
        }

        println!("  Queue state: Q0={}, Q1={}, Q2={}, Q3={}",
                 scheduler.queue_lengths()[0],
                 scheduler.queue_lengths()[1],
                 scheduler.queue_lengths()[2],
                 scheduler.queue_lengths()[3]
        );
        println!();
    }

    println!("{}", "─".repeat(70));
    println!("Final Process Information:");
    println!("{}", "─".repeat(70));
    for process in manager.active_processes() {
        let queue = scheduler.get_process_queue(process.pid).map_or("N/A".to_string(), |q| format!("Q{}", q));
        println!("PID: {:<3} | PPID: {:<3} | State: {:<10?} | Queue: {}",
                 process.pid, process.ppid, process.state, queue);
    }

    println!("\n{}", "─".repeat(70));
    println!("Key Features Demonstrated:");
    println!("{}", "─".repeat(70));
    println!("✓ Multi-Level Feedback Queue (4 priority levels)");
    println!("✓ Dynamic priority adjustment based on behavior");
    println!("✓ Process demotion when using full quantum");
    println!("✓ Process promotion when yielding early");
    println!("✓ Anti-starvation mechanism (priority boost every 100 ticks)");
    println!("\nNext steps:");
    println!("  1. Implement CLI Shell with commands (Week 1, Days 6-7)");
    println!("  2. Add Memory Management (Week 2)");
    println!("  3. Add Filesystem (Week 3)");
}