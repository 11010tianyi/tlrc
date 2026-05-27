use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RiskError {
    #[error("Command failed")]
    Failed,
}

pub fn command_exists(cmd: &str) -> bool {
    if cmd.is_empty() { return false; }
    #[cfg(unix)]
    {
        Command::new("sh")
            .args(["-c", &format!("command -v {} 2>/dev/null || which {} 2>/dev/null", cmd, cmd)])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    { Command::new("where").arg(cmd).output().map(|o| o.status.success()).unwrap_or(false) }
    #[cfg(not(any(unix, windows)))]
    { true }
}

pub fn is_destructive(cmd: &str) -> bool {
    ["rm -rf", "dd", "mkfs", ":(){ :|:& };:"].iter().any(|p| cmd.contains(p))
}

pub fn is_natural_language(query: &str) -> bool {
    // Check for Chinese characters
    if query.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c)) {
        return true;
    }

    // Check for long sentences (more than 3 words with spaces)
    if query.split_whitespace().count() > 3 {
        return true;
    }

    // Check for natural language patterns
    let patterns = [
        "删除", "查看", "显示", "怎么", "如何", "创建", "新建", "搜索", "查找",
        "list", "show", "how to", "create", "new", "search", "find",
    ];
    let lower = query.to_lowercase();
    patterns.iter().any(|p| lower.contains(p))
}
