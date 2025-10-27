use std::collections::HashMap;
use rhai::{Engine, Scope};

// --------- The Core Trait ----------
pub trait WorkflowCommand {
    fn name(&self) -> &'static str;
    fn execute(
        &self,
        inputs: &HashMap<String, String>,
        scope: &mut Scope,
        engine: &Engine,
        step_name: &str,
        step_id: u64,
    ) -> Result<String, String>;
}

// --------- Public Registry Type ----------
pub type CommandRegistry = HashMap<String, Box<dyn WorkflowCommand>>;

// --------- Public API ----------
pub fn register_default_commands() -> CommandRegistry {
    let mut registry: CommandRegistry = HashMap::new();

    // Register built-in commands here
    registry.insert("Log".into(), Box::new(crate::commands::log_command::LogCommand));
    registry.insert("WriteFile".into(), Box::new(crate::commands::write_file_command::WriteFileCommand));

    registry
}

// Include submodules
pub mod log_command;
pub mod write_file_command;
