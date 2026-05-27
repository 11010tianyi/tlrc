# From Python to Rust: aitldr Feature Migration

aitldr started as a Python CLI ([aitldr-cli](https://github.com/11010tianyi/aitldr-cli)) and was later rewritten as a Rust fork of the official [tlrc](https://github.com/tldr-pages/tlrc) client. This document tracks what carried over, what changed, and what was added.

## Why Rewrite?

The Python version was a standalone tool that implemented its own page lookup, rendering, and cache management. While functional, it could never match the official tldr client's rendering quality, platform support, or update mechanism.

The Rust version takes a different approach: **fork the official client, add AI as a fallback.** This means the official tldr experience is identical to upstream — AI only fills gaps.

## Feature Comparison

| Feature | Python | Rust | Status |
|---------|--------|------|--------|
| Official tldr page lookup | HTTP fetch per page | Upstream tlrc cache system | Replaced |
| Page rendering | Plain text | Upstream tlrc renderer | Replaced |
| AI page generation (DeepSeek) | Yes | Yes | Migrated |
| AI page generation (OpenAI) | Yes | Stub (planned) | Partial |
| AI page generation (Ollama) | Yes | Page only | Partial |
| Natural language queries | Yes | Yes (DeepSeek) | Migrated |
| Command explanation | Yes | Yes (DeepSeek) | Migrated |
| Destructive command warning | Yes | Yes | Migrated |
| Command existence check | Yes | Yes | Migrated |
| Smart caching | Yes | Yes (3-tier) | Migrated + Enhanced |
| Bioinformatics community pages | No | Yes (40+ tools, 25+ formats) | New |
| Page source badges | Disclaimer text | `[AI Generated Page]` badges | Improved |
| Rating system | `rate` command | Not implemented | Dropped |
| Submit guide | `submit` command | Not implemented | Dropped |
| Rich terminal output | `rich` library | Upstream tlrc + yansi | Replaced |
| Config management | Custom TOML | Custom TOML (compatible) | Migrated |
| Environment variable keys | `env:` prefix | `env:` prefix | Migrated |
| Chinese/English output | Yes | Yes | Migrated |
| Offline mode | Skips AI entirely | Skips cache update, AI still works | Improved |
| `--refresh` force regeneration | Yes | Yes (page + explain cache) | Migrated + Enhanced |
| `--model` override | Yes | Yes | Migrated |
| Platform detection | Custom | Upstream tlrc | Replaced |

## Detailed Changes

### Replaced: Official Page Handling

The Python version fetched individual pages via HTTP from `raw.githubusercontent.com`. This was slow, fragile, and didn't support platforms well.

The Rust version inherits upstream tlrc's complete page pipeline:
- Local cache with periodic updates
- Multi-platform page resolution (linux, osx, windows, common)
- Proper Markdown rendering with configurable styles
- Language-specific page support

**Impact**: Official pages render identically to upstream tlrc. No compromise.

### Replaced: Rendering and Output

| Python | Rust |
|--------|------|
| `rich` library for colored output | Upstream tlrc renderer with yansi |
| Plain text fallback | Full Markdown rendering |
| Custom formatting | Configurable style (colors, bold, etc.) |

### Migrated: AI Generation

All three AI backends were carried over:

**DeepSeek** — Full support in both versions:
- Endpoint: `https://api.deepseek.com/chat/completions`
- Default model: `deepseek-chat`
- Temperature: 0.3 (pages), 0.1 (NL), 0.3 (explain)
- Max tokens: 800 (pages), 200 (NL), 500 (explain)
- Hallucination prevention: "If unsure, output command not found marker"

**OpenAI** — Full in Python, stub in Rust:
- Python used `gpt-4o-mini` as default
- Rust has the config and dispatch, but API call is not yet implemented
- Planned for future release

**Ollama** — Page generation in both:
- Python: all three features (page, NL, explain)
- Rust: page generation only (NL and explain are stubs)
- Default model: `qwen2:7b`
- 60-second timeout for local inference

### Migrated: Natural Language Queries

The detection logic was rewritten in Rust:

```python
# Python
def is_natural_language(query):
    # Chinese character detection
    # Sentence length > 3 words
    # Pattern matching: "how to", "查找", etc.
```

```rust
// Rust
pub fn is_natural_language(query: &str) -> bool {
    // Chinese character detection (CJK unified ideographs)
    // Long sentence detection (> 3 words)
    // Keyword matching: "how to", "查找", etc.
}
```

Same detection strategy, same keywords. The Rust version uses Unicode character properties for CJK detection.

### Migrated: Destructive Command Warning

Same blacklist in both:

```
rm -rf, dd, mkfs, :(){ :|:& };:
```

### Migrated: Command Existence Check

Both versions use `command -v` / `which` to verify a command exists before asking AI to generate a page. This prevents hallucinated pages for nonexistent commands.

### Enhanced: Caching System

Python cached AI pages with metadata headers:

```
<!--
AI-Generated: true
Model: gpt-4o-mini
Generated-At: 2026-05-28
Confidence: medium
Sources: []
-->
```

Rust uses plain Markdown files (no metadata), but adds **three cache tiers**:

| Cache | Python | Rust |
|-------|--------|------|
| AI pages | `~/.aitldr/ai/*.md` | `~/.aitldr/ai/*.md` |
| NL results | Not cached | `~/.aitldr/nl/*.txt` |
| Explanations | Not cached | `~/.aitldr/explain/*.txt` |
| Bio-command | N/A | `~/.aitldr/bio-command/*.md` |
| Bio-format | N/A | `~/.aitldr/bio-format/*.md` |

Python only cached AI-generated pages. Rust caches natural language results and command explanations too, reducing API calls significantly.

### Enhanced: Offline Mode

| Python `--offline` | Rust `--offline` |
|--------------------|--------------------|
| Skips AI generation entirely | Skips cache updates only |
| No AI features available | AI generation still works |
| Must have local cache | Can generate new AI pages |

The Rust version is more useful offline because it only skips the tldr cache update, not AI features.

### Enhanced: `--refresh`

Python deleted the AI page cache only. Rust deletes **both** AI page cache and explanation cache before regenerating.

### New: Bioinformatics Community Pages

The biggest feature not in Python. Bio pages come from [bioinformatics-command](https://github.com/11010tianyi/bioinformatics-command):

- **40+ bio tools**: samtools, gatk, stringtie, bwa, hisat2, etc.
- **25+ bio formats**: bam, vcf, fasta, bed, gff, etc.
- Checked **before** AI generation (saves tokens)
- Auto-downloaded on first use
- Refreshed with `--update`
- No API key needed

### New: Page Source Badges

Python showed a generic disclaimer. Rust shows contextual badges:

```
[Bioinformatics Community Page: samtools.md]
[AI Generated Page (cached)]
[AI Generated Page]
```

Users can immediately see where content comes from.

### Dropped: Rating System

Python had `aitldr rate <command> up|down` to rate AI-generated pages. This was dropped in Rust because:
- No backend to store or use ratings
- Local ratings had limited value without a sharing mechanism
- Adds complexity without clear benefit

### Dropped: Submit Guide

Python had `aitldr submit <command>` to guide users toward submitting pages to the official tldr repository. Dropped because:
- Users can contribute directly via GitHub
- The feature was informational, not functional
- Reduces maintenance burden

## Architecture Comparison

```
Python aitldr                          Rust aitldr (tlrc fork)
========================               ========================

Standalone CLI                         Fork of official tlrc
  ├─ pages.py (HTTP fetch)              ├─ upstream cache + renderer
  ├─ core.py (lookup pipeline)          ├─ upstream page resolution
  ├─ ai.py (generation)        →        ├─ ai.rs (cache + dispatch)
  ├─ cache.py (file cache)              ├─ aitldr-cache crate (5-tier cache)
  ├─ config.py (TOML config)            ├─ aitldr-ai crate (generation)
  ├─ cli.py (Click CLI)                 ├─ aitldr-risk crate (safety)
  └─ __main__.py (entry)                ├─ aitldr-ai/config.rs (TOML config)
                                         └─ args.rs (clap CLI)

Page lookup:                            Page lookup:
  official (HTTP) → AI cache → AI        official (cache) → bio → AI cache → AI
```

## Configuration Compatibility

Both versions use `~/.aitldr/config.toml` with the same structure:

```toml
[general]
explain_default = false
cache_enabled = true
language = "zh"

[model]
provider = "deepseek"
model = "deepseek-chat"

[deepseek]
api_key = "env:DEEPSEEK_API_KEY"

[openai]
api_key = "env:OPENAI_API_KEY"

[ollama]
endpoint = "http://localhost:11434"
model = "qwen2:7b"
```

The Rust version reads the same config format. Migration requires no config changes.

## Summary

| Category | Count |
|----------|-------|
| Features replaced by upstream | 3 (page lookup, rendering, platform detection) |
| Features migrated | 8 (AI generation, NL queries, explain, caching, safety, config, env vars, language) |
| Features enhanced | 3 (caching, offline, refresh) |
| Features added | 2 (bio pages, source badges) |
| Features dropped | 2 (rating, submit) |

The rewrite traded two minor features (rating, submit) for full upstream compatibility, bioinformatics community pages, and a much better caching system. The core promise remains the same: **Official TLDR first. AI only fills gaps.**
