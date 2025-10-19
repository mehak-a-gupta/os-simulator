// src/main.rs

use os_simulator::process::ProcessManager;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                      OS Simulator                              ║");
    println!("║              Project Initialization Complete                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let mut manager = ProcessManager::new();

    // Create init process (PID 1)
    let init_pid = manager.create_process(0);
    println!("✓ Init process created (PID: {})", init_pid);

    // Create a few test processes
    let pid2 = manager.create_process(init_pid);
    let pid3 = manager.create_process(init_pid);
    println!("✓ Test processes created (PID: {}, {})\n", pid2, pid3);

    // Display process information
    println!("Active Processes:");
    println!("─────────────────────────────────────────────────────────────────");
    for process in manager.active_processes() {
        println!(
            "PID: {:<3} | PPID: {:<3} | State: {:<10?} | Priority: {}",
            process.pid, process.ppid, process.state, process.priority
        );
    }

}