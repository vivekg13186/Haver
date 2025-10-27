use rhai::{Dynamic, Engine, Scope};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::{read_to_string};
mod commands; 
use commands::{register_default_commands};
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Step {
    Start { next: String },
    End {},
    Condition { conditions: HashMap<String, String> },
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
// ---- Convert JSON -> Rhai Dynamic ----
fn json_value_to_rhai(value: &Value) -> Dynamic {
    match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::Number(n) => n
            .as_f64()
            .map(Dynamic::from)
            .unwrap_or_else(|| Dynamic::UNIT),
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Array(arr) => Dynamic::from_array(
            arr.iter().map(|v| json_value_to_rhai(v)).collect(),
        ),
        Value::Object(obj) => {
            let mut map = rhai::Map::new();
            for (k, v) in obj {
                map.insert(k.into(), json_value_to_rhai(v));
            }
            Dynamic::from_map(map)
        }
    }
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
    let command_registry = register_default_commands();
    // ---- Load and parse workflow ----
    let result = read_to_string("./src/sample/workflow2.json")
        .expect("File not found");
    let workflow: Workflow =
        serde_json::from_str(&result).expect("Unable to parse JSON");

    // ---- Find start step ----
    let (start_name, _) = workflow
        .steps
        .iter()
        .find(|(_, v)| matches!(v, Step::Start { .. }))
        .expect("No start step found");

    let mut step_name = start_name.clone();
    let engine = Engine::new();
    let mut scope = Scope::new();

    // ---- Load workflow inputs into scope ----
    for (key, value) in &workflow.inputs {
        scope.push_dynamic(key.clone(), json_value_to_rhai(value));
    }

    let mut step_counter: u64 = 1;

    // ---- Main workflow loop ----
    loop {
        let current_step = match workflow.steps.get(&step_name) {
            Some(s) => s,
            None => {
                emit_event("error", &step_name, "Step not found in workflow", step_counter);
                break;
            }
        };

        emit_event("start", &step_name, "Step execution started", step_counter);

        match current_step {
            Step::Start { next } => {
                emit_event("end", &step_name, "Start step complete", step_counter);
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
                if let Some(cmd) = command_registry.get(command) {
                    match cmd.execute(inputs, &mut scope, &engine, &step_name, step_counter) {
                        Ok(msg) => println!("✅ {}", msg),
                        Err(err) => {
                            eprintln!("❌ Error in {}: {}", command, err);
                            break;
                        }
                    }
                } else {
                    eprintln!("⚠️ Unknown command: {}", command);
                    break;
                }

                step_name = next.clone();
            }
            Step::Condition { conditions } => {
                let mut matched = false;
                for (cond, nxt) in conditions {
                    let result = if cond == "else" {
                        true
                    } else {
                        match engine.eval_with_scope::<bool>(&mut scope, cond) {
                            Ok(v) => v,
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
