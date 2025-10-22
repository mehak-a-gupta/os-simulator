// src/scheduler/programs.rs
// Mock programs for scheduler testing

use std::collections::HashMap;

/// Program type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramType {
    CpuBound,
    IoBound,
    Interactive,
    Mixed,
    Batch,
}

/// Mock program definition
#[derive(Debug, Clone)]
pub struct Program {
    pub name: String,
    pub program_type: ProgramType,
    pub description: String,
    pub typical_quantum_usage: f32,
    pub expected_priority: u8,
}

impl Program {
    pub fn new(name: &str, program_type: ProgramType, description: &str, usage: f32) -> Self {
        let expected_priority = match program_type {
            ProgramType::CpuBound => 3,
            ProgramType::IoBound => 0,
            ProgramType::Interactive => 0,
            ProgramType::Mixed => 1,
            ProgramType::Batch => 2,
        };

        Program {
            name: name.to_string(),
            program_type,
            description: description.to_string(),
            typical_quantum_usage: usage,
            expected_priority,
        }
    }

    pub fn execute_quantum(&self) -> bool {
        rand::random::<f32>() < self.typical_quantum_usage
    }

    pub fn behavior_description(&self) -> String {
        match self.program_type {
            ProgramType::CpuBound => {
                "Runs for full time quantum every cycle (CPU-intensive)".to_string()
            }
            ProgramType::IoBound => {
                "Frequently yields early for I/O operations (responsive)".to_string()
            }
            ProgramType::Interactive => {
                "Yields on user interaction (very responsive)".to_string()
            }
            ProgramType::Mixed => {
                "Mixes CPU and I/O operations (balanced)".to_string()
            }
            ProgramType::Batch => {
                "Mostly CPU with occasional I/O (background)".to_string()
            }
        }
    }
}

/// Program registry
pub struct ProgramRegistry {
    programs: HashMap<String, Program>,
}

impl ProgramRegistry {
    pub fn new() -> Self {
        let mut programs = HashMap::new();

        programs.insert(
            "video_encoder".to_string(),
            Program::new(
                "video_encoder",
                ProgramType::CpuBound,
                "Encodes video files to different formats",
                0.95,
            ),
        );

        programs.insert(
            "compiler".to_string(),
            Program::new(
                "compiler",
                ProgramType::CpuBound,
                "Compiles source code to executable",
                0.92,
            ),
        );

        programs.insert(
            "rendering".to_string(),
            Program::new(
                "rendering",
                ProgramType::CpuBound,
                "3D graphics rendering engine",
                0.98,
            ),
        );

        programs.insert(
            "web_browser".to_string(),
            Program::new(
                "web_browser",
                ProgramType::IoBound,
                "Web browser waiting for network responses",
                0.15,
            ),
        );

        programs.insert(
            "file_transfer".to_string(),
            Program::new(
                "file_transfer",
                ProgramType::IoBound,
                "Transfers files over network",
                0.20,
            ),
        );

        programs.insert(
            "text_editor".to_string(),
            Program::new(
                "text_editor",
                ProgramType::Interactive,
                "Text editor waiting for keyboard input",
                0.10,
            ),
        );

        programs.insert(
            "terminal".to_string(),
            Program::new(
                "terminal",
                ProgramType::Interactive,
                "Terminal shell waiting for commands",
                0.05,
            ),
        );

        programs.insert(
            "music_player".to_string(),
            Program::new(
                "music_player",
                ProgramType::Interactive,
                "Music player awaiting user interaction",
                0.12,
            ),
        );

        programs.insert(
            "database".to_string(),
            Program::new(
                "database",
                ProgramType::Mixed,
                "Database server (queries + disk I/O)",
                0.45,
            ),
        );

        programs.insert(
            "game".to_string(),
            Program::new(
                "game",
                ProgramType::Mixed,
                "Game with graphics and I/O",
                0.50,
            ),
        );

        programs.insert(
            "backup".to_string(),
            Program::new(
                "backup",
                ProgramType::Batch,
                "Backup system (sequential file processing)",
                0.70,
            ),
        );

        programs.insert(
            "search".to_string(),
            Program::new(
                "search",
                ProgramType::Batch,
                "Full-text search indexing",
                0.75,
            ),
        );

        ProgramRegistry { programs }
    }

    pub fn get_program(&self, name: &str) -> Option<Program> {
        self.programs.get(name).cloned()
    }

    pub fn list_programs(&self) -> Vec<&Program> {
        self.programs.values().collect()
    }

    pub fn get_by_type(&self, program_type: ProgramType) -> Vec<&Program> {
        self.programs
            .values()
            .filter(|p| p.program_type == program_type)
            .collect()
    }

    pub fn print_catalog(&self) -> String {
        let mut output = String::from(
            "╔════════════════════════════════════════════════════════════════╗\n\
             ║                  AVAILABLE PROGRAMS                            ║\n\
             ╚════════════════════════════════════════════════════════════════╝\n\n"
        );

        output.push_str("CPU-Bound Programs (High CPU Usage):\n");
        output.push_str("────────────────────────────────────────────────────────────\n");
        for prog in self.get_by_type(ProgramType::CpuBound) {
            output.push_str(&format!(
                "  {} - {}\n    Usage: {:.0}% quantum\n",
                prog.name, prog.description,
                prog.typical_quantum_usage * 100.0
            ));
        }

        output.push_str("\nI/O-Bound Programs (Frequently Yield):\n");
        output.push_str("────────────────────────────────────────────────────────────\n");
        for prog in self.get_by_type(ProgramType::IoBound) {
            output.push_str(&format!(
                "  {} - {}\n    Usage: {:.0}% quantum\n",
                prog.name, prog.description,
                prog.typical_quantum_usage * 100.0
            ));
        }

        output.push_str("\nInteractive Programs (Very Responsive):\n");
        output.push_str("────────────────────────────────────────────────────────────\n");
        for prog in self.get_by_type(ProgramType::Interactive) {
            output.push_str(&format!(
                "  {} - {}\n    Usage: {:.0}% quantum\n",
                prog.name, prog.description,
                prog.typical_quantum_usage * 100.0
            ));
        }

        output.push_str("\nMixed Programs (Balanced CPU/IO):\n");
        output.push_str("────────────────────────────────────────────────────────────\n");
        for prog in self.get_by_type(ProgramType::Mixed) {
            output.push_str(&format!(
                "  {} - {}\n    Usage: {:.0}% quantum\n",
                prog.name, prog.description,
                prog.typical_quantum_usage * 100.0
            ));
        }

        output.push_str("\nBatch Programs (Background Processing):\n");
        output.push_str("────────────────────────────────────────────────────────────\n");
        for prog in self.get_by_type(ProgramType::Batch) {
            output.push_str(&format!(
                "  {} - {}\n    Usage: {:.0}% quantum\n",
                prog.name, prog.description,
                prog.typical_quantum_usage * 100.0
            ));
        }

        output.push_str("\nUsage: run_program <program_name>\n");
        output
    }
}

impl Default for ProgramRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_creation() {
        let prog = Program::new("test", ProgramType::CpuBound, "Test program", 0.8);
        assert_eq!(prog.name, "test");
        assert_eq!(prog.program_type, ProgramType::CpuBound);
    }

    #[test]
    fn test_program_registry() {
        let registry = ProgramRegistry::new();
        let prog = registry.get_program("video_encoder");
        assert!(prog.is_some());
    }

    #[test]
    fn test_get_programs_by_type() {
        let registry = ProgramRegistry::new();
        let cpu_programs = registry.get_by_type(ProgramType::CpuBound);
        assert!(cpu_programs.len() > 0);
    }
}