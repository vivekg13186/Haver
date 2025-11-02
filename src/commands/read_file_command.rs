use std::collections::HashMap;
use std::fs::read_to_string;
use boa_engine::Context;
use crate::commands::WorkflowCommand;

pub struct ReadFileCommand;

impl WorkflowCommand for ReadFileCommand {
    fn name(&self) -> &'static str {
        "ReadFile"
    }

    fn execute(
        &self,
        inputs: &HashMap<String, String>,
       context : &mut Context,
        _step_name: &str,
        _step_id: u64,
    ) -> Result<HashMap<String, String>, String> {
        let path_expr = inputs.get("path").ok_or("Missing 'path' input")?;
        let path = engine
            .eval_with_scope::<String>(scope, path_expr)
            .map_err(|e| format!("Failed to evaluate path: {}", e))?;

        let content = read_to_string(&path).map_err(|e| format!("File read failed: {}", e))?;

        let mut output = HashMap::new();
        output.insert("path".to_string(), path);
        output.insert("content".to_string(), content);
        Ok(output)
    }
}
