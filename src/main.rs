// src/main.rs

use os_simulator::shell::{Shell, parse_command};
use std::io::{self, Write};

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║          OS Simulator v0.3                                     ║");
    println!("║              Interactive CLI Shell                             ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("Welcome to the OS Simulator!");
    println!("Type 'help' for available commands or 'exit' to quit.\n");

    let mut shell = Shell::new();

    // Main REPL loop
    loop {
        // Print prompt
        print!("os> ");
        io::stdout().flush().unwrap();

        // Read input
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let trimmed = input.trim();

                // Skip empty lines
                if trimmed.is_empty() {
                    continue;
                }

                // Parse and execute command
                match parse_command(trimmed) {
                    Some(cmd) => {
                        let output = shell.execute(cmd);
                        println!("{}", output);

                        // Check if we should exit
                        if !shell.is_running() {
                            break;
                        }
                    }
                    None => {
                        println!("Error: Unknown command '{}'. Type 'help' for available commands.", trimmed);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                      Simulator Exiting                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}