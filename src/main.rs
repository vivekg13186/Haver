use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::read_to_string;
use std::vec::Vec;

use rhai::{Engine, Scope};
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
struct BasicAuth {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Condition {
    exp: String,
    next: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Step {
    Start {
        next: i32,
    },
    End {},
    Condition {
        conditions: Vec<Condition>,
    },
    Log {
        message: String,
        next: i32,
    },
    ReadFile {
        path: String,
        next: i32,
        content: String,
        error: String,
    },
    WriteFile {
        path: String,
        content: String,
        next: i32,
        sucess: String,
        error: String,
    },
    AppendFile {
        path: String,
        content: String,
        next: i32,
        sucess: String,
        error: String,
    },
    DeleteFile {
        path: String,
        next: i32,
        sucess: String,
        error: String,
    },
    Http {
        url: String,
        method: String,
        headers: Option<HashMap<String, String>>,
        query: Option<HashMap<String, String>>,
        body: Option<String>,
        auth: Option<BasicAuth>,
        next: i32,
        status: String,
        response: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct Workflow {
    name: String,
    inputs: HashMap<String, Value>,
    steps: Vec<Step>,
}

fn set_string_var(name: &str, value: &str, scope: &mut Scope) {
    scope.set_value(name, value.to_string());
}
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
            info!(
                "{}",
                engine.eval_with_scope::<String>(scope, message.as_str())?
            );
            return Ok(*next);
        }

        Step::ReadFile {
            path,
            next,
            content,
            error,
        } => {
            let path_value = engine.eval_with_scope::<String>(scope, path.as_str())?;
            match std::fs::read_to_string(path_value) {
                Ok(file_content) => {
                    info!("Read file ok : {}", path);
                    set_string_var(content.as_str(), file_content.as_str(), scope);
                    return Ok(*next);
                }
                Err(e) => {
                    error!("Failed to read file: {}", e);
                    set_string_var(error.as_str(), e.to_string().as_str(), scope);
                    return Ok(*next);
                }
            }
        }
        Step::WriteFile {
            path,
            content,
            next,
            sucess,
            error,
        } => {
            let path_value = engine.eval_with_scope::<String>(scope, path.as_str())?;
            let content_value = engine.eval_with_scope::<String>(scope, content.as_str())?;
            match std::fs::write(path_value, content_value) {
                Ok(_) => {
                    info!("Wrote file ok : {}", path);
                    set_string_var(sucess.as_str(), "true", scope);
                    set_string_var(error.as_str(), "", scope);
                    return Ok(*next);
                }
                Err(e) => {
                    error!("Failed to write file: {}", e);
                    set_string_var(sucess.as_str(), "false", scope);
                    set_string_var(error.as_str(), e.to_string().as_str(), scope);
                    return Ok(*next);
                }
            }
        }
        Step::AppendFile {
            path,
            content,
            next,
            sucess,
            error,
        } => {
            let path_value = engine.eval_with_scope::<String>(scope, path.as_str())?;
            let content_value = engine.eval_with_scope::<String>(scope, content.as_str())?;
            match OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path_value)
            {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(content_value.as_bytes()) {
                        error!("Failed to append to file: {}", e);
                        set_string_var(sucess.as_str(), "false", scope);
                        set_string_var(error.as_str(), e.to_string().as_str(), scope);
                    } else {
                        info!("Appended to file ok : {}", path);
                        set_string_var(sucess.as_str(), "true", scope);
                        set_string_var(error.as_str(), "", scope);
                    }
                    Ok(*next)
                }
                Err(e) => {
                    error!("Failed to open file for append: {}", e);
                    set_string_var(sucess.as_str(), "false", scope);
                    set_string_var(error.as_str(), e.to_string().as_str(), scope);
                    Ok(*next)
                }
            }
        }
        Step::DeleteFile {
            path,
            next,
            sucess,
            error,
        } => {
            let path_value = engine.eval_with_scope::<String>(scope, path.as_str())?;
            match std::fs::remove_file(&path_value) {
                Ok(_) => {
                    info!("Deleted file ok : {}", path);
                    set_string_var(sucess.as_str(), "true", scope);
                    set_string_var(error.as_str(), "", scope);
                    Ok(*next)
                }
                Err(e) => {
                    error!("Failed to delete file: {}", e);
                    set_string_var(sucess.as_str(), "false", scope);
                    set_string_var(error.as_str(), e.to_string().as_str(), scope);
                    Ok(*next)
                }
            }
        }
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
        } => {
            let url_value = engine.eval_with_scope::<String>(scope, url.as_str())?;
            let method_value = engine.eval_with_scope::<String>(scope, method.as_str())?;
            let client = reqwest::blocking::Client::new();

            let mut req_builder = match method_value.to_uppercase().as_str() {
                "GET" => client.get(&url_value),
                "POST" => client.post(&url_value),
                "PUT" => client.put(&url_value),
                "DELETE" => client.delete(&url_value),
                "PATCH" => client.patch(&url_value),
                _ => {
                    set_string_var(status.as_str(), "invalid_method", scope);
                    set_string_var(response.as_str(), "", scope);
                    return Ok(*next);
                }
            };

            if let Some(hdrs) = headers {
                let mut req_headers = reqwest::header::HeaderMap::new();
                for (k, v) in hdrs {
                    req_headers.insert(
                        reqwest::header::HeaderName::from_bytes(k.as_bytes())?,
                        reqwest::header::HeaderValue::from_str(v)?,
                    );
                }
                req_builder = req_builder.headers(req_headers);
            }

            if let Some(q) = query {
                req_builder = req_builder.query(&q);
            }

            if let Some(b) = body {
                let body_value = engine.eval_with_scope::<String>(scope, b.as_str())?;
                req_builder = req_builder.body(body_value);
            }

            if let Some(auth) = auth {
                req_builder = req_builder.basic_auth(&auth.username, Some(&auth.password));
            }

            match req_builder.send() {
                Ok(resp) => {
                    let status_code = resp.status().as_u16().to_string();
                    let resp_text = resp.text().unwrap_or_else(|_| "".to_string());
                    set_string_var(status.as_str(), &status_code, scope);
                    set_string_var(response.as_str(), &resp_text, scope);
                }
                Err(e) => {
                    set_string_var(status.as_str(), "error", scope);
                    set_string_var(response.as_str(), &e.to_string(), scope);
                }
            }
            Ok(*next)
        }
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
