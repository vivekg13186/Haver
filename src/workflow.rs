use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Condition {
    pub exp: String,
    pub next: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Step {
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
pub struct Workflow {
    pub name: String,
    pub inputs: HashMap<String, Value>,
    pub steps: Vec<Step>,
}
