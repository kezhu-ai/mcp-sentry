# build-plan.md — mcp-sentry 第 1 周 MVP 构建计划

> 配套: opportunity-memo.md / problem-corpus.md / handoffs/current-cycle.md / launch/launch-kit.md

## 目标 (本周)

**W1 deliverable**: 一个能装的 `mcp-sentry` v0.0.1 alpha,能做 3 件事:
1. `mcp-sentry list` — 列出本机所有 MCP server (从 Claude Code / Cursor config 读取)
2. `mcp-sentry audit <server>` — 静态扫描 server 二进制/源码,显示 tool 列表 + 权限推断
3. `mcp-sentry wrap --policy=policy.yaml -- <server-cmd>` — 拦截所有 MCP tool call,按 policy 决定 approve/deny

## 4 周 macro plan (来自 ChatGPT)

| 周 | 主题 | 周交付 |
|---|---|---|
| W1 | alpha + demo | 上述 3 个 subcommand + 30s demo gif + corpus |
| W2 | README + 社区出现 | 改 README + 5 个冷启动测试者 + 3 个 issue 出现 |
| W3 | 软启动 | X / Reddit / V2EX / 掘金各 1 条,中文镜像重写 |
| W4 | 主 launch | Show HN 主帖 + 案例 + 中文复投 |

## W1 daily breakdown

### Day 1 (今天)
- [ ] 建项目目录 + git init (DONE)
- [ ] 写 opportunity-memo.md / problem-corpus.md / build-plan.md / launch/launch-kit.md 4 个 markdown (DONE)
- [ ] cargo init --bin mcp-sentry
- [ ] 决定 stdio vs http 优先 (stdio JSON-RPC 是 MCP 标准)

### Day 2
- [ ] 实现 `mcp-sentry list`:
  - 读 `~/.config/claude-code/mcp_servers.json` (Claude Code 路径)
  - 读 `~/.codex/config.toml` (Codex CLI)
  - 读 `~/.cursor/mcp.json` (Cursor)
  - 解析后表格输出: server name, command, args, env var count
- [ ] 测试: 本机应该有 5-10 个 MCP server

### Day 3
- [ ] 实现 `mcp-sentry audit`:
  - 接收 server 路径 (binary 或源码 dir)
  - 静态扫描: 找 `tools/list` 返回的 JSON, 列 tool name + description + input schema
  - 显示: tool count, 是否有 destructive tool (write/delete/exec)
- [ ] 测试: 对本机常用 MCP server (filesystem / github / postgres) 各跑一次

### Day 4
- [ ] 实现 `mcp-sentry wrap`:
  - 启动子进程, stdio pipe
  - parse JSON-RPC messages, 拦截 `tools/call`
  - 按 policy.yaml 决策: allow / deny / prompt
  - audit log: 写每条 call 到 ~/.mcp-sentry/audit.log
- [ ] policy.yaml schema:
  ```yaml
  version: 1
  default: deny  # deny / allow / prompt
  rules:
    - tool: filesystem__read_file
      action: allow
    - tool: filesystem__write_file
      action: prompt
    - tool: "*__delete_*"
      action: deny
  ```

### Day 5
- [ ] 集成测试: 用 policy.yaml 启动 filesystem MCP server, 让 Claude Code 调它, 验证 deny 生效
- [ ] 写 README.md 草稿 (含 30s demo gif 脚本)
- [ ] git tag v0.0.1-alpha

### Day 6
- [ ] 录 30s demo gif (用 vhs 在本地 Git Bash, 已经装好)
  - Scene 1: `mcp-sentry list` (5s, 显示本机 8 个 MCP server)
  - Scene 2: `mcp-sentry wrap --policy=strict.yaml -- filesystem` (10s, Claude Code 试图 delete /etc, 被 deny)
  - Scene 3: `cat ~/.mcp-sentry/audit.log` (5s, 显示 timestamp + tool + decision)
- [ ] 写 README + LICENSE-MIT + LICENSE-APACHE

### Day 7
- [ ] 第一次 git push + GitHub repo create
- [ ] 写 Show HN draft (基于 marketing/show-hn-draft.md 模板)
- [ ] commit + 准备进入 W2

## 技术栈 (复用 ctxguard 经验)

- **语言**: Rust 1.94 (已用)
- **CLI**: clap (已用)
- **JSON**: serde_json (已用)
- **配置**: figment (新, 支持 JSON/TOML/YAML)
- **Policy 引擎**: 自写简单 evaluator (不需要复杂的 CEL/OPA)
- **子进程**: std::process::Command + stdio pipe (已用过)
- **Audit log**: 简单 JSON Lines 写到 ~/.mcp-sentry/audit.log

## 不做的 (避免 over-engineering)

- ❌ HTTP transport — MCP stdio 是 90% 用例
- ❌ Database (SQLite 等) — audit log 是 plain text 足够 v0.0.1
- ❌ Web UI — CLI 是 v0.0.1 的全部
- ❌ Cloud 同步 — 本地优先是定位
- ❌ Plugin 系统 — 写死 3 个 built-in check 就够

## critical path

| 必须 W1 完成 | 否则 |
|---|---|
| `wrap` subcommand (实际拦截) | 没 demo 可录,没价值 |
| policy.yaml schema | demo 无说服力 |
| 真实 audit log | 用户看不到发生什么 |
| 30s demo gif | launch 时没视觉资产 |

## 风险

| 风险 | 缓解 |
|---|---|
| Claude Code MCP config 格式变了 | 用 try-parse + 错误提示,不 crash |
| filesystem MCP server 在 Windows 路径不同 | W1 只测 macOS, W2 加 Windows |
| Demo 录屏失败 (vhs 在 Windows 慢) | 用 SVG fallback (参考 ctxguard) |

---

**Owner**: zhuke-ai
**Last updated**: 2026-07-19
**Next**: Day 2 开始 cargo init + list subcommand