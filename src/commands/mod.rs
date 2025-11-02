use std::collections::HashMap;
use boa_engine::Context;
// --------- The Core Trait ----------
pub trait WorkflowCommand {
    fn name(&self) -> &'static str;
    fn execute(
        &self,
        inputs: &HashMap<String, String>,
        context: &mut Context,
        step_name: &str,
        step_id: u64,
    ) -> Result<HashMap<String, String>, String>;
}

// --------- Public Registry Type ----------
pub type CommandRegistry = HashMap<String, Box<dyn WorkflowCommand>>;

// --------- Public API ----------
pub fn register_default_commands() -> CommandRegistry {
    let mut registry: CommandRegistry = HashMap::new();

    // Register built-in commands here
    registry.insert(
        "Log".into(),
        Box::new(crate::commands::log_command::LogCommand),
    );
    registry.insert(
        "WriteFile".into(),
        Box::new(crate::commands::write_file_command::WriteFileCommand),
    );
    registry.insert(
        "ReadFile".into(),
        Box::new(crate::commands::read_file_command::ReadFileCommand),
    );
    registry.insert(
        "AppendFile".into(),
        Box::new(crate::commands::append_file_command::AppendFileCommand),
    );
    registry.insert(
        "DeleteFile".into(),
        Box::new(crate::commands::delete_file_command::DeleteFileCommand),
    );
    registry.insert(
        "RestApi".into(),
        Box::new(crate::commands::rest_api_command::RestApiCommand),
    );

    registry
}

// Include submodules
pub mod append_file_command;
pub mod delete_file_command;
pub mod log_command;
pub mod read_file_command;
pub mod rest_api_command;
pub mod write_file_command;
