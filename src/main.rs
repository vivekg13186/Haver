use log::{error, info};

use serde_json::Value;
use std::error::Error;
use std::fs::read_to_string;
use std::vec::Vec;
use std::{env, thread::sleep};

use rhai::{Engine, Scope};

mod step_handlers;
mod utils;
mod workflow;

use step_handlers::*;
use utils::*;
use workflow::*;

use job_scheduler::{Job, JobScheduler};
use std::time::Duration;

fn handle_step(step: &Step, engine: &mut Engine, scope: &mut Scope) -> Result<i32, Box<dyn Error>> {
    match step {
        Step::Start { next, schedule: _ } => {
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
            let print_msg = engine.eval_with_scope::<String>(scope, &message.as_str())?;
            info!("{}", print_msg);
            Ok(*next)
        }
        Step::ReadFile {
            path,
            next,
            content,
            error,
        } => handle_read_file(path, *next, content, error, engine, scope),
        Step::WriteFile {
            path,
            content,
            next,
            status,
            error,
        } => handle_write_file(path, content, *next, status, error, engine, scope),
    Step::AppendFile {
        path,
        content,
        next,
        status,
        error,
    } => handle_append_file(path, content, *next, status, error, engine, scope),
        Step::DeleteFile {
            path,
            next,
            status,
            error,
        } => handle_delete_file(path, *next, status, error, engine, scope),
        Step::Http {
            url,
            method,
            headers,
            query,
            body,
            auth,
            next,
            status,
            response,
        } => handle_http(
            url, method, headers, query, body, auth, *next, status, response, engine, scope,
        ),
    }
}

fn schedule_workflow(workflow: &Workflow, schedule: &String) {
    let mut job_sch = JobScheduler::new();
    job_sch.add(Job::new(schedule.parse().unwrap(), || {
        run_workflow(workflow).expect("Error running workflow");
    }));

    loop {
        job_sch.tick();
        sleep(Duration::from_millis(500));
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_condtion_workflow() {
        handle_file("./sample/condition.json");
    }

    
}

fn handle_file(file_path:&str){
 let result = read_to_string(file_path).expect("File not found");
    let workflow: Workflow = serde_json::from_str(&result).expect("Unable to parse JSON");
    match workflow.steps.get(0) {
        Some(Step::Start { next: _, schedule }) => match schedule {
            Some(schedule) => schedule_workflow(&workflow, schedule),
            None => run_workflow(&workflow).expect("Error running workflow"),
        },
        _ => panic!("First step to be type of start"),
    }
}
fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("expect work flow filename as input");
    }
    
    handle_file(&args[1]);
     
}
