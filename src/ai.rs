use std::sync::Mutex;
use crate::error::{Error, Result};
use aitldr_ai::{self, AiConfig};
use aitldr_cache as cache_module;
use aitldr_risk as risk_module;

lazy_static::lazy_static! {
    static ref AI_CACHE_DIR: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);
}

pub async fn fallback_generate(command: &str) -> Result<Option<String>> {
    // Check if command exists
    if !risk_module::command_exists(command) {
        log::warn!("Command '{}' not found on system, skipping AI generation", command);
        return Ok(None);
    }

    // Check cache
    if let Some(cached) = check_cache(command)? {
        log::info!("Using cached AI page for '{}'", command);
        return Ok(Some(cached));
    }

    // Generate with AI
    log::info!("Generating AI page for '{}'", command);
    let config = aitldr_ai::config::load_config();

    match aitldr_ai::generate_page(command, &config).await {
        Ok(content) => {
            // Save to cache
            if let Err(e) = save_cache(command, &content) {
                log::warn!("Failed to cache AI page: {}", e);
            }
            Ok(Some(content))
        }
        Err(e) => {
            log::error!("AI generation failed: {}", e);
            Ok(None)
        }
    }
}

fn check_cache(command: &str) -> Result<Option<String>> {
    match cache_module::load_page(command) {
        Ok(Some(content)) => Ok(Some(content)),
        Ok(None) => Ok(None),
        Err(e) => {
            log::warn!("Failed to read cache: {}", e);
            Ok(None)
        }
    }
}

fn save_cache(command: &str, content: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    cache_module::save_page(command, content)?;
    Ok(())
}

pub fn init_ai_cache_dir() {
    let mut dir = AI_CACHE_DIR.lock().unwrap();
    if dir.is_none() {
        *dir = Some(cache_module::get_ai_cache_dir().ok());
    }
}
