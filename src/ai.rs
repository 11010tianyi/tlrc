use crate::error::Result;
use aitldr_ai::config::load_config;
use aitldr_ai::generate_page;
use aitldr_cache as cache_module;
use aitldr_risk as risk_module;

/// Step 2: Check AI cache (equivalent to Python's get_ai_page)
pub fn check_ai_cache(command: &str) -> Result<Option<String>> {
    match cache_module::load_page(command) {
        Ok(Some(content)) => {
            log::info!("Using cached AI page for '{command}'");
            Ok(Some(content))
        }
        Ok(None) => Ok(None),
        Err(e) => {
            log::warn!("Failed to read AI cache: {e}");
            Ok(None)
        }
    }
}

/// Check natural language cache (query → command mapping)
pub fn check_nl_cache(query: &str) -> Result<Option<String>> {
    match cache_module::load_nl_page(query) {
        Ok(Some(command)) => {
            log::info!("Using cached NL result for '{query}'");
            Ok(Some(command))
        }
        Ok(None) => Ok(None),
        Err(e) => {
            log::warn!("Failed to read NL cache: {e}");
            Ok(None)
        }
    }
}

/// Save natural language query → command mapping to cache
pub fn save_nl_cache(query: &str, command: &str) {
    if let Err(e) = cache_module::save_nl_page(query, command) {
        log::warn!("Failed to save NL cache: {e}");
    }
}

/// Delete AI cache for --refresh
pub fn delete_ai_cache(command: &str) {
    if let Err(e) = cache_module::delete_page(command) {
        log::warn!("Failed to delete AI cache: {e}");
    }
    if let Err(e) = cache_module::delete_explain_page(command) {
        log::warn!("Failed to delete explain cache: {e}");
    }
}

/// Check explanation cache for a command
pub fn check_explain_cache(command: &str) -> Result<Option<String>> {
    match cache_module::load_explain_page(command) {
        Ok(Some(explanation)) => {
            log::info!("Using cached explanation for '{command}'");
            Ok(Some(explanation))
        }
        Ok(None) => Ok(None),
        Err(e) => {
            log::warn!("Failed to read explain cache: {e}");
            Ok(None)
        }
    }
}

/// Save explanation to cache
pub fn save_explain_cache(command: &str, explanation: &str) {
    if let Err(e) = cache_module::save_explain_page(command, explanation) {
        log::warn!("Failed to save explain cache: {e}");
    }
}

/// Step 3: AI generation with command_exists check (equivalent to Python's generate_page)
/// - Online mode: check command_exists first (prevent hallucination)
/// - Offline mode: skip command_exists check (allow forced generation)
pub async fn fallback_generate(command: &str, offline: bool) -> Result<Option<String>> {
    // In online mode, check command existence to prevent AI hallucination
    if !offline && !risk_module::command_exists(command) {
        log::warn!(
            "Command '{command}' not found on system, skipping AI generation"
        );
        return Ok(None);
    }

    log::info!("Generating AI page for '{command}'");
    let config = load_config();

    match generate_page(command, &config).await {
        Ok(content) => {
            if let Err(e) = cache_module::save_page(command, &content) {
                log::warn!("Failed to cache AI page: {e}");
            }
            Ok(Some(content))
        }
        Err(e) => {
            log::error!("AI generation failed: {e}");
            Ok(None)
        }
    }
}
