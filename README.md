# macOS Teaching OS Simulator

A comprehensive operating system simulator built in Rust for educational purposes. This project implements core OS concepts including process scheduling, virtual memory management, filesystem operations, security models, and inter-process communication.

## Features

- **Multi-Level Feedback Queue (MLFQ) Scheduler**: CPU scheduling with priority queues
- **Virtual Memory Management**: Paging system with multiple page replacement algorithms
- **Filesystem**: Inode-based filesystem with directory structure
- **Security Model**: Capability-based security tokens
- **IPC Mechanisms**: Pipes, shared memory, message queues, and signals
- **Interactive CLI Shell**: Command-line interface for interacting with the OS

## Project Structure

```
src/
├── scheduler/   - CPU scheduling implementation
├── process/     - Process management and control
├── memory/      - Virtual memory and paging
├── fs/          - Filesystem and inode management
├── security/    - Capability-based security
├── ipc/         - Inter-process communication
└── main.rs      - CLI shell and entry point
```

## Timeline

- Week 1: Process Scheduler & Foundation
- Week 2: Memory Management
- Week 3: Filesystem & Security
- Week 4: IPC & Polish

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```

## Testing

```bash
cargo test
```
