use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::StreamExt;
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;

use crate::proxy::mappers::codex;
use crate::proxy::server::AppState;

pub async fn handle_responses(
    State(_state): State<AppState>,
    _headers: axum::http::HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("");

    let provider = match crate::proxy::providers::get_provider("codex") {
        Some(p) => p,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, "Codex provider not configured").into_response()
        }
    };

    let req = crate::proxy::providers::ProviderRequest {
        model: model.to_string(),
        body: body.clone(),
        account_id: String::new(),
    };

    match provider.send_request(req).await {
        Ok(resp) => {
            (StatusCode::from_u16(resp.status).unwrap_or(StatusCode::OK), Json(resp.body)).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_GATEWAY, format!("Codex proxy error: {}", e)).into_response()
        }
    }
}

pub async fn handle_responses_stream(
    State(_state): State<AppState>,
    headers: axum::http::HeaderMap,
    body: String,
) -> impl IntoResponse {
    let parsed: Value = serde_json::from_str(&body).unwrap_or(json!({}));
    let model = parsed.get("model").and_then(|v| v.as_str()).unwrap_or("gpt-4o");

    let api_key = match extract_codex_api_key(&headers, &parsed) {
        Some(k) => k,
        None => {
            return (StatusCode::UNAUTHORIZED, "Missing API key").into_response();
        }
    };

    let chat_body = match codex::codex_response_to_chat(&parsed) {
        Ok(mut b) => {
            b["stream"] = json!(true);
            b["stream_options"] = json!({"include_usage": true});
            b
        }
        Err(_) => {
            let mut b = parsed.clone();
            b["stream"] = json!(true);
            b
        }
    };

    let client = reqwest::Client::new();
    let upstream_resp = match client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&api_key)
        .json(&chat_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::BAD_GATEWAY, format!("Upstream request failed: {}", e)).into_response();
        }
    };

    if !upstream_resp.status().is_success() {
        let status = upstream_resp.status();
        let text = upstream_resp.text().await.unwrap_or_default();
        return (StatusCode::BAD_GATEWAY, format!("Upstream error ({}): {}", status, text)).into_response();
    }

    let stream = upstream_resp.bytes_stream();
    let transformed = async_stream::stream! {
        let mut buffer = Vec::new();
        let mut rng = rand::thread_rng();
        let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let random_id: String = (0..24).map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset.chars().nth(idx).unwrap()
        }).collect();
        let response_id = format!("resp-{}", random_id);
        let item_id = format!("item-{}", &random_id[..16]);
        let mut started = false;
        let mut completed = false;

        tokio::pin!(stream);
        loop {
            match stream.next().await {
                Some(Ok(chunk)) => {
                    buffer.extend_from_slice(&chunk);
                    while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                        let line: Vec<u8> = buffer.drain(..=pos).collect();
                        let line_str = String::from_utf8_lossy(&line);
                        let trimmed = line_str.trim();
                        if trimmed.is_empty() || !trimmed.starts_with("data: ") {
                            continue;
                        }
                        let data = trimmed.trim_start_matches("data: ").trim();
                        if data == "[DONE]" {
                            if !completed {
                                completed = true;
                                let done_ev = json!({
                                    "type": "response.output_item.done",
                                    "item": { "id": &item_id, "type": "message" }
                                });
                                yield Ok::<bytes::Bytes, String>(bytes::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&done_ev).unwrap())));
                                let completed_ev = json!({
                                    "type": "response.completed",
                                    "response": {
                                        "id": &response_id,
                                        "object": "response",
                                        "status": "completed",
                                        "output": [{ "id": &item_id, "type": "message" }]
                                    }
                                });
                                yield Ok::<bytes::Bytes, String>(bytes::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&completed_ev).unwrap())));
                            }
                            continue;
                        }
                        if let Ok(chat_chunk) = serde_json::from_str::<Value>(data) {
                            if !started {
                                started = true;
                                let created_ev = json!({
                                    "type": "response.created",
                                    "response": { "id": &response_id, "object": "response", "status": "in_progress", "output": [] }
                                });
                                yield Ok::<bytes::Bytes, String>(bytes::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&created_ev).unwrap())));
                                let item_added = json!({
                                    "type": "response.output_item.added",
                                    "output_index": 0,
                                    "item": { "id": &item_id, "type": "message", "role": "assistant", "status": "in_progress", "content": [] }
                                });
                                yield Ok::<bytes::Bytes, String>(bytes::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&item_added).unwrap())));
                                let part_added = json!({
                                    "type": "response.content_part.added",
                                    "item_id": &item_id,
                                    "output_index": 0,
                                    "content_index": 0,
                                    "part": { "type": "output_text", "text": "" }
                                });
                                yield Ok::<bytes::Bytes, String>(bytes::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&part_added).unwrap())));
                            }
                            if let Some(choices) = chat_chunk.get("choices").and_then(|c| c.as_array()) {
                                for choice in choices {
                                    if let Some(delta) = choice.get("delta") {
                                        if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                            if !content.is_empty() {
                                                let delta_ev = json!({
                                                    "type": "response.output_text.delta",
                                                    "item_id": &item_id,
                                                    "output_index": 0,
                                                    "content_index": 0,
                                                    "delta": content
                                                });
                                                yield Ok::<bytes::Bytes, String>(bytes::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&delta_ev).unwrap())));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    let err_ev = json!({ "type": "error", "error": { "message": format!("Stream error: {}", e) } });
                    yield Ok::<bytes::Bytes, String>(bytes::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&err_ev).unwrap())));
                    break;
                }
                None => break,
            }
        }
        if !completed {
            let done_ev = json!({
                "type": "response.output_item.done",
                "item": { "id": &item_id, "type": "message" }
            });
            yield Ok::<bytes::Bytes, String>(bytes::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&done_ev).unwrap())));
            let completed_ev = json!({
                "type": "response.completed",
                "response": { "id": &response_id, "object": "response", "status": "completed", "output": [] }
            });
            yield Ok::<bytes::Bytes, String>(bytes::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&completed_ev).unwrap())));
        }
    };

    use axum::body::Body;
    use axum::response::Response;
    let body = Body::from_stream(transformed);
    (StatusCode::OK, [("Content-Type", "text/event-stream"), ("Cache-Control", "no-cache"), ("Connection", "keep-alive")], body).into_response()
}

fn extract_codex_api_key(headers: &axum::http::HeaderMap, body: &Value) -> Option<String> {
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(key) = auth.strip_prefix("Bearer ") {
            return Some(key.to_string());
        }
    }
    if let Some(key) = body.get("api_key").and_then(|v| v.as_str()) {
        return Some(key.to_string());
    }
    None
}
