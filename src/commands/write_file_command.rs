use std::collections::HashMap;
use rhai::{Engine, Scope};
use std::fs::write;
use crate::commands::WorkflowCommand;

pub struct WriteFileCommand;

impl WorkflowCommand for WriteFileCommand {
    fn name(&self) -> &'static str {
        "WriteFile"
    }

    fn execute(
        &self,
        inputs: &HashMap<String, String>,
        scope: &mut Scope,
        engine: &Engine,
        _step_name: &str,
        _step_id: u64,
    ) -> Result<String, String> {
        let path_expr = inputs.get("path").ok_or("Missing 'path' input")?;
        let text_expr = inputs.get("text").ok_or("Missing 'text' input")?;

        let path = engine
            .eval_with_scope::<String>(scope, path_expr)
            .map_err(|e| format!("Failed to evaluate path: {}", e))?;
        let text = engine
            .eval_with_scope::<String>(scope, text_expr)
            .map_err(|e| format!("Failed to evaluate text: {}", e))?;

        write(&path, text.as_bytes()).map_err(|e| format!("File write failed: {}", e))?;

        println!("[FILE] Wrote file: {}", path);
        Ok(format!("Wrote file: {}", path))
    }
}
