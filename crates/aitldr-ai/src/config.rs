use dirs::home_dir;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub provider: String,
    pub model: String,
    pub deepseek_key: Option<String>,
    pub openai_key: Option<String>,
    pub ollama_endpoint: String,
    pub ollama_model: String,
    pub language: String,
    pub explain_default: bool,
    pub cache_enabled: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: "deepseek".to_string(),
            model: "deepseek-chat".to_string(),
            deepseek_key: None,
            openai_key: None,
            ollama_endpoint: "http://localhost:11434".to_string(),
            ollama_model: "qwen2:7b".to_string(),
            language: "zh".to_string(),
            explain_default: false,
            cache_enabled: true,
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    home_dir().unwrap_or_default().join(".aitldr")
}

pub fn load_config() -> AiConfig {
    let config_path = get_config_dir().join("config.toml");

    if !config_path.exists() {
        return AiConfig::default();
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return AiConfig::default(),
    };

    parse_config(&content)
}

fn parse_config(content: &str) -> AiConfig {
    let mut cfg = AiConfig::default();

    if let Some(v) = extract_value(content, "model", "provider") { cfg.provider = v; }
    if let Some(v) = extract_value(content, "model", "model") { cfg.model = v; }
    if let Some(v) = extract_value(content, "general", "language") { cfg.language = v; }
    if let Some(v) = extract_value(content, "general", "explain_default") { cfg.explain_default = v == "true"; }
    if let Some(v) = extract_value(content, "general", "cache_enabled") { cfg.cache_enabled = v != "false"; }
    cfg.deepseek_key = extract_value(content, "deepseek", "api_key");
    cfg.openai_key = extract_value(content, "openai", "api_key");
    if let Some(v) = extract_value(content, "ollama", "endpoint") { cfg.ollama_endpoint = v; }
    if let Some(v) = extract_value(content, "ollama", "model") { cfg.ollama_model = v; }

    cfg
}

pub fn save_config(cfg: &AiConfig) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir();
    fs::create_dir_all(&config_dir)?;

    let deepseek_key = cfg.deepseek_key.as_deref().unwrap_or("env:DEEPSEEK_API_KEY");
    let openai_key = cfg.openai_key.as_deref().unwrap_or("env:OPENAI_API_KEY");

    let content = format!(
        "[general]\nexplain_default = {}\ncache_enabled = {}\nlanguage = \"{}\"\n\n\
        [model]\nprovider = \"{}\"\nmodel = \"{}\"\n\n\
        [openai]\napi_key = \"{}\"\n\n\
        [deepseek]\napi_key = \"{}\"\n\n\
        [ollama]\nendpoint = \"{}\"\nmodel = \"{}\"\n",
        cfg.explain_default, cfg.cache_enabled, cfg.language,
        cfg.provider, cfg.model,
        openai_key, deepseek_key,
        cfg.ollama_endpoint, cfg.ollama_model,
    );

    fs::write(config_dir.join("config.toml"), content)?;
    Ok(())
}

fn extract_value(content: &str, section: &str, key: &str) -> Option<String> {
    let section_header = format!("[{section}]");
    let section_start = content.find(&section_header)?;
    let after_header = &content[section_start + section_header.len()..];
    let section_end = after_header.find("\n[").map(|i| section_start + section_header.len() + i).unwrap_or(content.len());

    let section_content = &content[section_start..section_end];
    let key_prefix = format!("{key} =");
    let key_line = section_content.lines().find(|line| line.trim().starts_with(&key_prefix))?;
    let value = key_line.trim_start_matches(&key_prefix).trim().trim_matches('"').to_string();

    if value.is_empty() {
        return None;
    }

    if let Some(env_var) = value.strip_prefix("env:") {
        std::env::var(env_var).ok()
    } else {
        Some(value)
    }
}
