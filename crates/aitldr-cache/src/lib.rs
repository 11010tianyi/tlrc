use std::path::PathBuf;
use dirs::home_dir;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("No home directory")]
    NoHome,
}

pub fn get_ai_cache_dir() -> Result<PathBuf, CacheError> {
    let home = home_dir().ok_or(CacheError::NoHome)?;
    let dir = home.join(".aitldr/ai");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn get_nl_cache_dir() -> Result<PathBuf, CacheError> {
    let home = home_dir().ok_or(CacheError::NoHome)?;
    let dir = home.join(".aitldr/nl");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn get_explain_cache_dir() -> Result<PathBuf, CacheError> {
    let home = home_dir().ok_or(CacheError::NoHome)?;
    let dir = home.join(".aitldr/explain");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

pub fn save_page(cmd: &str, content: &str) -> Result<(), CacheError> {
    let path = get_ai_cache_dir()?.join(format!("{}.md", cmd));
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn load_page(cmd: &str) -> Result<Option<String>, CacheError> {
    let path = get_ai_cache_dir()?.join(format!("{}.md", cmd));
    Ok(path.exists().then(|| std::fs::read_to_string(&path).ok()).flatten())
}

pub fn delete_page(cmd: &str) -> Result<(), CacheError> {
    let path = get_ai_cache_dir()?.join(format!("{}.md", cmd));
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn save_nl_page(query: &str, command: &str) -> Result<(), CacheError> {
    let dir = get_nl_cache_dir()?;
    let filename = format!("{}.txt", sanitize_filename(query));
    std::fs::write(dir.join(&filename), command)?;
    Ok(())
}

pub fn load_nl_page(query: &str) -> Result<Option<String>, CacheError> {
    let dir = get_nl_cache_dir()?;
    let filename = format!("{}.txt", sanitize_filename(query));
    let path = dir.join(&filename);
    Ok(path.exists().then(|| std::fs::read_to_string(&path).ok()).flatten())
}

pub fn save_explain_page(cmd: &str, explanation: &str) -> Result<(), CacheError> {
    let path = get_explain_cache_dir()?.join(format!("{}.txt", sanitize_filename(cmd)));
    std::fs::write(&path, explanation)?;
    Ok(())
}

pub fn load_explain_page(cmd: &str) -> Result<Option<String>, CacheError> {
    let path = get_explain_cache_dir()?.join(format!("{}.txt", sanitize_filename(cmd)));
    Ok(path.exists().then(|| std::fs::read_to_string(&path).ok()).flatten())
}

pub fn delete_explain_page(cmd: &str) -> Result<(), CacheError> {
    let path = get_explain_cache_dir()?.join(format!("{}.txt", sanitize_filename(cmd)));
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

// Bioinformatics community pages

pub fn get_bio_command_dir() -> Result<PathBuf, CacheError> {
    let home = home_dir().ok_or(CacheError::NoHome)?;
    let dir = home.join(".aitldr/bio-command");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn get_bio_format_dir() -> Result<PathBuf, CacheError> {
    let home = home_dir().ok_or(CacheError::NoHome)?;
    let dir = home.join(".aitldr/bio-format");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Look up a command in bio-command pages, returns (content, filename) if found
pub fn find_bio_command(command: &str) -> Result<Option<(String, String)>, CacheError> {
    let dir = get_bio_command_dir()?;
    // Try exact match: samtools.md, blast-plus.md
    let exact = dir.join(format!("{}.md", command));
    if let Ok(content) = std::fs::read_to_string(&exact) {
        let name = exact.file_name().unwrap().to_string_lossy().to_string();
        return Ok(Some((content, name)));
    }
    // Try fuzzy: search for files containing the command name
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") {
                let stem = name.trim_end_matches(".md");
                if stem == command
                    || stem.replace('-', "") == command.replace('-', "")
                    || command.contains(stem)
                    || stem.contains(command)
                {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        return Ok(Some((content, name)));
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Look up a format in bio-format pages
pub fn find_bio_format(fmt: &str) -> Result<Option<(String, String)>, CacheError> {
    let dir = get_bio_format_dir()?;
    let exact = dir.join(format!("{}.md", fmt));
    if let Ok(content) = std::fs::read_to_string(&exact) {
        let name = exact.file_name().unwrap().to_string_lossy().to_string();
        return Ok(Some((content, name)));
    }
    Ok(None)
}

/// Remove all cached bio pages so they can be re-downloaded
pub fn clean_bio_pages() {
    if let Some(home) = dirs::home_dir() {
        let _ = std::fs::remove_dir_all(home.join(".aitldr/bio-command"));
        let _ = std::fs::remove_dir_all(home.join(".aitldr/bio-format"));
    }
}

/// Download bioinformatics community pages from GitHub
pub fn download_bio_pages() -> Result<(), CacheError> {
    let bio_cmd_dir = get_bio_command_dir()?;
    let bio_fmt_dir = get_bio_format_dir()?;

    // Use git clone if available, otherwise skip
    let status = std::process::Command::new("git")
        .args([
            "clone",
            "--depth=1",
            "https://github.com/11010tianyi/bioinformatics-command.git",
        ])
        .env("GIT_TERMINAL_PROMPT", "0")
        .current_dir(std::env::temp_dir())
        .status();

    let tmp_repo = std::env::temp_dir().join("bioinformatics-command");

    match status {
        Ok(s) if s.success() => {
            // Copy bio-command pages
            let src_cmd = tmp_repo.join("bio-command");
            if src_cmd.exists() {
                if let Ok(entries) = std::fs::read_dir(&src_cmd) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "md").unwrap_or(false) {
                            let dest = bio_cmd_dir.join(path.file_name().unwrap());
                            let _ = std::fs::copy(&path, &dest);
                        }
                    }
                }
            }
            // Copy bio-format pages
            let src_fmt = tmp_repo.join("bio-format");
            if src_fmt.exists() {
                if let Ok(entries) = std::fs::read_dir(&src_fmt) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "md").unwrap_or(false) {
                            let dest = bio_fmt_dir.join(path.file_name().unwrap());
                            let _ = std::fs::copy(&path, &dest);
                        }
                    }
                }
            }
            // Clean up clone
            let _ = std::fs::remove_dir_all(&tmp_repo);
            Ok(())
        }
        _ => {
            // git not available or clone failed - clean up
            let _ = std::fs::remove_dir_all(&tmp_repo);
            Err(CacheError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to clone bioinformatics-command repository",
            )))
        }
    }
}
