use rhai::{Dynamic, Engine, Scope};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::read_to_string;
use std::{collections::HashMap, process::Command};

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

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    name: String,
    start_time: String,
    end_time: String,
    status: String,
    error: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct WorkflowOutput {
    logs: Vec<String>,
    tasks: Vec<Task>,
    start_time: String,
    end_time: String,
}

// same helper function from above
fn json_value_to_rhai(value: &Value) -> Result<Dynamic, Box<rhai::EvalAltResult>> {
    Ok(match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::UNIT
            }
        }
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Array(arr) => {
            let vec: Vec<Dynamic> = arr
                .iter()
                .map(|v| json_value_to_rhai(v).unwrap_or(Dynamic::UNIT))
                .collect();
            Dynamic::from_array(vec)
        }
        Value::Object(obj) => {
            let mut map = rhai::Map::new();
            for (k, v) in obj {
                map.insert(k.into(), json_value_to_rhai(v)?);
            }
            Dynamic::from_map(map)
        }
    })
}

fn main() {
    let result = read_to_string("./src/sample/workflow1.json").expect("File not found");
    let json: Workflow = serde_json::from_str(&result).expect("Unable to parse json");

    //dbg!(&json);
    let start_step = json
        .steps
        .iter()
        .find(|(_, value)| match value {
            Step::Start { next: _ } => true,
            _ => false,
        })
        .expect("Start step not found");
    //dbg!(&start_step);
    let mut step_name = start_step.0;
    //dbg!(&step_name);
    let mut current_step = start_step.1;
    //dbg!(&current_step);
    let engine = Engine::new();
    let mut scope = Scope::new();
    for (key, value) in &json.inputs {
        scope.push_dynamic(
            key.clone(),
            json_value_to_rhai(value).expect("cannot parse given input"),
        );
    }
    loop {
        //print!("Executing : {}\n",step_name);
        match current_step {
            Step::Start { next: nxt } => {
                step_name = nxt;
                match json.steps.get(step_name) {
                    Some(step) => current_step = step,
                    None => {
                        print!("no next step found");
                        break;
                    }
                }
            }
            Step::End {} => {
                break;
            }
            Step::Command {
                command,
                next,
                inputs,
            } => {
                match command.as_str() {
                    "Log" => {
                        let message_exp = inputs
                            .get("message")
                            .expect("print command missing message input");
                        let message = engine
                            .eval_with_scope::<String>(&mut scope, &message_exp)
                            .expect("error eval print statement");
                        print!("{}\n", message);
                    }
                    _ => {
                        print!("unknown command {}", command)
                    }
                }
                step_name = next;
                current_step = json.steps.get(step_name).expect("Next step not found");
            }
            Step::Condition { conditions } => {
                let mut match_condition = false;
                for (condition, next) in conditions {
                    let result = if condition == "else" {
                        true
                    } else {
                        engine
                            .eval_with_scope::<bool>(&mut scope, &condition)
                            .expect("error eval condition")
                    };
                    if result {
                        step_name = next;
                        current_step = json.steps.get(step_name).expect("Next step not found");
                        match_condition = true;
                    }
                }
                if !match_condition {
                    break;
                }
            }
        }
    }
}
