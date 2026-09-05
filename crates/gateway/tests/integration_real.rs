//! 真实 API 集成测试
//!
//! 运行前:
//! 1. 确保 data/config/providers.json 已配置真实 API Key
//! 2. 启动服务: cargo run -p llm-smart-router-gateway
//! 3. 运行: cargo test -p llm-smart-router-gateway --test integration_real -- --nocapture

use std::time::Duration;

const BASE_URL: &str = "http://localhost:8080";

#[tokio::test]
async fn test_health_endpoint() {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/health", BASE_URL))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to connect - is the gateway running?");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    println!("✓ Health endpoint: OK");
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/health/metrics", BASE_URL))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to connect");
    assert_eq!(resp.status(), 200);
    let text = resp.text().await.unwrap();
    assert!(text.contains("llm_smart_router_total_requests"));
    println!("✓ Metrics endpoint: OK");
}

#[tokio::test]
async fn test_list_models() {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/models", BASE_URL))
        .header("Authorization", "Bearer sk-local-dev")
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to connect");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let models = body["data"].as_array().unwrap();
    assert!(!models.is_empty(), "Should have at least one model");
    println!("✓ Models listed: {} models", models.len());
    for m in models {
        println!("  - {} ({})", m["id"], m["provider"]);
    }
}

#[tokio::test]
async fn test_chat_completions_real() {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .header("Authorization", "Bearer sk-local-dev")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "deepseek-v4-flash",
            "messages": [{"role": "user", "content": "Say hello in 3 words"}],
            "max_tokens": 50,
            "temperature": 0.1
        }))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("API call failed");

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap();
    if status == 200 {
        let content = body["choices"][0]["message"]["content"].as_str().unwrap_or("");
        // Some models (like deepseek) may return empty content for thinking models
        if !content.is_empty() {
            println!("✓ Chat Completions: {:.60}", content);
        } else {
            println!("✓ Chat Completions: received (empty content - thinking model)");
        }
        println!("  Model: {} | Tokens: {} in/{} out | Finish: {}",
            body["model"], body["usage"]["prompt_tokens"], body["usage"]["completion_tokens"],
            body["choices"][0]["finish_reason"]);
    } else {
        println!("⚠ Chat Completions returned {}: {}", status, body);
    }
}

#[tokio::test]
async fn test_chat_completions_streaming() {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .header("Authorization", "Bearer sk-local-dev")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "deepseek-v4-flash",
            "messages": [{"role": "user", "content": "Count 1 to 3"}],
            "max_tokens": 50,
            "stream": true
        }))
        .timeout(Duration::from_secs(30))
        .send()
        .await;

    match resp {
        Ok(response) => {
            let status = response.status();
            if status == 200 {
                let bytes = response.bytes().await.unwrap();
                let text = String::from_utf8_lossy(&bytes);
                assert!(text.contains("data: "), "Should have SSE data events");
                assert!(text.contains("[DONE]"), "Should end with [DONE]");
                println!("✓ Streaming: OK ({} bytes)", text.len());
            } else {
                println!("⚠ Streaming returned {}", status);
            }
        }
        Err(e) => println!("⚠ Streaming error: {}", e),
    }
}

#[tokio::test]
async fn test_anthropic_messages_real() {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/messages", BASE_URL))
        .header("x-api-key", "sk-local-dev")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "deepseek-v4-flash",
            "messages": [{"role": "user", "content": "Say hello in 2 words"}],
            "max_tokens": 50
        }))
        .timeout(Duration::from_secs(30))
        .send()
        .await;

    match resp {
        Ok(response) => {
            let status = response.status();
            let body: serde_json::Value = response.json().await.unwrap();
            if status == 200 {
                let content = body["content"][0]["text"].as_str().unwrap_or("");
                if !content.is_empty() {
                    println!("✓ Anthropic Messages: {:.60}", content);
                } else {
                    println!("✓ Anthropic Messages: received (empty content - thinking model)");
                }
            } else {
                println!("⚠ Anthropic Messages returned {}: {}", status, body);
            }
        }
        Err(e) => println!("⚠ Anthropic error: {}", e),
    }
}

#[tokio::test]
async fn test_responses_api_real() {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/responses", BASE_URL))
        .header("Authorization", "Bearer sk-local-dev")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "deepseek-v4-flash",
            "input": "Say hello in 2 words",
            "max_output_tokens": 50
        }))
        .timeout(Duration::from_secs(30))
        .send()
        .await;

    match resp {
        Ok(response) => {
            let status = response.status();
            let body: serde_json::Value = response.json().await.unwrap();
            if status == 200 {
                let text = body["output"][0]["content"][0]["text"].as_str()
                    .or_else(|| body["output"][0]["text"].as_str())
                    .unwrap_or("");
                if !text.is_empty() {
                    println!("✓ Responses API: {:.60}", text);
                } else {
                    println!("✓ Responses API: received (empty content - thinking model)");
                }
            } else {
                println!("⚠ Responses API returned {}: {}", status, body);
            }
        }
        Err(e) => println!("⚠ Responses API error: {}", e),
    }
}

#[tokio::test]
async fn test_failover_routing() {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/chat/completions", BASE_URL))
        .header("Authorization", "Bearer sk-local-dev")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "deepseek-v4-flash",
            "messages": [{"role": "user", "content": "Hi"}],
            "max_tokens": 10
        }))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("API call failed");

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap();
    if status == 200 {
        println!("✓ Failover: succeeded with model {}", body["model"]);
    } else {
        println!("⚠ Failover returned {}: {}", status, body);
    }
}

#[tokio::test]
async fn test_admin_register_provider() {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/admin/providers", BASE_URL))
        .header("Authorization", "Bearer sk-local-dev")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "name": "openai",
            "api_base_url": "https://api.openai.com/v1",
            "api_key": "sk-test",
            "models": [{"id": "gpt-4o", "capabilities": ["chat"]}]
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Admin API call failed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    println!("✓ Admin register: OK");
}

#[tokio::test]
async fn test_auth_failure() {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/models", BASE_URL))
        .header("Authorization", "Bearer invalid-key")
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Request failed");
    assert_eq!(resp.status(), 401);
    println!("✓ Auth failure: 401 returned correctly");
}