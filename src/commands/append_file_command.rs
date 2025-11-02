use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use boa_engine::Context;
use crate::commands::WorkflowCommand;

pub struct AppendFileCommand;

impl WorkflowCommand for AppendFileCommand {
    fn name(&self) -> &'static str {
        "AppendFile"
    }

    fn execute(
        &self,
        inputs: &HashMap<String, String>,
        context : &mut Context,
        _step_name: &str,
        _step_id: u64,
    ) -> Result<HashMap<String, String>, String> {
        let path_expr = inputs.get("path").ok_or("Missing 'path' input")?;
        let text_expr = inputs.get("text").ok_or("Missing 'text' input")?;

        let path = engine
            .eval_with_scope::<String>(scope, path_expr)
            .map_err(|e| format!("Failed to evaluate path: {}", e))?;
        let text = engine
            .eval_with_scope::<String>(scope, text_expr)
            .map_err(|e| format!("Failed to evaluate text: {}", e))?;

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        file.write_all(text.as_bytes())
            .map_err(|e| format!("Failed to append to file: {}", e))?;

        let mut output = HashMap::new();
        output.insert("path".to_string(), path);
        output.insert("status".to_string(), "appended".to_string());
        Ok(output)
    }
}
