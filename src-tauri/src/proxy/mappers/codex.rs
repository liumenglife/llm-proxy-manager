/// 将 Codex Responses API 请求体转换为 OpenAI Chat API 格式
pub fn codex_response_to_chat(body: &serde_json::Value) -> Result<serde_json::Value, String> {
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("");

    let messages = body.get("input")
        .and_then(|v| v.as_str())
        .map(|text| {
            serde_json::json!([{"role": "user", "content": text}])
        })
        .or_else(|| body.get("input").and_then(|v| v.as_array()).map(|a| serde_json::Value::Array(a.clone())))
        .unwrap_or_default();

    let mut chat_body = serde_json::json!({
        "model": model,
        "messages": messages,
    });

    if let Some(instructions) = body.get("instructions").and_then(|v| v.as_str()) {
        if let Some(obj) = chat_body.as_object_mut() {
            obj.insert("system".to_string(), serde_json::Value::String(instructions.to_string()));
        }
    }

    Ok(chat_body)
}

/// 将 OpenAI Chat API 响应转换为 Codex Responses API 格式
pub fn chat_to_codex_response(body: &serde_json::Value) -> Result<serde_json::Value, String> {
    let mut output: Vec<serde_json::Value> = Vec::new();

    if let Some(choices) = body.get("choices").and_then(|v| v.as_array()) {
        for choice in choices {
            if let Some(msg) = choice.get("message") {
                let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("assistant");

                let content_part = serde_json::json!({
                    "type": "text",
                    "text": content
                });

                output.push(serde_json::json!({
                    "id": "msg_001",
                    "role": role,
                    "content": [content_part]
                }));
            }
        }
    }

    Ok(serde_json::json!({
        "id": body.get("id").and_then(|v| v.as_str()).unwrap_or("resp_001"),
        "object": "response",
        "model": body.get("model"),
        "output": output,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_response_to_chat_basic() {
        let input = serde_json::json!({
            "model": "GPT-5.1",
            "input": "Hello world",
            "instructions": "Be helpful"
        });

        let result = codex_response_to_chat(&input).unwrap();
        assert_eq!(result.get("model").unwrap(), "GPT-5.1");
        assert_eq!(result.get("system").unwrap(), "Be helpful");
    }

    #[test]
    fn test_codex_response_to_chat_without_instructions() {
        let input = serde_json::json!({
            "model": "GPT-5.1",
            "input": "Hello world"
        });

        let result = codex_response_to_chat(&input).unwrap();
        assert_eq!(result.get("model").unwrap(), "GPT-5.1");
        assert!(result.get("system").is_none());
    }

    #[test]
    fn test_codex_response_to_chat_with_array_input() {
        let input = serde_json::json!({
            "model": "GPT-5.1",
            "input": [
                {"role": "user", "content": "Hello"},
                {"role": "assistant", "content": "Hi"}
            ]
        });

        let result = codex_response_to_chat(&input).unwrap();
        let messages = result.get("messages").and_then(|v| v.as_array()).unwrap();
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_chat_to_codex_response_basic() {
        let input = serde_json::json!({
            "id": "chatcmpl-123",
            "model": "GPT-5.1",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Hello back"
                }
            }]
        });

        let result = chat_to_codex_response(&input).unwrap();
        assert_eq!(result.get("object").unwrap(), "response");
        assert!(result.get("output").and_then(|v| v.as_array()).is_some());
    }

    #[test]
    fn test_chat_to_codex_response_empty_choices() {
        let input = serde_json::json!({
            "id": "chatcmpl-123",
            "model": "GPT-5.1",
            "choices": []
        });

        let result = chat_to_codex_response(&input).unwrap();
        let output = result.get("output").and_then(|v| v.as_array()).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_chat_to_codex_response_multi_choice() {
        let input = serde_json::json!({
            "id": "chatcmpl-123",
            "model": "GPT-5.1",
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "First response"
                    }
                },
                {
                    "message": {
                        "role": "assistant",
                        "content": "Second response"
                    }
                }
            ]
        });

        let result = chat_to_codex_response(&input).unwrap();
        let output = result.get("output").and_then(|v| v.as_array()).unwrap();
        assert_eq!(output.len(), 2);
    }
}
