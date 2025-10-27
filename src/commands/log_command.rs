use std::collections::HashMap;
use rhai::{Engine, Scope};
use crate::commands::WorkflowCommand;

pub struct LogCommand;

impl WorkflowCommand for LogCommand {
    fn name(&self) -> &'static str {
        "Log"
    }

    fn execute(
        &self,
        inputs: &HashMap<String, String>,
        scope: &mut Scope,
        engine: &Engine,
        step_name: &str,
        step_id: u64,
    ) -> Result<String, String> {
        let expr = inputs.get("message").ok_or("Missing 'message' input")?;
        let msg = engine
            .eval_with_scope::<String>(scope, expr)
            .map_err(|e| format!("Failed to evaluate message: {}", e))?;

        println!("[LOG] Step {} (#{}): {}", step_name, step_id, msg);
        Ok(format!("Log message: {}", msg))
    }
}
