use log::{error, info};
use rhai::{Engine, Scope};
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use crate::utils::set_string_var;
use crate::workflow::BasicAuth;
 

pub fn handle_read_file(
    path: &str,
    next: i32,
    content: &str,
    error: &str,
    engine: &Engine,
    scope: &mut Scope,
) -> Result<i32, Box<dyn Error>> {
    let path_value = engine.eval_with_scope::<String>(scope, path)?;
    match std::fs::read_to_string(&path_value) {
        Ok(file_content) => {
            info!("Read file ok: {}", path_value);
            set_string_var(content, &file_content, scope);
            Ok(next)
        }
        Err(e) => {
            error!("Failed to read file: {}", e);
            set_string_var(error, &e.to_string(), scope);
            Ok(next)
        }
    }
}

pub fn handle_write_file(
    path: &str,
    content: &str,
    next: i32,
    success: &str,
    error: &str,
    engine: &Engine,
    scope: &mut Scope,
) -> Result<i32, Box<dyn Error>> {
    let path_value = engine.eval_with_scope::<String>(scope, path)?;
    let content_value = engine.eval_with_scope::<String>(scope, content)?;
    match std::fs::write(&path_value, content_value) {
        Ok(_) => {
            info!("Wrote file ok: {}", path_value);
            set_string_var(success, "true", scope);
            set_string_var(error, "", scope);
        }
        Err(e) => {
            error!("Failed to write file: {}", e);
            set_string_var(success, "false", scope);
            set_string_var(error, &e.to_string(), scope);
        }
    }
    Ok(next)
}

pub fn handle_append_file(
    path: &str,
    content: &str,
    next: i32,
    success: &str,
    error: &str,
    engine: &Engine,
    scope: &mut Scope,
) -> Result<i32, Box<dyn Error>> {
    let path_value = engine.eval_with_scope::<String>(scope, path)?;
    let content_value = engine.eval_with_scope::<String>(scope, content)?;
    match OpenOptions::new().append(true).create(true).open(&path_value) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(content_value.as_bytes()) {
                error!("Failed to append to file: {}", e);
                set_string_var(success, "false", scope);
                set_string_var(error, &e.to_string(), scope);
            } else {
                info!("Appended to file ok: {}", path_value);
                set_string_var(success, "true", scope);
                set_string_var(error, "", scope);
            }
        }
        Err(e) => {
            error!("Failed to open file for append: {}", e);
            set_string_var(success, "false", scope);
            set_string_var(error, &e.to_string(), scope);
        }
    }
    Ok(next)
}

pub fn handle_delete_file(
    path: &str,
    next: i32,
    success: &str,
    error: &str,
    engine: &Engine,
    scope: &mut Scope,
) -> Result<i32, Box<dyn Error>> {
    let path_value = engine.eval_with_scope::<String>(scope, path)?;
    match std::fs::remove_file(&path_value) {
        Ok(_) => {
            info!("Deleted file ok: {}", path_value);
            set_string_var(success, "true", scope);
            set_string_var(error, "", scope);
        }
        Err(e) => {
            error!("Failed to delete file: {}", e);
            set_string_var(success, "false", scope);
            set_string_var(error, &e.to_string(), scope);
        }
    }
    Ok(next)
}

pub fn handle_http(
    url: &str,
    method: &str,
    headers: &Option<HashMap<String, String>>,
    query: &Option<HashMap<String, String>>,
    body: &Option<String>,
    auth: &Option<BasicAuth>,
    next: i32,
    status: &str,
    response: &str,
    engine: &Engine,
    scope: &mut Scope,
) -> Result<i32, Box<dyn Error>> {
    let url_value = engine.eval_with_scope::<String>(scope, url)?;
    let method_value = engine.eval_with_scope::<String>(scope, method)?;
    let client = reqwest::blocking::Client::new();

    let mut req_builder = match method_value.to_uppercase().as_str() {
        "GET" => client.get(&url_value),
        "POST" => client.post(&url_value),
        "PUT" => client.put(&url_value),
        "DELETE" => client.delete(&url_value),
        "PATCH" => client.patch(&url_value),
        _ => {
            set_string_var(status, "invalid_method", scope);
            set_string_var(response, "", scope);
            return Ok(next);
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
        let body_value = engine.eval_with_scope::<String>(scope, b)?;
        req_builder = req_builder.body(body_value);
    }

    if let Some(auth) = auth {
        req_builder = req_builder.basic_auth(&auth.username, Some(&auth.password));
    }

    match req_builder.send() {
        Ok(resp) => {
            let status_code = resp.status().as_u16().to_string();
            let resp_text = resp.text().unwrap_or_default();
            set_string_var(status, &status_code, scope);
            set_string_var(response, &resp_text, scope);
        }
        Err(e) => {
            set_string_var(status, "error", scope);
            set_string_var(response, &e.to_string(), scope);
        }
    }

    Ok(next)
}
