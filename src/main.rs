use log::{error, info};
 
use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs::read_to_string;
use std::vec::Vec;

use rhai::{Engine, Scope};

mod workflow;
mod utils;
mod step_handlers;

use workflow::*;
use utils::*;
use step_handlers::*;

 

 
fn handle_step(step: &Step, engine: &mut Engine, scope: &mut Scope) -> Result<i32, Box<dyn Error>> {
    match step {
       Step::Start { next } => {
            info!("Starting workflow");
            Ok(*next)
        }
        Step::End {} => {
            info!("Ending workflow");
            return Ok(-1);
        }
        Step::Condition { conditions } => {
            info!("executing condition");
            for condition in conditions {
                if engine.eval_with_scope::<bool>(scope, condition.exp.as_str())? {
                    info!("matched condition {}", condition.exp);
                    return Ok(condition.next);
                }
            }
            return Ok(-1);
        }
        Step::Log { message, next } => {
            let print_msg =engine.eval_with_scope::<String>(scope, &message.as_str())?;
            info!("{}",print_msg);
            Ok(*next) 
        },
        Step::ReadFile { path, next, content, error } => handle_read_file(path, *next, content, error, engine, scope),
        Step::WriteFile { path, content, next, sucess, error } => handle_write_file(path, content, *next, sucess, error, engine, scope),
        Step::AppendFile { path, content, next, sucess, error } => handle_append_file(path, content, *next, sucess, error, engine, scope),
        Step::DeleteFile { path, next, sucess, error } => handle_delete_file(path, *next, sucess, error, engine, scope),
        Step::Http { url, method, headers, query, body, auth, next, status, response } =>
            handle_http(url, method, headers, query, body, auth, *next, status, response, engine, scope),
    }
}

 
fn run_workflow(workflow: &Workflow) -> Result<(), Box<dyn Error>> {
    let mut engine = Engine::new();
    let mut scope: Scope = Scope::new();
    let mut current_step = 0;

    for (key, val) in &workflow.inputs {
        match val {
            Value::String(s) => {
                set_string_var(key, s, &mut scope);
            }
            Value::Bool(b) => {
                set_string_var(key, &b.to_string(), &mut scope);
            }
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    set_string_var(key, &f.to_string(), &mut scope);
                }
            }
            Value::Null => {
                set_string_var(key, "", &mut scope); // or "null" if you prefer
            }
            _ => {
                // Optionally log or handle unsupported types like arrays or objects
                error!("Unsupported input type for key '{}': {:?}", key, val);
            }
        }
    }

    while current_step != -1 {
        if current_step < 0 || current_step as usize >= workflow.steps.len() {
            error!("Invalid step index: {}", current_step);
            break;
        }
        let step = &workflow.steps[current_step as usize];
        current_step = handle_step(step, &mut engine, &mut scope)?;
    }
    Ok(())
}
fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("expect work flow filename as input");
    }
    let result = read_to_string(&args[1]).expect("File not found");
    let workflow: Workflow = serde_json::from_str(&result).expect("Unable to parse JSON");
    run_workflow(&workflow).expect("Workflow execution failed");
}
