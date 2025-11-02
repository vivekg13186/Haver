use std::collections::HashMap;
use std::fs::remove_file;
use boa_engine::Context;
use crate::commands::WorkflowCommand;

pub struct DeleteFileCommand;

impl WorkflowCommand for DeleteFileCommand {
    fn name(&self) -> &'static str {
        "DeleteFile"
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

        remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))?;

        let mut output = HashMap::new();
        output.insert("path".to_string(), path);
        output.insert("status".to_string(), "deleted".to_string());
        Ok(output)
    }
}
