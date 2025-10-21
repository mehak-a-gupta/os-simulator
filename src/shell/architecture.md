# OS Simulator Shell Architecture

## Overview

The OS Simulator shell is an interactive command-line interface (REPL) that allows users to manage and observe an operating system's process scheduling, memory management, and system statistics in real-time.

## Architecture Layers

### Layer 1: Command Parsing
**Module:** `parse_command(input: &str) -> Option<Command>`

```
User Input String
      ↓
Tokenization (split_whitespace)
      ↓
Pattern Matching on first token
      ↓
Parse arguments to specific types (u32, etc.)
      ↓
Return Command enum variant
```

**Responsibility:** Convert raw user text into strongly-typed Command enum
**Validation:** Type-safe, prevents invalid arguments from reaching execution layer

### Layer 2: Command Execution
**Module:** `Shell::execute(cmd: Command) -> String`

```
Command enum
      ↓
Match on Command variant
      ↓
Call appropriate cmd_* handler
      ↓
Return formatted String output
```

**Responsibility:** Execute command logic and return user-facing output
**Error Handling:** All errors become user-readable strings

### Layer 3: System Management
**Modules:** ProcessManager, MLFQScheduler, SchedulerStats

```
Shell
  ├── ProcessManager (manages all processes)
  ├── MLFQScheduler (schedules processes)
  └── SchedulerStats (tracks metrics)
```

**Responsibility:** Maintain OS state and execute system operations

### Layer 4: User Interface (REPL)
**Module:** `src/main.rs`

```
loop {
    print prompt
    read user input
    parse_command()
    shell.execute()
    print output
}
```

**Responsibility:** User interaction loop and I/O handling

---

## Command Taxonomy

### Process Management Commands
| Command | Purpose | Parameters |
|---------|---------|-----------|
| `fork [ppid]` | Create new process | Parent PID (optional) |
| `ps` | List all processes | None |
| `kill <pid>` | Terminate process | Process ID |
| `run <pid>` | Transition to running | Process ID |

### Process State Commands
| Command | Purpose | Parameters |
|---------|---------|-----------|
| `block <pid>` | Block process (I/O wait) | Process ID |
| `unblock <pid>` | Unblock process | Process ID |
| `info <pid>` | Detailed process info | Process ID |

### Scheduler Commands
| Command | Purpose | Parameters |
|---------|---------|-----------|
| `schedule <cycles>` | Simulate N cycles | Number of cycles |
| `queues` | Show queue state | None |

### Statistics Commands
| Command | Purpose | Parameters |
|---------|---------|-----------|
| `stats` | System-wide metrics | None |
| `metrics <pid>` | Process metrics | Process ID |
| `reset_stats` | Clear statistics | None |

### System Commands
| Command | Purpose | Parameters |
|---------|---------|-----------|
| `help` | Show commands | None |
| `exit` | Exit simulator | None |

---

## Data Flow Example: `schedule 10`

```
1. User types: "schedule 10"
   ↓
2. parse_command() → Command::Schedule { cycles: 10 }
   ↓
3. shell.execute(cmd)
   ↓
4. cmd_schedule(10) executed:
   - Loop 10 times
   - Get next process from scheduler
   - Update metrics
   - Simulate process behavior
   - Format output
   ↓
5. Return formatted output string
   ↓
6. main.rs prints output
   ↓
7. Back to prompt
```

---

## Error Handling Strategy

### Input Validation
```rust
// Type-safe parsing
parts[1].parse::<u32>().ok() → Option<u32>

// Semantic validation
get_process(pid) → Option<&Process>
if None { return "Error: Process not found" }
```

### Graceful Degradation
- Invalid commands → "Unknown command" message
- Missing PIDs → "Process not found" message
- Invalid transitions → State-specific error messages

---

## Command Flow Patterns

### Read-Only Pattern (ps, stats, info)
```
parse → no state change
       → read from ProcessManager/SchedulerStats
       → format and return output
```

### State-Modifying Pattern (fork, kill, run)
```
parse → validate inputs
      → modify ProcessManager state
      → update SchedulerStats
      → return success/error message
```

### Interactive Pattern (schedule)
```
parse → loop N times
      → query scheduler
      → update metrics
      → generate output for each iteration
      → accumulate formatted output
      → return full simulation output
```

---

## Shell State

```rust
pub struct Shell {
    manager: ProcessManager,      // Process lifecycle
    scheduler: MLFQScheduler,     // CPU scheduling
    stats: SchedulerStats,        // Performance metrics
    running: bool,                // REPL active?
}
```

**Invariants:**
- Every process in manager is tracked by scheduler (or terminated)
- Stats reflect all recorded operations
- `running` controls REPL loop

---

## Command Registry Pattern

Current implementation uses match statement on Command enum:

```rust
match cmd {
    Command::Fork { ppid } => self.cmd_fork(ppid),
    Command::Ps => self.cmd_ps(),
    Command::Run { pid } => self.cmd_run(pid),
    // ... 13+ commands
}
```

**Advantages:**
- Type-safe (compile-time checking)
- No string lookups
- Exhaustive matching prevents missing commands

**Future Enhancement:** Could use trait-based registry for dynamic commands

---

## Output Formatting

### Consistency Rules
1. **Success messages:** Start with `✓ `
2. **Error messages:** Start with `Error: `
3. **Headers:** Use `════` and `────` separators
4. **Tables:** Aligned columns with consistent spacing
5. **Process states:** Always show as `{:?}` for consistency

### Example Output Blocks
```
Success:
✓ Process created: PID 2 (parent: 1)

Error:
Error: Process 5 not found

Header:
Process Information (PID: 2)
────────────────────────────────────

Table:
PID  PPID STATE       PRIORITY QUEUE
─────────────────────────────────────
1    0    Ready       3        Q3
2    1    Running     3        Q2
```

---

## Integration Points

### With ProcessManager
- `create_process()` → Create new process
- `get_process()` → Read-only access
- `get_process_mut()` → Modify process
- `terminate_process()` → Kill process
- `set_running_process()` → Change running process

### With MLFQScheduler
- `add_process()` → Add to default queue
- `next_process()` → Get next to run
- `process_used_full_quantum()` → Demote
- `process_yielded_early()` → Promote
- `queue_lengths()` → Get queue state

### With SchedulerStats
- `record_process_created()` → Track creation
- `record_context_switch()` → Track switch
- `record_execution_time()` → Track CPU time
- `summary_report()` → Generate report

---

## Testing Strategy

### Command Parsing Tests
- Valid commands → Some(Command)
- Invalid commands → None
- Missing arguments → None or error handling

### Execution Tests
- Process creation → Correct PID, parent relationship
- Process termination → State transition
- Schedule simulation → Correct process execution

### Integration Tests
- fork → ps shows new process
- kill → process in Terminated state
- schedule → metrics updated

---

## Performance Considerations

### Time Complexity
- Command parsing: O(n) where n = input length
- ps command: O(p) where p = number of processes
- schedule command: O(c × log 4) where c = cycles
- stats command: O(p) to aggregate metrics

### Memory Usage
- Shell struct: ~1KB overhead
- Per process: ~200 bytes (ProcessManager)
- Per metric sample: ~32 bytes
- Scaling: Linear with process count

---

## Future Enhancements

### Short-term
1. Command history (↑/↓ arrows)
2. Tab completion for commands
3. Colorized output (success=green, error=red)
4. Pipe commands together

### Medium-term
1. Script file execution
2. Command aliases (e.g., `ls` → `ps`)
3. Batch operations
4. Output redirection to file

### Long-term
1. Web UI dashboard
2. Remote command execution
3. Plugin system for custom commands
4. Real-time visualization

---

## Design Decisions

### Why Match-based Command Registry?
- Type safety: Compiler checks all commands handled
- Performance: Direct dispatch, no hash lookup
- Clarity: Easy to see all available commands
- Testing: Each command is independently testable

### Why String Output?
- Simplicity: Easy formatting and display
- Flexibility: Can redirect to file or network
- Compatibility: Works with any terminal
- Future-proof: Easy to convert to JSON if needed

### Why Separate cmd_* Methods?
- Single Responsibility: Each method does one thing
- Testability: Can test individual commands
- Maintainability: Easy to find command implementation
- Extensibility: Adding commands is straightforward

---

## Example Command Flow: Complete Session

```
os> help                          # List commands
[Shows command list]

os> fork 1                        # Create process
✓ Process created: PID 2 (parent: 1)

os> fork 1                        # Create another
✓ Process created: PID 3 (parent: 1)

os> ps                            # List processes
PID PPID STATE    PRIORITY QUEUE
1   0    Ready    3        Q3
2   1    Ready    3        Q3
3   1    Ready    3        Q3

os> schedule 5                    # Run scheduler
Simulating 5 scheduling cycles:
Cycle 1: PID 1 ran for 64ms in Q3
         • Yielded early → Promoted to Q2
[... more cycles ...]

os> stats                         # Show metrics
[System statistics]

os> metrics 2                      # Process metrics
Process Metrics (PID: 2)
Execution Time: 128ms
Context Switches: 2
Queue Changes: 1

os> kill 2                         # Terminate
✓ Process 2 terminated

os> exit                           # Exit
Simulator Exiting...
```

---

## Architecture Benefits

1. **Modularity:** Each layer has clear responsibility
2. **Testability:** Commands can be tested independently
3. **Maintainability:** Easy to locate and modify functionality
4. **Extensibility:** Adding new commands is straightforward
5. **Type Safety:** Rust compiler prevents many errors
6. **User Experience:** Clear error messages and consistent output