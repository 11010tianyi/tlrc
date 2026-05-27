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
