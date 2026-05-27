<div align="center">

# tlrc

使用 Rust 编写的 [tldr](https://github.com/tldr-pages/tldr) 客户端，集成 AI 智能页面生成功能。

[![CI](https://img.shields.io/github/actions/workflow/status/tldr-pages/tlrc/ci.yml?label=CI&logo=github&labelColor=363a4f&logoColor=d9e0ee)](https://github.com/tldr-pages/tlrc/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/tldr-pages/tlrc?&logo=github&color=cba6f7&logoColor=d9e0ee&labelColor=363a4f)][latest-release]
[![crates.io](https://img.shields.io/crates/v/tlrc?&logo=rust&color=cba6f7&logoColor=d9e0ee&labelColor=363a4f)][crate]
[![license](https://img.shields.io/github/license/tldr-pages/tlrc?color=b4befe&labelColor=363a4f)](/LICENSE)
<br>
[![github downloads](https://img.shields.io/github/downloads/tldr-pages/tlrc/total?logo=github&color=94e2d5&logoColor=d9e0ee&labelColor=363a4f)][latest-release]
[![matrix](https://img.shields.io/matrix/tldr-pages%3Amatrix.org?logo=matrix&color=94e2d5&logoColor=d9e0ee&labelColor=363a4f&label=tldr-pages%20matrix)](https://matrix.to/#/#tldr-pages:matrix.org)

![screenshot](https://github.com/tldr-pages/tlrc/assets/126529524/daa76702-f437-4a99-adfb-7830a6f33eb9)

</div>

## aitldr 新增功能

aitldr 在官方 tlrc 基础上增加了 AI 驱动的功能：

- **AI 页面生成** — 当官方仓库中没有对应的 tldr 页面时，aitldr 会使用 AI（DeepSeek、OpenAI 或 Ollama）自动生成。
- **自然语言查询** — 用自然语言（包括中文）提问，直接获取对应的 shell 命令。
- **智能缓存** — AI 生成的页面、自然语言查询结果和命令解释都会缓存在本地，避免重复消耗 token。
- **生信社区页面** — 优先从 [bioinformatics-command](https://github.com/11010tianyi/bioinformatics-command) 仓库查询 40+ 生信工具和 25+ 生信格式页面。
- **危险命令警告** — 自动检测潜在的危险命令（如 `rm -rf`）并显示警告。
- **命令解释** — 使用 `--explain` 获取 AI 生成的命令详细解释，解释结果也会缓存。
- **离线 AI 支持** — `--offline` 只跳过缓存更新，AI 生成和缓存页面仍然可用（跳过命令存在性检查）。

## 安装

<a href="https://repology.org/project/tlrc/versions">
    <img src="https://repology.org/badge/vertical-allrepos/tlrc.svg?exclude_unsupported=1" alt="Packaging status" align="right">
</a>

### 从源码构建

```shell
git clone https://github.com/your-repo/aitldr-cli.git
cd aitldr-cli/tlrc
cargo build --release
```

编译后的二进制文件位于 `target/release/tldr`。

### Linux/macOS 使用 Homebrew

使用 Homebrew 安装 [tlrc](https://formulae.brew.sh/formula/tlrc)：

```shell
brew install tlrc
```

> **注意：** Homebrew 版本不包含 AI 功能。如需完整 aitldr 体验，请从源码构建。

### 从 crates.io 安装

```shell
cargo install tlrc --locked
```

> **注意：** 这会安装官方 tlrc，不包含 AI 功能。

### 从 GitHub Releases 下载

你可以在[这里][latest-release]找到预编译的二进制文件和 Debian 包。

## 快速开始

1. **初始化配置：**

```shell
tldr --init
```

这会创建 `~/.aitldr/config.toml`。编辑它来设置你的 API 密钥：

```toml
[deepseek]
api_key = "你的API密钥"

# 或者使用环境变量：
# api_key = "env:DEEPSEEK_API_KEY"
```

2. **查看命令页面：**

```shell
tldr tar
```

如果官方仓库中存在该页面，会显示官方版本。否则 aitldr 会使用 AI 生成一个。

3. **用自然语言提问：**

```shell
tldr "how to find large files"
tldr "查找大文件"
tldr "按文件大小排序"
```

4. **解释命令：**

```shell
tldr --explain "how to find large files"
tldr "按MB大小排序文件" --explain
```

## 使用说明

```
tldr [OPTIONS] [PAGE]...

标准选项：
  -u, --update                    更新缓存（同时刷新生信社区页面）
  -l, --list                      列出当前平台的所有页面
  -a, --list-all                  列出所有页面
  -s, --search <KEYWORD>          搜索包含关键词的页面
  -p, --platform <PLATFORM>       指定平台（linux、osx、windows 等）
  -L, --language <LANGUAGE_CODE>  指定语言
  -o, --offline                   不更新缓存（AI 生成仍然可用）
  -R, --raw                       以原始 Markdown 格式输出
  -r, --render <FILE>             渲染指定的 tldr 页面
  -q, --quiet                     静默模式，不显示状态信息

AI 选项：
  -e, --explain                   用 AI 解释命令（首次使用后自动缓存）
  -r, --refresh                   刷新 AI 页面和解释缓存，重新生成
  -m, --model <MODEL>             覆盖 AI 模型提供者（deepseek、openai、ollama）
      --init                      初始化 aitldr 配置（~/.aitldr/config.toml）
      --ai-status                 显示 aitldr AI 配置状态
```

## AI 功能详解

### 页面查找流程

运行 `tldr <命令>` 时，aitldr 按以下顺序查找：

1. **官方页面** — 在本地 tldr 缓存中搜索命令。
2. **生信社区页面** — 检查 `~/.aitldr/bio-command/` 中的生信工具页面（如 `samtools`、`gatk`、`stringtie`）。
3. **生信格式页面** — 检查 `~/.aitldr/bio-format/` 中的生信格式页面（如 `bam`、`vcf`、`fasta`）。
4. **AI 缓存** — 检查 `~/.aitldr/ai/` 中是否有之前生成的页面。
5. **AI 生成** — 使用配置的 AI 提供者生成新页面，但仅当命令在系统中存在时（防止幻觉）。离线模式下跳过存在性检查。

生信社区页面首次运行时自动从 [bioinformatics-command](https://github.com/11010tianyi/bioinformatics-command) 下载。使用 `--update` 可刷新。

### 离线模式

`--offline` 只跳过缓存更新——**不会**禁用 AI 功能：

- AI 缓存页面仍然显示
- AI 生成仍然可用（跳过 `command_exists` 检查，允许为本机未安装的命令生成页面）
- 生信社区页面如果已下载仍可使用
- 自然语言查询仍然可用

```shell
# 离线可用：显示缓存或 AI 生成的页面
tldr --offline stringtie

# 离线可用：自然语言查询 + AI 解释
tldr --offline "查找大文件" --explain
```

### 自然语言模式

当你的查询包含中文字符、超过 3 个单词或自然语言模式（如 "how to"、"find"、"show"）时，aitldr 会切换到自然语言模式：

1. **NL 缓存** — 检查 `~/.aitldr/nl/` 中是否有缓存结果。
2. **AI 生成** — 使用 AI 生成对应的 shell 命令。
3. **危险警告** — 如果命令有潜在危险，会显示警告。

示例：

```shell
# 英文查询
tldr "how to find large files"
tldr "create a new directory"
tldr "search for text in files"

# 中文查询
tldr "查找大文件"
tldr "按MB大小排序文件"
tldr "删除所有临时文件"
```

### 命令解释

添加 `--explain` 获取 AI 生成的命令详细解释：

```shell
tldr "sort files by size" --explain
```

你也可以在配置中默认启用解释功能：

```toml
[general]
explain_default = true
```

### AI 缓存

所有 AI 生成的内容会缓存在本地，避免重复的 API 调用：

- **页面缓存** — `~/.aitldr/ai/{命令}.md`，用于 AI 生成的 tldr 页面
- **NL 缓存** — `~/.aitldr/nl/{查询}.txt`，用于自然语言查询结果
- **解释缓存** — `~/.aitldr/explain/{命令}.txt`，用于 AI 生成的命令解释
- **生信社区页面** — `~/.aitldr/bio-command/` 和 `~/.aitldr/bio-format/`

使用 `--refresh` 删除特定命令的页面缓存和解释缓存，然后重新生成：

```shell
tldr --refresh tar
```

使用 `--update` 同时刷新官方 tldr 页面和生信社区页面：

```shell
tldr --update
```

### 危险命令警告

aitldr 会检测潜在的危险命令并显示警告：

```
tldr "删除所有文件"

rm -rf /path/to/dir

WARNING: Destructive command! This operation may cause irreversible data loss.
```

## AI 配置

配置文件位于 `~/.aitldr/config.toml`。使用 `tldr --init` 生成。

```toml
[general]
# 自然语言模式下默认解释命令
explain_default = false
# 启用 AI 结果缓存（推荐）
cache_enabled = true
# 输出语言："en" 英文，"zh" 中文
language = "zh"

[model]
# AI 提供者："deepseek"、"openai" 或 "ollama"
provider = "deepseek"
# 模型名称（因提供者而异）
model = "deepseek-chat"

[deepseek]
# API 密钥（或使用 env:DEEPSEEK_API_KEY 引用环境变量）
api_key = "env:DEEPSEEK_API_KEY"

[openai]
# API 密钥（或使用 env:OPENAI_API_KEY 引用环境变量）
api_key = "env:OPENAI_API_KEY"

[ollama]
# Ollama 服务端地址
endpoint = "http://localhost:11434"
# Ollama 使用的模型
model = "qwen2:7b"
```

### 查看配置

查看当前配置：

```shell
tldr --ai-status
```

输出示例：

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

### 环境变量

你可以在配置中使用 `env:` 前缀引用环境变量：

```toml
[deepseek]
api_key = "env:DEEPSEEK_API_KEY"

[openai]
api_key = "env:OPENAI_API_KEY"
```

然后设置环境变量：

```shell
export DEEPSEEK_API_KEY="sk-..."
```

### 支持的 AI 提供者

| 提供者 | 页面生成 | 自然语言 | 命令解释 | 需要 API 密钥 |
|--------|---------|---------|---------|--------------|
| DeepSeek | 支持 | 支持 | 支持 | 是 |
| OpenAI | 框架 | 框架 | 框架 | 是 |
| Ollama | 支持 | 暂不支持 | 暂不支持 | 否 |

> **注意：** OpenAI 和 Ollama 的自然语言/命令解释功能正在开发中。目前请使用 DeepSeek 获得完整功能体验。

## 标准 tlrc 配置

tlrc 可以通过 [TOML](https://toml.io) 配置文件自定义。获取系统默认路径：

```shell
tldr --config-path
```

生成默认配置文件：

```shell
tldr --gen-config > "$(tldr --config-path)"
```

### 配置选项

```toml
[cache]
# 覆盖缓存目录（'~' 会展开为你的主目录）
dir = "/path/to/cache"
# 覆盖下载 tldr 页面的基础 URL
mirror = "https://github.com/tldr-pages/tldr/releases/latest/download"
# 如果缓存超过 max_age 小时则自动更新
auto_update = true
# 在显示页面后执行自动更新
defer_auto_update = false
max_age = 336
# 指定需要的页面语言列表
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

如果想要类似 [tldr-python-client](https://github.com/tldr-pages/tldr-python-client) 的样式，可以添加以下配置：

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
