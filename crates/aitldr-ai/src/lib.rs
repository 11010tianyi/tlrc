pub mod config;

use config::AiConfig;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("No API key for provider: {0}")]
    MissingKey(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub type Result<T> = std::result::Result<T, AiError>;

pub async fn generate_page(command: &str, config: &AiConfig) -> Result<String> {
    match config.provider.as_str() {
        "deepseek" => generate_deepseek(command, config).await,
        "openai" => generate_openai(command, config).await,
        "ollama" => generate_ollama(command, config).await,
        _ => Err(AiError::MissingKey(config.provider.clone())),
    }
}

async fn generate_deepseek(command: &str, config: &AiConfig) -> Result<String> {
    let api_key = config.deepseek_key.as_ref().ok_or_else(|| AiError::MissingKey("deepseek".to_string()))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let lang_instruction = if config.language == "zh" {
        "用中文输出命令说明和示例描述"
    } else {
        "Output command descriptions in English"
    };

    let not_found_marker = format!("# {command}\n\n> Command not found, may be a typo.");
    let prompt = format!(
        "Generate a TLDR page for command: {command}\n\n\
        CRITICAL: If unsure if command exists, output \"{not_found_marker}\"\n\n\
        Requirements:\n\
        - Follow tldr-pages format\n\
        - Maximum 8 examples\n\
        - Use concise wording\n\
        - {lang_instruction}\n\n\
        Format:\n\
        # {command}\n\n\
        > Brief description\n\n\
        - Example description:\n\
        `{{{{{{arg1}}}}}}`\n\n\
        Generate only markdown, no other text."
    );

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 800,
        }))
        .send()
        .await?;

    let data: serde_json::Value = response.json().await?;
    data["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| AiError::InvalidResponse("No content".to_string()))
}

async fn generate_openai(_command: &str, _config: &AiConfig) -> Result<String> {
    Err(AiError::MissingKey("openai".to_string()))
}

/// Generate a shell command from natural language query
pub async fn generate_command_from_natural_language(query: &str, config: &AiConfig) -> Result<String> {
    match config.provider.as_str() {
        "deepseek" => generate_command_deepseek(query, config).await,
        "openai" => generate_command_openai(query, config).await,
        _ => Err(AiError::MissingKey(config.provider.clone())),
    }
}

async fn generate_command_deepseek(query: &str, config: &AiConfig) -> Result<String> {
    let api_key = config.deepseek_key.as_ref().ok_or_else(|| AiError::MissingKey("deepseek".to_string()))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let lang_instruction = if config.language == "zh" {
        "用中文回答"
    } else {
        "Answer in English"
    };
    let system_prompt = format!(
        "You are a command-line expert. Generate the exact shell command that answers the user's request. Output only the command, no explanation. {lang_instruction}"
    );

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": query}
            ],
            "temperature": 0.1,
            "max_tokens": 200,
        }))
        .send()
        .await?;

    let data: serde_json::Value = response.json().await?;
    data["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| AiError::InvalidResponse("No content".to_string()))
}

async fn generate_command_openai(_query: &str, _config: &AiConfig) -> Result<String> {
    Err(AiError::MissingKey("openai".to_string()))
}

/// Generate explanation for a command
pub async fn generate_command_explanation(command: &str, config: &AiConfig) -> Result<String> {
    match config.provider.as_str() {
        "deepseek" => generate_explanation_deepseek(command, config).await,
        "openai" => generate_explanation_openai(command, config).await,
        _ => Err(AiError::MissingKey(config.provider.clone())),
    }
}

async fn generate_explanation_deepseek(command: &str, config: &AiConfig) -> Result<String> {
    let api_key = config.deepseek_key.as_ref().ok_or_else(|| AiError::MissingKey("deepseek".to_string()))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let (system_prompt, user_prompt) = if config.language == "zh" {
        (
            "你是一个命令行专家。简洁地解释给定的shell命令。说明每个命令的作用以及它们如何协同工作。用中文回答。".to_string(),
            format!("解释这个命令: {command}")
        )
    } else {
        (
            "You are a command-line expert. Explain the given shell command concisely. Include what each command does and how they work together.".to_string(),
            format!("Explain this command: {command}")
        )
    };

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 500,
        }))
        .send()
        .await?;

    let data: serde_json::Value = response.json().await?;
    data["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| AiError::InvalidResponse("No content".to_string()))
}

async fn generate_explanation_openai(_command: &str, _config: &AiConfig) -> Result<String> {
    Err(AiError::MissingKey("openai".to_string()))
}

// Ollama support

async fn generate_ollama(command: &str, config: &AiConfig) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let lang_instruction = if config.language == "zh" {
        "用中文输出命令说明和示例描述"
    } else {
        "Output command descriptions in English"
    };
    let prompt = format!(
        "Generate a TLDR page for command: {command}\n\n\
        CRITICAL: If unsure if command exists, output \"# {command}\\n\\n> Command not found, may be a typo.\"\n\n\
        Requirements:\n- Follow tldr-pages format\n- Maximum 8 examples\n- {lang_instruction}\n\n\
        Generate only markdown, no other text."
    );

    let response = client
        .post(format!("{}/api/generate", config.ollama_endpoint))
        .json(&serde_json::json!({
            "model": config.ollama_model,
            "prompt": prompt,
            "stream": false,
            "options": {"temperature": 0.3, "num_predict": 800}
        }))
        .send()
        .await?;

    let data: serde_json::Value = response.json().await?;
    data.get("response")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .ok_or_else(|| AiError::InvalidResponse("No content from Ollama".to_string()))
}

async fn generate_command_ollama(_query: &str, _config: &AiConfig) -> Result<String> {
    Err(AiError::MissingKey("ollama (command generation not implemented)".to_string()))
}

async fn generate_explanation_ollama(_command: &str, _config: &AiConfig) -> Result<String> {
    Err(AiError::MissingKey("ollama (explanation not implemented)".to_string()))
}
