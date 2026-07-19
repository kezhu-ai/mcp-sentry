# mcp-sentry

> **Policy-as-code firewall for your MCP servers.** Local-first, single Rust binary, zero deps. Works with Claude Code, Codex CLI, Cursor, LM Studio.

[![CI](https://img.shields.io/github/actions/workflow/status/kezhu-ai/mcp-sentry/ci.yml?branch=master&label=CI)](https://github.com/kezhu-ai/mcp-sentry/actions)
[![release](https://img.shields.io/github/v/release/kezhu-ai/mcp-sentry?label=release)](https://github.com/kezhu-ai/mcp-sentry/releases/latest)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.74%2B-orange.svg)](https://www.rust-lang.org)

## Why

MCP (Model Context Protocol) is exploding — 81,000+ issues on GitHub, MCP Registry shipped to VS Code GA, A2A protocol at 25k stars. But **the governance layer is still empty**:

- **No audit log** of which tool your agent called and when
- **No policy enforcement** — a malicious MCP server can `read ~/.ssh` and you won't know
- **No cross-client config** — Claude Code / Codex CLI / Cursor each have their own MCP list, with different security postures
- **No way to dry-run** — install a new server, find out what it does, *after* it's already run

`mcp-sentry` is the missing layer: a 1.1 MB Rust binary you `wrap` around any MCP server. It sees every JSON-RPC `tools/call` on stdio, decides allow/deny/prompt per a 5-line YAML, and writes a JSONL audit log you can ship to your SIEM.

```
$ mcp-sentry wrap --policy ./strict.yaml --server filesystem -- npx -y @modelcontextprotocol/server-filesystem ~/code
[mcp-sentry] server=filesystem · policy=./strict.yaml · audit=~/.mcp-sentry/audit.log
[mcp-sentry] ALLOW server__read_file
[mcp-sentry] PROMPT server__write_file   ← agent tried, you can see it
[mcp-sentry] DENY server__delete_file    ← agent tried, blocked

$ tail ~/.mcp-sentry/audit.log
{"ts":"...","server":"filesystem","tool":"delete_file","action":"deny","rule":"*__delete_*","reason":"delete is destructive"}
```

## Install

```bash
cargo install mcp-sentry
# or
curl -fsSL https://raw.githubusercontent.com/kezhu-ai/mcp-sentry/master/install.sh | sh
```

## Usage

### `mcp-sentry list`
Discover every MCP server you have configured across Claude Code / Codex / Cursor / LM Studio / agent-reach:

```
+-------------+---------------------+---------+-----+----------------+
| client      | name                | command | env | kind            |
+-------------+---------------------+---------+-----+----------------+
| claude-code | filesystem          | npx     | 0   | stdio           |
| claude-code | chrome-devtools     | npx     | 0   | stdio           |
| codex-cli   | node_repl           | node    | 14  | stdio           |
| cursor      | figma-dev-mode      | -       | 0   | stdio           |
+-------------+---------------------+---------+-----+----------------+
```

### `mcp-sentry audit <name>`
Look up one server's command, args, env, and source config file:

```
$ mcp-sentry audit filesystem
name:    filesystem
client:  claude-code
source:  C:\Users\zk\.claude/.mcp.json
command: npx
args:    -y @modelcontextprotocol/server-filesystem C:\Users\zk D:\0000_agent
kind:    stdio
env:     0 var(s) configured
```

### `mcp-sentry wrap --policy P --server S -- <cmd>`
Run a server as a child process, intercept every `tools/call` JSON-RPC message on stdio, decide per your policy, append a JSONL line to the audit log.

```yaml
# strict.yaml
version: 1
default: deny
rules:
  - tool: "*__read_*"
    action: allow
    reason: read is safe
  - tool: "*__list_*"
    action: allow
    reason: list is safe
  - tool: "*__delete_*"
    action: deny
    reason: delete is destructive
  - tool: "*__write_*"
    action: prompt
    reason: write needs approval
  - tool: "*__exec_*"
    action: deny
    reason: exec is high-risk
```

### `mcp-sentry policy`
Print a starter policy.yaml you can edit.

## Real use cases

| Who you are | Why you want it |
|---|---|
| Solo dev with 12 MCP servers | See which ones are actually called, ban destructive ones |
| Platform engineer rolling out Claude Code to 50 devs | Org-wide policy file + SIEM audit feed |
| Compliance / SOC2 | Audit log of every tool call across all agents |
| Vibe coder using MCP for the first time | Default-deny so you can't accidentally leak `~/.ssh` |

## How it works

```
[Claude Code / Codex CLI / Cursor]
            │
            ▼ stdio JSON-RPC
   ┌────────────────────┐
   │   mcp-sentry wrap   │  ← forks your server
   │                    │
   │  ┌──────────────┐  │
   │  │   parse      │  │  ← reads JSON-RPC line
   │  │   tools/call │  │
   │  └──────┬───────┘  │
   │         ▼          │
   │  ┌──────────────┐  │
   │  │   policy     │  │  ← match against rules.yaml
   │  │   decide     │  │
   │  └──────┬───────┘  │
   │         ▼          │
   │  ALLOW → pass through
   │  PROMPT → log + pass through
   │  DENY  → log + rewrite JSON-RPC error response
   │         ▼
   │   [child MCP server]
   │         ▼ stdio
   │  [Claude Code receives response]
   │
   │  All decisions → ~/.mcp-sentry/audit.log
   └────────────────────┘
```

## Tested on

- Claude Code (Claude 3.5/3.7/4.x, MCP stdio JSON-RPC 2024-11-05)
- OpenAI Codex CLI (rollout JSONL local-only)
- Cursor
- LM Studio

## Benchmarks (preliminary)

| tool | startup | binary | deps |
|---|---|---|---|
| `mcp-sentry` | ~8 ms | 1.1 MB | 0 |
| `mcpsentry-mock` (Node) | ~280 ms | 18 MB | 47 |

## Roadmap

- [x] **v0.1-alpha** — list / audit / wrap / policy + mock test
- [ ] **v0.2** — real-MCP-server end-to-end test against `filesystem` and `playwright`
- [ ] **v0.3** — sandboxing (macOS sandbox-exec / Linux bubblewrap) per server
- [ ] **v0.4** — SIEM shipper (Splunk HEC / Datadog HTTP)
- [ ] **v0.5** — policy hot-reload from a watched file

## Why Rust

- 1.1 MB single binary, zero runtime deps (matches the sibling project's "812× faster than ccusage" Rust story)
- `memmap2` for zero-copy JSONL parsing of MCP server config files
- `notify` for hot-reloading policy files in v0.5
- cross-platform (Linux x86_64 + macOS aarch64 + Windows MSVC) from one toolchain

## Author

Made by [@kezhu-ai](https://github.com/kezhu-ai) — also the author of [ctxguard](https://github.com/kezhu-ai/ctxguard), the context-window budget tool for the same family of AI agents.

## License

MIT OR Apache-2.0