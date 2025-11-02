use crate::commands::WorkflowCommand;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Method;
use boa_engine::{Context, Source};
use std::collections::HashMap;

pub struct RestApiCommand;

impl WorkflowCommand for RestApiCommand {
    fn name(&self) -> &'static str {
        "RestApi"
    }

    fn execute(
        &self,
        inputs: &HashMap<String, String>,
        context: &mut Context,
        _step_name: &str,
        _step_id: u64,
    ) -> Result<HashMap<String, String>, String> {
        // Required fields
        let method_expr = inputs.get("method").ok_or("Missing 'method' input")?;
        let url_expr = inputs.get("url").ok_or("Missing 'url' input")?;

        // Optional fields
        let body_expr = inputs.get("body");
        let headers_expr = inputs.get("headers");
        let query_expr = inputs.get("query");

        // Evaluate method and URL
        let method_str = context
            .eval(Source::from(method_expr))
            .map_err(|e| format!("Failed to evaluate method: {}", e))?
            .to_string(context)
            .map_err(|e| format!("Failed to convert method to string: {}", e))?;

        let url = context
            .eval(Source::from(url_expr))
            .map_err(|e| format!("Failed to evaluate URL: {}", e))?
            .to_string(context)
            .map_err(|e| format!("Failed to convert URL to string: {}", e))?;

        let method = match method_str.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            other => return Err(format!("Unsupported HTTP method: {}", other)),
        };

        let client = Client::new();
        let mut request = client.request(method.clone(), &url);

        // Add query parameters
        if let Some(expr) = query_expr {
            let query_result = context
                .eval(Source::from(expr))
                .map_err(|e| format!("Failed to evaluate query params: {}", e))?;

            let query_obj = query_result
                .as_object()
                .ok_or("Query expression must evaluate to an object")?;

            let mut query_map = HashMap::new();
            for (k, v) in query_obj.properties().iter() {
                let key = k.to_string();
                let value = v.value().to_string(context).unwrap_or_default();
                query_map.insert(key, value);
            }

            request = request.query(&query_map);
        }

        // Add headers
        if let Some(expr) = headers_expr {
            let headers_result = context
                .eval(Source::from(expr))
                .map_err(|e| format!("Failed to evaluate headers: {}", e))?;

            let headers_obj = headers_result
                .as_object()
                .ok_or("Headers expression must evaluate to an object")?;

            let mut header_map = HeaderMap::new();
            for (k, v) in headers_obj.properties().iter() {
                let key = k.to_string();
                let value = v.value().to_string(context).unwrap_or_default();

                let header_name = HeaderName::from_bytes(key.as_bytes())
                    .map_err(|e| format!("Invalid header name: {}", e))?;
                let header_value = value
                    .parse()
                    .map_err(|e| format!("Invalid header value: {}", e))?;

                header_map.insert(header_name, header_value);
            }

            request = request.headers(header_map);
        }

        // Add body for POST
        if method == Method::POST {
            let body = match body_expr {
                Some(expr) => context
                    .eval(Source::from_bytes(expr))
                    .map_err(|e| format!("Failed to evaluate body: {}", e))?
                    .to_string(context)
                    .map_err(|e| format!("Failed to convert body to string: {}", e))?,
                None => "".to_string().into(),
            };
            request = request.body(body);
        }

        // Send request
        let response = request
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        let status = response.status().as_u16().to_string();
        let text = response
            .text()
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        let mut output = HashMap::new();
        output.insert("status".to_string(), status);
        output.insert("response".to_string(), text);
        Ok(output)
    }
}
