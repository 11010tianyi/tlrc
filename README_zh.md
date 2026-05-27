<div align="center">

# aitldr

**官方 TLDR 优先。AI 只填补空白。**

[tldr](https://github.com/tldr-pages/tldr) 客户端 — 官方页面有就用官方的，没有就 AI 生成一个。

[![license](https://img.shields.io/github/license/11010tianyi/tlrc?color=b4befe&labelColor=363a4f)](/LICENSE)

[English](README.md)

</div>

## 工作原理

```
tldr <命令>
  ├─ 官方页面找到了？       → 显示官方页面（与上游 tlrc 完全一致）
  ├─ 生信社区页面找到了？   → 显示社区页面 [Bioinformatics Community Page]
  ├─ AI 缓存找到了？       → 显示缓存页面 [AI Generated Page (cached)]
  └─ 都没找到？            → AI 生成新页面 [AI Generated Page]
```

官方 tldr 体验不受任何影响。AI 只是兜底 — 仅此而已。

## 功能

- **AI 页面生成** — DeepSeek、OpenAI 或 Ollama 为官方仓库没有的命令生成 tldr 页面。
- **自然语言查询** — 用自然语言（包括中文）提问，直接获取对应的 shell 命令。
- **生信社区页面** — 40+ 生信工具和 25+ 生信格式，来自 [bioinformatics-command](https://github.com/11010tianyi/bioinformatics-command)，优先于 AI 查询。
- **智能缓存** — 页面、自然语言结果和命令解释全部本地缓存，不浪费 token。
- **命令解释** — `--explain` 获取 AI 生成的命令解释，同样缓存。
- **危险命令警告** — 自动检测 `rm -rf` 等危险命令并警告。
- **离线 AI** — `--offline` 只跳过缓存更新，AI 生成仍然可用。

## 安装

```shell
git clone https://github.com/11010tianyi/tlrc.git
cd tlrc
cargo build --release
```

编译后的二进制文件在 `target/release/tldr`。移到 PATH 中：

```shell
cp target/release/tldr /usr/local/bin/
```

## 快速开始

**1. 初始化：**

```shell
tldr --init
```

编辑 `~/.aitldr/config.toml` 设置 API 密钥：

```toml
[deepseek]
api_key = "你的API密钥"

# 或使用环境变量：
# api_key = "env:DEEPSEEK_API_KEY"
```

**2. 使用：**

```shell
tldr tar              # 官方页面 — 和上游完全一样
tldr stringtie        # 生信社区页面 — 不消耗 token
tldr codesign         # AI 生成 — 只在没有其他来源时才用 AI
tldr "查找大文件"      # 自然语言 → shell 命令
tldr "sort by size" -e # 命令 + AI 解释
```

## 使用说明

```
tldr [OPTIONS] [PAGE]...

标准选项：
  -u, --update                    更新缓存 + 生信社区页面
  -l, --list                      列出当前平台的页面
  -a, --list-all                  列出所有页面
  -s, --search <KEYWORD>          搜索页面
  -p, --platform <PLATFORM>       指定平台（linux、osx、windows 等）
  -L, --language <LANGUAGE_CODE>  指定语言
  -o, --offline                   跳过缓存更新（AI 仍然可用）
  -R, --raw                       原始 Markdown 输出
  -q, --quiet                     静默模式

AI 选项：
  -e, --explain                   用 AI 解释命令（自动缓存）
  -r, --refresh                   删除 AI 页面和解释缓存，重新生成
  -m, --model <MODEL>             覆盖 AI 提供者（deepseek、openai、ollama）
      --init                      初始化 ~/.aitldr/config.toml
      --ai-status                 显示 AI 配置状态

页面查找顺序：官方缓存 → 生信社区页面 → AI 缓存 → AI 生成。
```

## 页面查找详解

运行 `tldr <命令>` 时，按以下顺序查找：

1. **官方页面** — 本地 tldr 缓存。与上游 tlrc 完全一致。
2. **生信社区页面** — `~/.aitldr/bio-command/` 中的生信工具页面（samtools、gatk、stringtie 等）。
3. **生信格式页面** — `~/.aitldr/bio-format/` 中的生信格式页面（bam、vcf、fasta 等）。
4. **AI 缓存** — `~/.aitldr/ai/` 中之前生成的页面。
5. **AI 生成** — 仅当命令在系统中存在时生成（防止幻觉）。`--offline` 跳过此检查。

生信社区页面首次运行时自动下载。`--update` 可刷新。

### 离线模式

`--offline` 只跳过缓存更新。其他功能照常：

```shell
tldr --offline stringtie           # 生信社区页面或 AI 缓存
tldr --offline "查找大文件" -e      # 自然语言查询 + AI 解释
tldr --offline codesign            # AI 生成（跳过 command_exists 检查）
```

## 缓存

所有 AI 内容本地缓存，避免重复 API 调用：

| 缓存 | 路径 | 用途 |
|------|------|------|
| 页面 | `~/.aitldr/ai/{命令}.md` | AI 生成的 tldr 页面 |
| NL | `~/.aitldr/nl/{查询}.txt` | 自然语言 → 命令 |
| 解释 | `~/.aitldr/explain/{命令}.txt` | AI 命令解释 |
| 生信命令 | `~/.aitldr/bio-command/*.md` | 生信工具页面 |
| 生信格式 | `~/.aitldr/bio-format/*.md` | 生信格式页面 |

```shell
tldr --refresh tar    # 删除 AI 页面和解释缓存，重新生成
tldr --update         # 刷新官方页面 + 生信社区页面
```

## AI 配置

配置文件：`~/.aitldr/config.toml`（通过 `tldr --init` 生成）。

```toml
[general]
explain_default = false    # 自然语言模式下自动解释
cache_enabled = true       # 缓存 AI 结果（推荐）
language = "zh"            # "en" 英文，"zh" 中文

[model]
provider = "deepseek"      # "deepseek"、"openai" 或 "ollama"
model = "deepseek-chat"

[deepseek]
api_key = "env:DEEPSEEK_API_KEY"   # 或直接粘贴密钥

[openai]
api_key = "env:OPENAI_API_KEY"

[ollama]
endpoint = "http://localhost:11434"
model = "qwen2:7b"
```

### 查看配置

```shell
tldr --ai-status
```

### 环境变量

在配置中使用 `env:` 前缀引用环境变量：

```shell
export DEEPSEEK_API_KEY="sk-..."
```

### 支持的 AI 提供者

| 提供者 | 页面生成 | 自然语言 | 命令解释 | 需要 API 密钥 |
|--------|---------|---------|---------|--------------|
| DeepSeek | 支持 | 支持 | 支持 | 是 |
| OpenAI | 计划中 | 计划中 | 计划中 | 是 |
| Ollama | 支持 | 计划中 | 计划中 | 否 |

## 危险命令警告

```
tldr "删除所有文件"

rm -rf /path/to/dir

WARNING: Destructive command! This operation may cause irreversible data loss.
```

## 标准 tlrc 配置

aitldr 继承 [上游 tlrc](https://github.com/tldr-pages/tlrc) 的所有配置选项。运行 `tldr --config-path` 查看配置文件路径，或 `tldr --gen-config` 打印默认配置。

类似 [tldr-python-client](https://github.com/tldr-pages/tldr-python-client) 的样式：

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

## Python 版本

本项目从 Python CLI（[aitldr-cli](https://github.com/11010tianyi/aitldr-cli)）重写而来。详细的功能迁移对比见 [docs/PYTHON_MIGRATION.md](docs/PYTHON_MIGRATION.md)。

## 许可证

[MIT](/LICENSE)
