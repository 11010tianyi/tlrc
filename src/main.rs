mod ai;
mod args;
mod cache;
mod config;
mod error;
mod output;
mod util;

use std::process::ExitCode;

use clap::Parser;
use log::{info, warn};
use yansi::Paint;

use crate::ai::fallback_generate;
use crate::args::Cli;
use crate::cache::Cache;
use crate::config::{Config, OptionStyle, OutputMode};
use crate::error::{Error, Result};
use crate::output::PageRenderer;
use crate::util::{Logger, init_color};
use aitldr_risk::is_natural_language;

const DEFAULT_PLATFORM: &str = if cfg!(target_os = "linux") {
    "linux"
} else if cfg!(target_os = "macos") {
    "osx"
} else if cfg!(target_os = "windows") {
    "windows"
} else if cfg!(target_os = "freebsd") {
    "freebsd"
} else if cfg!(target_os = "openbsd") {
    "openbsd"
} else if cfg!(target_os = "netbsd") {
    "netbsd"
} else if cfg!(target_os = "android") {
    "android"
} else {
    "common"
};

fn main() -> ExitCode {
    let cli = Cli::parse();
    init_color(cli.color);
    Logger::init(cli.quiet, cli.verbose);

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    match rt.block_on(run(cli)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => e.exit_code(),
    }
}

fn include_cli_in_config(cfg: &mut Config, cli: &Cli) {
    if cli.compact {
        warn!(
            "--compact is deprecated.\nPlease use output.mode = \"very_compact\" in the config instead"
        );
        cfg.output.mode = OutputMode::VeryCompact;
    }
    if cli.no_compact {
        warn!(
            "--no-compact is deprecated.\nPlease use output.mode = \"normal\" in the config instead"
        );
        if cfg.output.mode == OutputMode::VeryCompact {
            cfg.output.mode = OutputMode::Normal;
        }
    }

    if cli.raw {
        cfg.output.mode = OutputMode::Raw;
    }
    if cli.no_raw {
        warn!("--no-raw is deprecated.\nPlease use output.mode = \"normal\" in the config instead");
        if cfg.output.mode == OutputMode::Raw {
            cfg.output.mode = OutputMode::Normal;
        }
    }

    cfg.output.edit_link |= cli.edit;

    match (cli.short_options, cli.long_options) {
        (false, false) => {}
        (true, true) => cfg.output.option_style = OptionStyle::Both,
        (true, false) => cfg.output.option_style = OptionStyle::Short,
        (false, true) => cfg.output.option_style = OptionStyle::Long,
    }
}

/// Handle `--init`: create ~/.aitldr/config.toml with defaults
fn handle_init() -> Result<()> {
    let config_path = aitldr_ai::config::get_config_dir().join("config.toml");
    if config_path.exists() {
        println!("{} Configuration already exists at {}", "!".yellow().bold(), config_path.display());
        println!("Edit it directly, or delete it first if you want to reset.");
        return Ok(());
    }
    let cfg = aitldr_ai::config::AiConfig::default();
    aitldr_ai::config::save_config(&cfg)
        .map_err(|e| Error::new(format!("Failed to save config: {e}")))?;
    let config_dir = aitldr_ai::config::get_config_dir();
    println!("{} Configuration initialized at {}", "✓".green().bold(), config_dir.display());
    println!("\nEdit config.toml to set your API keys and preferences.");
    Ok(())
}

/// Handle `--ai-status`: show current AI configuration
fn handle_ai_status() -> Result<()> {
    let cfg = aitldr_ai::config::load_config();
    let config_dir = aitldr_ai::config::get_config_dir();

    let ds_key_status = if cfg.deepseek_key.is_some() {
        "********************".green().to_string()
    } else {
        "Not configured".red().to_string()
    };
    let oai_key_status = if cfg.openai_key.is_some() {
        "********************".green().to_string()
    } else {
        "Not configured".red().to_string()
    };

    println!("{}", "aitldr Configuration".bold());
    println!();
    println!("  {}:", "General".bold());
    println!("    Explain default: {}", cfg.explain_default);
    println!("    Cache enabled:   {}", cfg.cache_enabled);
    println!("    Language:        {}", cfg.language);
    println!();
    println!("  {}:", "Model".bold());
    println!("    Provider: {}", cfg.provider);
    println!("    Model:    {}", cfg.model);
    println!();
    println!("  {}:", "DeepSeek".bold());
    println!("    API Key: {ds_key_status}");
    println!();
    println!("  {}:", "OpenAI".bold());
    println!("    API Key: {oai_key_status}");
    println!();
    println!("  {}:", "Ollama".bold());
    println!("    Endpoint: {}", cfg.ollama_endpoint);
    println!("    Model:    {}", cfg.ollama_model);
    println!();
    println!("  Config directory: {}", config_dir.display());

    Ok(())
}

/// Print explanation for a command, using cache unless skip_cache is true
async fn print_explanation(command: &str, ai_config: &aitldr_ai::config::AiConfig, skip_cache: bool) {
    if !skip_cache {
        if let Ok(Some(cached)) = crate::ai::check_explain_cache(command) {
            println!("\n{cached}");
            return;
        }
    }
    match aitldr_ai::generate_command_explanation(command, ai_config).await {
        Ok(explanation) => {
            crate::ai::save_explain_cache(command, &explanation);
            println!("\n{explanation}");
        }
        Err(e) => warn!("Failed to generate explanation: {e}"),
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Handle AI-specific commands first
    if cli.init {
        return handle_init();
    }
    if cli.ai_status {
        return handle_ai_status();
    }

    if cli.config_path {
        return Config::print_path();
    }

    if cli.gen_config {
        return Config::print_default();
    }

    let mut cfg = Config::new(cli.config.as_deref())?;
    include_cli_in_config(&mut cfg, &cli);

    if let Some(path) = cli.render {
        return PageRenderer::print(&path, &cfg);
    }

    // This is needed later to print a different error message if --language was used.
    let languages_are_from_cli = cli.languages.is_some();
    // We need to clone() because this vector will not be sorted,
    // unlike the one in the config.
    let languages = cli.languages.unwrap_or_else(|| cfg.cache.languages.clone());
    let cache = Cache::new(&cfg.cache.dir);

    if cli.clean_cache {
        return cache.clean();
    }

    if cli.update {
        // update() should never use languages from --language.
        return cache.update(&cfg.cache.mirror, &mut cfg.cache.languages);
    }

    let mut update_later = false;

    if !cache.subdir_exists(cache::ENGLISH_DIR) {
        if cli.offline {
            return Err(Error::offline_no_cache());
        }
        info!("cache is empty, downloading...");
        cache
            .update(&cfg.cache.mirror, &mut cfg.cache.languages)
            .map_err(|e| e.describe(Error::DESC_NO_INTERNET))?;
    } else if cfg.cache.auto_update
        && let age = cache.age()?
        && age > cfg.cache_max_age()
    {
        let age = util::duration_fmt(age.as_secs());
        let age = age.green().bold();

        if cli.offline {
            warn!("cache is stale (last update: {age} ago). Run tldr without --offline to update.");
        } else if cfg.cache.defer_auto_update {
            info!("cache is stale (last update: {age} ago), update has been deferred");
            update_later = true;
        } else {
            info!("cache is stale (last update: {age} ago), updating...");
            cache
                .update(&cfg.cache.mirror, &mut cfg.cache.languages)
                .map_err(|e| e.describe(Error::DESC_AUTO_UPDATE_ERR))?;
        }
    }

    let platform = match cli.platform.as_deref() {
        // "macos" should be an alias of "osx".
        // Since the `macos` directory doesn't exist, this has to be changed before it
        // gets passed to cache functions (which expect directory names).
        Some("macos") => "osx",
        Some(p) => p,
        None => DEFAULT_PLATFORM,
    };

    if cli.list {
        cache.list_for(platform)?;
    } else if cli.list_all {
        cache.list_all()?;
    } else if let Some(query) = cli.search {
        // All platforms should be searched when `-p` isn't used.
        let search_plat = cli.platform.as_deref().map(|_| platform);
        cache.search(&query, search_plat, &languages, languages_are_from_cli)?;
    } else if cli.info {
        cache.info(&cfg)?;
    } else if cli.list_platforms {
        cache.list_platforms()?;
    } else if cli.list_languages {
        cache.list_languages()?;
    } else {
        let raw_query = cli.page.join(" ");
        let page_name = cli.page.join("-").to_lowercase();

        // Load AI config and apply --model override
        let mut ai_config = aitldr_ai::config::load_config();
        if let Some(ref model) = cli.model {
            ai_config.provider = model.clone();
        }
        // Apply explain_default from config if --explain not explicitly set
        let explain = cli.explain || ai_config.explain_default;

        // Natural language mode: detect Chinese or natural language patterns
        if is_natural_language(&raw_query) {
            info!("Detected natural language query, generating command...");
            let skip_explain_cache = cli.refresh;
            // Check NL cache first
            if let Some(cached_cmd) = crate::ai::check_nl_cache(&raw_query)? {
                println!("{}", cached_cmd.bold());
                if explain {
                    print_explanation(&cached_cmd, &ai_config, skip_explain_cache).await;
                }
                if aitldr_risk::is_destructive(&cached_cmd) {
                    eprintln!("\n{} {}", "WARNING:".yellow().bold(), "Destructive command! This operation may cause irreversible data loss.".yellow());
                }
                return Ok(());
            }

            match aitldr_ai::generate_command_from_natural_language(&raw_query, &ai_config).await {
                Ok(command) if !command.is_empty() => {
                    // Save to NL cache
                    crate::ai::save_nl_cache(&raw_query, &command);
                    println!("{}", command.bold());
                    if explain {
                        print_explanation(&command, &ai_config, skip_explain_cache).await;
                    }
                    if aitldr_risk::is_destructive(&command) {
                        eprintln!("\n{} {}", "WARNING:".yellow().bold(), "Destructive command! This operation may cause irreversible data loss.".yellow());
                    }
                    return Ok(());
                }
                Ok(_) => {
                    warn!("AI could not generate a command for this query");
                }
                Err(e) => {
                    warn!("Natural language command generation failed: {e}");
                }
            }
            return Err(Error::new("could not generate a command from the given query."));
        }

        // Download bio pages on first use if not cached
        if !cli.offline {
            let bio_dir = dirs::home_dir()
                .map(|h| h.join(".aitldr/bio-command"))
                .filter(|d| d.exists());
            if bio_dir.is_none() {
                info!("Downloading bioinformatics community pages...");
                if let Err(e) = aitldr_cache::download_bio_pages() {
                    info!("Bio pages download skipped: {e}");
                }
            }
        }

        // --refresh: delete AI cache and regenerate
        if cli.refresh {
            crate::ai::delete_ai_cache(&page_name);
            info!("Refreshing AI page for '{page_name}'...");
        }

        let mut page_paths = cache.find(&page_name, &languages, platform)?;

        if update_later && page_paths.is_empty() {
            // Since the page hasn't been found and the cache is stale, disregard the defer option.
            warn!("page not found, updating now...");
            cache
                .update(&cfg.cache.mirror, &mut cfg.cache.languages)
                .map_err(|e| e.describe(Error::DESC_AUTO_UPDATE_ERR))?;
            page_paths = cache.find(&page_name, &languages, platform)?;
            // Reset the defer flag in order not to update twice.
            update_later = false;
        }

        if page_paths.is_empty() {
            // Step 2: Check bioinformatics community pages (bio-command & bio-format)
            if let Ok(Some((content, filename))) = aitldr_cache::find_bio_command(&page_name) {
                eprintln!("{}", format!("[Bioinformatics Community Page: {filename}]").dim());
                println!("{content}");
                return Ok(());
            }
            // Also check bio-format (for format names like bam, vcf, fasta)
            if let Ok(Some((content, filename))) = aitldr_cache::find_bio_format(&page_name) {
                eprintln!("{}", format!("[Bioinformatics Format Page: {filename}]").dim());
                println!("{content}");
                return Ok(());
            }

            // Step 3: Check AI cache (same as Python: get_ai_page)
            if let Some(cached) = crate::ai::check_ai_cache(&page_name)? {
                eprintln!("{}", "[AI Generated Page (cached)]".dim());
                let temp_path =
                    std::env::temp_dir().join(format!("tldr_ai_{page_name}.md"));
                std::fs::write(&temp_path, cached)?;
                PageRenderer::print_cache_result(&[temp_path], &cfg)?;
                eprintln!("{}", "\nAI-generated pages may contain inaccuracies. Verify before use.".dim());
                return Ok(());
            }

            // Step 3: AI generation (command_exists check happens inside, matching Python)
            match fallback_generate(&page_name, cli.offline).await {
                Ok(Some(ai_content)) => {
                    eprintln!("{}", "[AI Generated Page]".dim());
                    let temp_path =
                        std::env::temp_dir().join(format!("tldr_ai_{page_name}.md"));
                    std::fs::write(&temp_path, ai_content)?;
                    PageRenderer::print_cache_result(&[temp_path], &cfg)?;
                    eprintln!("{}", "\nAI-generated pages may contain inaccuracies. Verify before use.".dim());
                    return Ok(());
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("AI generation failed: {:?}", e);
                }
            }

            let e = Error::new("page not found.");
            return if languages_are_from_cli {
                Err(e.describe(Error::TRY_NO_EXPLICIT_LANGUAGE))
            } else {
                Err(e.describe(Error::desc_page_does_not_exist(cache.age()?)))
            };
        }

        PageRenderer::print_cache_result(&page_paths, &cfg)?;
    }

    if update_later {
        cache
            .update(&cfg.cache.mirror, &mut cfg.cache.languages)
            .map_err(|e| e.describe(Error::DESC_AUTO_UPDATE_ERR))?;
    }

    Ok(())
}
