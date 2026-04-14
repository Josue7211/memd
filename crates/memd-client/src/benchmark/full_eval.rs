use super::*;
use anyhow::Context;
use serde_json::Value as JsonValue;
use std::time::Duration;

/// Build a generation prompt for LongMemEval / LoCoMo.
pub(crate) fn build_generation_prompt(question: &str, retrieved_context: &str) -> String {
    format!(
        "You are a helpful assistant with access to the user's conversation history.\n\n\
         Relevant conversation history:\n{retrieved_context}\n\n\
         Question: {question}\n\n\
         Answer the question based only on the conversation history above. \
         If the information is not available, say so. Be concise."
    )
}

/// Build a generation prompt for MemBench multiple-choice questions.
pub(crate) fn build_mc_generation_prompt(
    question: &str,
    retrieved_context: &str,
    choices: &[String],
) -> String {
    let choices_text = choices.join("\n");
    format!(
        "You are a helpful assistant with access to the user's conversation history.\n\n\
         Relevant conversation history:\n{retrieved_context}\n\n\
         Question: {question}\n\n\
         Choices:\n{choices_text}\n\n\
         Select the correct answer. Reply with ONLY the choice letter (A, B, C, etc.)."
    )
}

/// Call an OpenAI-compatible API to generate a response.
pub(crate) async fn call_generator(
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
) -> anyhow::Result<GeneratorResponse> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(15))
        .build()
        .context("build generator client")?;
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    eprintln!("[generator] POST {url} model={model}");
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "n": 1,
            "temperature": 0,
            "max_tokens": 512
        }))
        .send()
        .await
        .context("send generator request")?;
    eprintln!("[generator] response status={}", response.status());
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("generator request failed with {status}: {body}");
    }
    let body = response
        .json::<JsonValue>()
        .await
        .context("parse generator response")?;
    let content = body
        .get("choices")
        .and_then(JsonValue::as_array)
        .and_then(|c| c.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(JsonValue::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let usage = body.get("usage").cloned();
    let prompt_tokens = usage
        .as_ref()
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    let completion_tokens = usage
        .as_ref()
        .and_then(|u| u.get("completion_tokens"))
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);
    Ok(GeneratorResponse {
        content,
        prompt_tokens,
        completion_tokens,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct GeneratorResponse {
    pub content: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

/// Resolve generator configuration from CLI args + env vars.
pub(crate) fn resolve_generator_config(
    args: &PublicBenchmarkArgs,
) -> anyhow::Result<GeneratorConfig> {
    let api_key = std::env::var("OPENAI_API_KEY").context(
        "--full-eval requires OPENAI_API_KEY (or ANTHROPIC_API_KEY with --generator-model claude-*)",
    )?;
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model = args
        .generator_model
        .as_deref()
        .unwrap_or("gpt-4o-mini")
        .to_string();
    let grader_model = args
        .grader_model
        .as_deref()
        .unwrap_or("gpt-4o-2024-08-06")
        .to_string();
    Ok(GeneratorConfig {
        base_url,
        api_key,
        model,
        grader_model,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct GeneratorConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub grader_model: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_generation_prompt_includes_context_and_question() {
        let prompt = build_generation_prompt("What color is the sky?", "User said: the sky is blue");
        assert!(prompt.contains("What color is the sky?"));
        assert!(prompt.contains("the sky is blue"));
    }

    #[test]
    fn build_mc_generation_prompt_includes_choices() {
        let choices = vec!["A. Red".to_string(), "B. Blue".to_string()];
        let prompt = build_mc_generation_prompt("What color?", "sky is blue", &choices);
        assert!(prompt.contains("A. Red"));
        assert!(prompt.contains("B. Blue"));
    }
}
