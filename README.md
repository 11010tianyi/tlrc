<div align="center">

# tlrc

A [tldr](https://github.com/tldr-pages/tldr) client written in Rust, with AI-powered page generation.

[![CI](https://img.shields.io/github/actions/workflow/status/tldr-pages/tlrc/ci.yml?label=CI&logo=github&labelColor=363a4f&logoColor=d9e0ee)](https://github.com/tldr-pages/tlrc/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/tldr-pages/tlrc?&logo=github&color=cba6f7&logoColor=d9e0ee&labelColor=363a4f)][latest-release]
[![crates.io](https://img.shields.io/crates/v/tlrc?&logo=rust&color=cba6f7&logoColor=d9e0ee&labelColor=363a4f)][crate]
[![license](https://img.shields.io/github/license/tldr-pages/tlrc?color=b4befe&labelColor=363a4f)](/LICENSE)
<br>
[![github downloads](https://img.shields.io/github/downloads/tldr-pages/tlrc/total?logo=github&color=94e2d5&logoColor=d9e0ee&labelColor=363a4f)][latest-release]
[![matrix](https://img.shields.io/matrix/tldr-pages%3Amatrix.org?logo=matrix&color=94e2d5&logoColor=d9e0ee&labelColor=363a4f&label=tldr-pages%20matrix)](https://matrix.to/#/#tldr-pages:matrix.org)

![screenshot](https://github.com/tldr-pages/tlrc/assets/126529524/daa76702-f437-4a99-adfb-7830a6f33eb9)

</div>

## What's New in aitldr

aitldr extends the official tlrc with AI-powered features:

- **AI Page Generation** — When a tldr page doesn't exist in the official repository, aitldr generates one using AI (DeepSeek, OpenAI, or Ollama).
- **Natural Language Queries** — Ask questions in plain language (including Chinese) and get the corresponding shell command.
- **Smart Caching** — AI-generated pages and natural language results are cached locally to avoid wasting tokens.
- **Destructive Command Warnings** — Flags potentially dangerous commands (e.g., `rm -rf`).
- **Command Explanation** — Get AI-generated explanations for any command with `--explain`.
- **Offline AI Support** — AI generation and cached pages work in offline mode (skips command existence check).

## Installation

<a href="https://repology.org/project/tlrc/versions">
    <img src="https://repology.org/badge/vertical-allrepos/tlrc.svg?exclude_unsupported=1" alt="Packaging status" align="right">
</a>

### Build from Source

```shell
git clone https://github.com/your-repo/aitldr-cli.git
cd aitldr-cli/tlrc
cargo build --release
```

The binary will be at `target/release/tldr`.

### Linux/macOS using Homebrew

Install [tlrc](https://formulae.brew.sh/formula/tlrc) with Homebrew:

```shell
brew install tlrc
```

> **Note:** The Homebrew version does not include AI features. Build from source for the full aitldr experience.

### From crates.io

```shell
cargo install tlrc --locked
```

> **Note:** This installs the official tlrc without AI features.

### From GitHub Releases

You can find prebuilt binaries and Debian packages [here][latest-release].

## Quick Start

1. **Initialize configuration:**

```shell
tldr --init
```

This creates `~/.aitldr/config.toml`. Edit it to set your API key:

```toml
[deepseek]
api_key = "your-api-key-here"

# Or use an environment variable:
# api_key = "env:DEEPSEEK_API_KEY"
```

2. **View a command page:**

```shell
tldr tar
```

If the page exists in the official repository, it shows the official version. If not, aitldr generates one with AI.

3. **Ask in natural language:**

```shell
tldr "how to find large files"
tldr "查找大文件"
tldr "按文件大小排序"
```

4. **Explain a command:**

```shell
tldr --explain "how to find large files"
tldr "按MB大小排序文件" --explain
```

## Usage

```
tldr [OPTIONS] [PAGE]...

Standard options:
  -u, --update                    Update the cache
  -l, --list                      List all pages in the current platform
  -a, --list-all                  List all pages
  -s, --search <KEYWORD>          Search for pages containing a keyword
  -p, --platform <PLATFORM>       Specify the platform (linux, osx, windows, etc.)
  -L, --language <LANGUAGE_CODE>  Specify the languages to use
  -o, --offline                   Do not update the cache, even if it is stale
  -R, --raw                       Print pages in raw Markdown
  -r, --render <FILE>             Render the specified tldr page
  -q, --quiet                     Suppress status messages and warnings

AI options:
  -e, --explain                   Explain the generated command (natural language mode)
  -r, --refresh                   Refresh AI-generated page (delete cache and regenerate)
  -m, --model <MODEL>             Override AI model provider (deepseek, openai, ollama)
      --init                      Initialize aitldr configuration
      --ai-status                 Show aitldr configuration status
```

## AI Features in Detail

### Page Lookup Flow

When you run `tldr <command>`, aitldr follows this lookup order:

1. **Official page** — Search the local tldr cache for the command.
2. **AI cache** — Check `~/.aitldr/ai/` for a previously generated page.
3. **AI generation** — Generate a new page using the configured AI provider, but only if the command exists on the system (prevents hallucination). In offline mode, the existence check is skipped.

### Natural Language Mode

When your query contains Chinese characters, more than 3 words, or natural language patterns (e.g., "how to", "find", "show"), aitldr switches to natural language mode:

1. **NL cache** — Check `~/.aitldr/nl/` for a cached result.
2. **AI generation** — Generate the corresponding shell command using AI.
3. **Destructive warning** — If the command is potentially dangerous, a warning is displayed.

Examples:

```shell
# English queries
tldr "how to find large files"
tldr "create a new directory"
tldr "search for text in files"

# Chinese queries
tldr "查找大文件"
tldr "按MB大小排序文件"
tldr "删除所有临时文件"
```

### Command Explanation

Add `--explain` to get an AI-generated explanation of the command:

```shell
tldr "sort files by size" --explain
```

You can also enable explanations by default in the config:

```toml
[general]
explain_default = true
```

### AI Caching

All AI-generated content is cached locally to avoid redundant API calls:

- **Page cache** — `~/.aitldr/ai/{command}.md` for AI-generated tldr pages
- **NL cache** — `~/.aitldr/nl/{query}.txt` for natural language query results

Use `--refresh` to delete the cache for a specific command and regenerate it:

```shell
tldr --refresh tar
```

### Destructive Command Warnings

aitldr detects potentially dangerous commands and displays a warning:

```
tldr "删除所有文件"

rm -rf /path/to/dir

WARNING: Destructive command! This operation may cause irreversible data loss.
```

## AI Configuration

The configuration file is at `~/.aitldr/config.toml`. Generate it with `tldr --init`.

```toml
[general]
# Explain commands by default when using natural language mode
explain_default = false
# Enable AI result caching (recommended)
cache_enabled = true
# Output language: "en" for English, "zh" for Chinese
language = "zh"

[model]
# AI provider: "deepseek", "openai", or "ollama"
provider = "deepseek"
# Model name (varies by provider)
model = "deepseek-chat"

[deepseek]
# API key (or use env:DEEPSEEK_API_KEY for environment variable)
api_key = "env:DEEPSEEK_API_KEY"

[openai]
# API key (or use env:OPENAI_API_KEY for environment variable)
api_key = "env:OPENAI_API_KEY"

[ollama]
# Ollama server endpoint
endpoint = "http://localhost:11434"
# Model to use with Ollama
model = "qwen2:7b"
```

### Checking Configuration

View your current configuration:

```shell
tldr --ai-status
```

Example output:

```
aitldr Configuration

  General:
    Explain default: false
    Cache enabled:   true
    Language:        zh

  Model:
    Provider: deepseek
    Model:    deepseek-chat

  DeepSeek:
    API Key: ********************

  OpenAI:
    API Key: Not configured

  Ollama:
    Endpoint: http://localhost:11434
    Model:    qwen2:7b

  Config directory: /home/user/.aitldr
```

### Environment Variables

You can reference environment variables in the config using the `env:` prefix:

```toml
[deepseek]
api_key = "env:DEEPSEEK_API_KEY"

[openai]
api_key = "env:OPENAI_API_KEY"
```

Then set the environment variable:

```shell
export DEEPSEEK_API_KEY="sk-..."
```

### Supported Providers

| Provider | Page Generation | Natural Language | Explanation | API Key Required |
|----------|----------------|------------------|-------------|-----------------|
| DeepSeek | Yes | Yes | Yes | Yes |
| OpenAI | Stub | Stub | Stub | Yes |
| Ollama | Yes | No | No | No |

> **Note:** OpenAI and Ollama natural language/explanation support are planned. Use DeepSeek for the full feature set.

## Standard tlrc Configuration

Tlrc can be customized with a [TOML](https://toml.io) configuration file. To get the default path for your system, run:

```shell
tldr --config-path
```

To generate a default config file, run:

```shell
tldr --gen-config > "$(tldr --config-path)"
```

### Configuration options

```toml
[cache]
# Override the cache directory ('~' will be expanded to your home directory).
dir = "/path/to/cache"
# Override the base URL used for downloading tldr pages.
mirror = "https://github.com/tldr-pages/tldr/releases/latest/download"
# Automatically update the cache if it's older than max_age hours.
auto_update = true
# Perform the automatic update after the page is shown.
defer_auto_update = false
max_age = 336
# Specify a list of desired page languages.
languages = []

[output]
show_title = true
platform_title = false
show_hyphens = false
edit_link = false
example_prefix = "- "
line_length = 0
compact = false
option_style = "long"
raw_markdown = false

[indent]
title = 2
description = 2
bullet = 2
example = 4

[style.title]
color = "magenta"
background = "default"
bold = true
underline = false
italic = false
dim = false
strikethrough = false

[style.description]
color = "magenta"
background = "default"
bold = false
underline = false
italic = false
dim = false
strikethrough = false

[style.bullet]
color = "green"
background = "default"
bold = false
underline = false
italic = false
dim = false
strikethrough = false

[style.example]
color = "cyan"
background = "default"
bold = false
underline = false
italic = false
dim = false
strikethrough = false

[style.url]
color = "red"
background = "default"
bold = false
underline = false
italic = true
dim = false
strikethrough = false

[style.inline_code]
color = "yellow"
background = "default"
bold = false
underline = false
italic = true
dim = false
strikethrough = false

[style.placeholder]
color = "red"
background = "default"
bold = false
underline = false
italic = true
dim = false
strikethrough = false
```

For a style similar to [tldr-python-client](https://github.com/tldr-pages/tldr-python-client), add this to your config:

```toml
[output]
show_hyphens = true
compact = true

[style]
title.color = "default"
title.bold = true
description.color = "default"
bullet.color = "green"
example.color = "red"
placeholder.color = "default"
```

[latest-release]: https://github.com/tldr-pages/tlrc/releases/latest
[crate]: https://crates.io/crates/tlrc
