# launch-kit.md — mcp-sentry launch 物料

> 配套: opportunity-memo.md / problem-corpus.md / build-plan.md / handoffs/current-cycle.md

## 项目一句话

```
mcp-sentry = 为 Claude Code / Cursor 部署 MCP server 的人提供的
             本地优先、跨 client、policy-as-code 的 approve/deny/audit 网关
```

**不**说: "AI 安全平台" / "下一代 MCP 生态" / "agent 安全"

## 目标受众

| 主 | 次 |
|---|---|
| 企业 DevOps / 平台工程团队 (50+ dev 用 Claude Code) | 个人 power user (装 10+ MCP server) |
| 合规需求 (SOC2 / ISO27001) | 担心 prompt injection 的开发者 |
| 已经用过但没治理的人 | 想 cross-client 统一 config 的人 |

## 标题候选 (Show HN)

| 编号 | 标题 | 风格 |
|---|---|---|
| A | Show HN: mcp-sentry – policy-as-code firewall for your MCP servers | 功能型 |
| B | Show HN: I built mcp-sentry after 12 MCP servers leaked my .env | 故事型 |
| C | Show HN: mcp-sentry – "ulimit" for MCP, deny any tool call before it runs | 类比型 |
| D | Show HN: mcp-sentry – local-first audit log for Claude Code's MCP calls | 数据型 |

**首推 C**: 类比 ulimit 是开发者秒懂的, "deny before it runs" 体现 enforce.

## 标题候选 (Reddit r/ClaudeAI)

- "I built mcp-sentry because I had 12 MCP servers and zero visibility into what they did"
- "Tool for anyone using MCP servers with Claude Code: local policy-as-code firewall"
- "Show your work: MCP server audit + enforce — looking for 5 beta testers"

## 标题候选 (V2EX)

- "我把 MCP server 治理做成了 CLI: mcp-sentry, 跨 client policy-as-code"
- "写了个 mcp-sentry 防止 MCP server 偷读 ~/.ssh, 求拍砖"
- "MCP server 装太多, 不知道哪个在干啥, 我做了 mcp-sentry"

## 30s demo gif 脚本

```
Scene 1 (5s) — 痛点
$ claude "delete /tmp/build"
[Claude 调 mcp server filesystem]
[filesystem 接受 delete 命令]

Scene 2 (10s) — 装 mcp-sentry + 拦截
$ mcp-sentry wrap --policy=strict.yaml -- filesystem
$ claude "delete /tmp/build"
[Claude 调 filesystem.delete_file]
[mcp-sentry] DENIED: filesystem.delete_file matches rule "tool: *__delete_*"
[Claude fallback to safer path]

Scene 3 (5s) — audit
$ tail ~/.mcp-sentry/audit.log
{"ts":"...","server":"filesystem","tool":"delete_file","decision":"denied","reason":"rule_match"}
{"ts":"...","server":"filesystem","tool":"read_file","decision":"allowed"}
```

## 中文内容 (V2EX / 掘金 / 知乎)

### 角度

**踩坑型** (V2EX): "我装了 12 个 MCP server, 直到 ~/.ssh 被读才慌了 — 然后我写了 mcp-sentry"

**工程型** (掘金): "MCP 协议下, 我们如何用 200 行 Rust 实现 policy-as-code 拦截器"

**问答型** (知乎): "AI coding agent 用 MCP server 时, 如何避免 prompt injection 和数据泄露?"

### 关键论点 (无论哪个平台)

1. MCP server 数量爆炸,治理层真空
2. ctxguard 已做 context budget, mcp-sentry 做 server governance — 同一作者, 同一 CLI 风格
3. Rust 1.1 MB single binary, local-first, 0 deps
4. policy.yaml = readable 5 行就生效
5. audit log 格式: JSON Lines, 标准 SIEM ingest

## KOL / Newsletter 名单

| 对象 | 平台 | 角度 |
|---|---|---|
| Simon Willison | blog + X | MCP security 关注度高 |
| Addy Osmani | blog | AI dev workflow |
| Latent Space | podcast + newsletter | MCP / agent infra |
| The Changelog | newsletter | Rust / CLI / DX |
| TLDR | newsletter | daily digest 适合简短 demo |
| Console | newsletter | devtools 早期项目 |
| HN | Show HN | 主战场 |

## 文案模板 (4 个平台)

### Show HN
```
Show HN: mcp-sentry – <一个具体结果> for <很窄的人群>

Hi HN, I built mcp-sentry because <具体痛点, 一句话>.
具体场景 (1-2 段真实场景).

What it does (3 个 bullet):
  • <action 1>
  • <action 2>
  • <action 3>

Numbers (1-2 个真实数字):
  • <benchmark>
  • <install: cargo install mcp-sentry>

Tested on Claude Code / Cursor / Codex CLI.

Looking for <具体反馈>. Happy to answer questions.
```

### Reddit r/ClaudeAI
```
I built mcp-sentry because <痛点>. Looking for feedback from people who <用户动作>.

What it does:
  • ...

Open source, MIT/Apache, single binary, zero deps.

[link to repo]
[link to 30s demo gif]
```

### X / Twitter
```
Built mcp-sentry after getting burned by <痛点>.

30s demo 👇

If you use Claude Code / Cursor / Codex CLI, roast it.

#MCP #devtools #AI
```

### V2EX
```
我把 <痛点> 做成了一个开源工具: mcp-sentry, 支持 <平台/环境>, 目前解决 <三个具体问题>, 求拍砖。
```

## Release artifact checklist (主 launch 前)

- [ ] repo public
- [ ] README 完整 (3 段 + install + usage + benchmarks)
- [ ] 30s demo gif (≤ 200 KB)
- [ ] LICENSE-MIT + LICENSE-APACHE
- [ ] v0.1.0 tag + release notes
- [ ] Linux x86_64 binary (≥ 1 MB)
- [ ] macOS Apple Silicon binary
- [ ] GitHub Actions CI 全绿
- [ ] 5 个 issue/discussion "非广告式出现"

## Time table

- W1 (现在 → 7/26): alpha + demo
- W2 (7/27 → 8/2): README + 冷启动
- W3 (8/3 → 8/9): 软启动 (X / Reddit / V2EX 各 1)
- W4 (8/10 → 8/16): 主 launch (Show HN + 中文复投)
- W5+ (8/17 →): 复盘 + 二次 launch

---

**Owner**: zhuke-ai
**Last updated**: 2026-07-19
**Status**: W1 Day 1 (4 markdown 工件完成)