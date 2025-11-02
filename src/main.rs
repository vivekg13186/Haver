use boa_engine::{Context, Source};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::read_to_string;
mod commands;
use commands::register_default_commands;
use std::env;
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Step {
    Start {
        next: String,
    },
    End {},
    Condition {
        conditions: HashMap<String, String>,
    },
    Command {
        command: String,
        next: String,
        inputs: HashMap<String, String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct Workflow {
    name: String,
    inputs: HashMap<String, Value>,
    steps: HashMap<String, Step>,
}

// ---- Emit structured JSON event ----
fn emit_event(event_type: &str, step_name: &str, message: &str, step_id: u64) {
    let event = json!({
        "id": step_id,
        "event": event_type,
        "step": step_name,
        "message": message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    println!("{}", serde_json::to_string_pretty(&event).unwrap());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("expect work flow filename as input");
    }
    let command_registry = register_default_commands();
    // ---- Load and parse workflow ----
    let result = read_to_string(&args[1]).expect("File not found");
    let workflow: Workflow = serde_json::from_str(&result).expect("Unable to parse JSON");

    // ---- Find start step ----
    let (start_name, _) = workflow
        .steps
        .iter()
        .find(|(_, v)| matches!(v, Step::Start { .. }))
        .expect("No start step found");

    let mut step_name = start_name.clone();

    let mut context = Context::default();

    // Load workflow inputs into JS context
    for (key, value) in &workflow.inputs {
        let js_value = match value {
            Value::Null => "undefined".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            _ => continue, // skip arrays/objects for now
        };
        let script = format!("let {} = {};", key, js_value);
        context.eval(Source::from_bytes(&script)).unwrap();
    }

    let mut step_counter: u64 = 1;

    // ---- Main workflow loop ----
    loop {
        let current_step = match workflow.steps.get(&step_name) {
            Some(s) => s,
            None => {
                emit_event(
                    "error",
                    &step_name,
                    "Step not found in workflow",
                    step_counter,
                );
                break;
            }
        };

        match current_step {
            Step::Start { next } => {
                emit_event("start", &step_name, "Start step complete", step_counter);
                step_counter += 1;
                step_name = next.clone();
            }

            Step::End {} => {
                emit_event("end", &step_name, "Workflow finished", step_counter);
                break;
            }

            Step::Command {
                command,
                next,
                inputs,
            } => {
                emit_event(
                    "start",
                    &step_name,
                    "Command execution started",
                    step_counter,
                );
                if let Some(cmd) = command_registry.get(command) {
                    match cmd.execute(inputs,&mut context, &step_name, step_counter) {
                        Ok(result_map) => {
                            for (key, value) in result_map {
                                let escaped_value = value.replace('"', "\\\"");
                                let script = format!("let {} = \"{}\";", key, escaped_value);
                                context.eval(Source::from_bytes(&script)).unwrap();
                            }
                        }
                        Err(err) => {
                            emit_event(
                                "error",
                                &step_name,
                                &format!("Command failed: {}", err),
                                step_counter,
                            );
                            break;
                        }
                    }
                } else {
                    eprintln!("⚠️ Unknown command: {}", command);
                    break;
                }
                emit_event(
                    "end",
                    &step_name,
                    "Command executed successfully",
                    step_counter,
                );
                step_name = next.clone();
            }
            Step::Condition { conditions } => {
                let mut matched = false;
                for (cond, nxt) in conditions {
                    let result = if cond == "else" {
                        true
                    } else {
                        match context.eval(Source::from_bytes(cond)) {
                            Ok(value) => value.as_boolean().unwrap_or(false),
                            Err(_) => {
                                emit_event(
                                    "error",
                                    &step_name,
                                    &format!("Invalid condition: {}", cond),
                                    step_counter,
                                );
                                false
                            }
                        }
                    };
                    if result {
                        emit_event(
                            "end",
                            &step_name,
                            &format!("Condition matched: {}", cond),
                            step_counter,
                        );
                        step_counter += 1;
                        step_name = nxt.clone();
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    emit_event("error", &step_name, "No condition matched", step_counter);
                    break;
                }
            }
        }
    }
}
