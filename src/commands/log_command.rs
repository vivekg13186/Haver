use std::collections::HashMap;
use crate::commands::WorkflowCommand;
use boa_engine::Context;
pub struct LogCommand;

impl WorkflowCommand for LogCommand {
    fn name(&self) -> &'static str {
        "Log"
    }

    fn execute(
        &self,
        inputs: &HashMap<String, String>,
  context : &mut Context,
        step_name: &str,
        step_id: u64,
    ) -> Result<HashMap<String, String>,String> {
        let expr = inputs.get("message").ok_or("Missing 'message' input")?;
        let msg = engine
            .eval_with_scope::<String>(scope, expr)
            .map_err(|e| format!("Failed to evaluate message: {}", e))?;

        println!("[LOG] Step {} (#{}): {}", step_name, step_id, msg);

        let mut output = HashMap::new();
        output.insert("message".to_string(), msg);
        Ok(output)
    }
}
