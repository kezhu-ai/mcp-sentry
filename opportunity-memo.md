# opportunity-memo.md — MCP 策略网关赛道判断

> 决策依据: ChatGPT 4 周规划 + HN Algolia 真数据 + GitHub API 实时数据 (2026-07-19)
> 配套: problem-corpus.md / build-plan.md / launch/launch-kit.md

## 赛道选择: MCP 策略网关 (model-context-protocol-policy-gateway)

### 为什么选这个赛道 (5 条硬证据)

1. **GitHub MCP Registry 已 GA**: VS Code 中 MCP GA = 企业级采用拐点 = **基础设施层窗口期已开**
2. **HN 真实数据 (2026-07-19)**:
   - Mcp-Agent (80p/28c) — agent framework
   - Self-Hosted OAuth for MCP (4p) — 唯一相邻项目，已显示需求
   - **MCPSec** (2p) — "OWASP MCP Top Scanner" — 唯一直接竞品
   - gRPC to MCP Gateway (2p) — 边界相邻
3. **官方安全风险已被点名**: confused deputy / token passthrough / session hijacking / prompt injection
4. **零 enforce 层**: MCPSec 只做 scan,没人做 "approve/deny/pin version/audit log" 实时拦截
5. **ctxguard 复用**: 我已有 Rust CLI 经验 + notify 实时监控 + memmap2 解析

### 现有竞品 (按切入角度)

| 项目 | Stars | 切入角度 | 差距 |
|---|---|---|---|
| MCPSec | <100★ | OWASP scan (只读) | 没 enforce,没 approve/deny |
| Self-Hosted OAuth | <100★ | OAuth 认证层 | 单点功能,不是策略 |
| Mcp-Agent | <500★ | agent framework | agent 框架,不是 gateway |
| Kvlar (cross MCP) | <100★ | MCP firewall | 通用,不是 policy-as-code |
| gRPC to MCP | <50★ | proxy | 协议层,不是策略 |

### 为什么"策略网关"是真窗口

GitHub 2025 Octoverse 数据 (via ChatGPT brief):
- LLM SDK 仓库 +178%
- 932,791 个 agent-authored PR 跨 116,211 仓库
- MCP Registry 上线

**结论**: agent 工程已规模化,但 **治理层没有同步成熟**。这就是"server 很多, policy 太少"的真空区。

## 用户画像

**主用户**: 企业 DevOps / 平台工程团队,负责给团队部署 MCP server 时审查 (类似 Docker Hub → Harbor 的演进路径)
- 在 GitHub Enterprise 上有 100+ 开发者
- 用 Claude Code / Cursor / Claude Desktop 做 AI 编程
- 需要控制哪些 MCP server 可以被安装,哪些 tool 调用被允许
- 合规需求 (SOC2, ISO27001) — 需要 audit log

**次用户**: 个人开发者 (power user)
- 想用 Claude Code + 一堆 MCP server
- 担心 prompt injection / 数据泄露
- 不想要 SaaS 控制台 (本地优先)

**anti-persona**: 不想管安全的开发者 (用 default MCP 行为就行) — 不是我们的用户

## 一句话价值主张

```
mcp-sentry = 为 Claude Code / Cursor 部署 MCP server 的人提供的
             本地优先、跨 client、policy-as-code 的 approve/deny/audit 网关
```

**不**说: "AI 安全平台" / "下一代 MCP 生态" / "agent 安全"

## 30 条 complaint corpus

见 `problem-corpus.md` (基于 MCP 官方 security guide + GitHub Security Advisory + HN 讨论)。

## 为什么是 mcp-sentry 这个名字 (替代名候选)

候选:
- mcp-sentry (用 sentry = 哨兵, 守卫感) ← 选这个
- mcp-guard
- mcp-policy-engine
- mcpsentry (无连字符,模仿 godns)

理由: "sentry" 是开发者熟悉的词 (Sentry.io 错误监控), 暗示"持续守护", 跟 GitHub 早期推出的 Dependabot 命名同款 (类比熟悉词 + 行为)。

## 立项后 4 周路径 (来自 ChatGPT 规划, 简化)

| 周 | 主题 | 关键 deliverable |
|---|---|---|
| 第 1 周 | alpha + demo | 30s demo gif + corpus + 30 个冷启动测试者 |
| 第 2 周 | 修 README + issue 出现 | 找 5 个 issue / discussion 做"非广告式出现" |
| 第 3 周 | 软启动 + 中文镜像 | X / Reddit / V2EX / 掘金各 1 条 |
| 第 4 周 | 主 launch | Show HN 主帖 + 案例 + 中文复投 |

## critical path

| 必须先做 | 否则 |
|---|---|
| 30s demo GIF | 没人能 30s 明白价值,launch 转化率低 |
| corpus 30 条 | README / 标题没锋利度 |
| 4 个 handoff markdown | strategy 在聊天里飘,执行不连贯 |
| v0.1 可装可跑 | 不算"可分发的项目", 只算 demo |

## 不做的 (anti-list)

- ❌ 做 cloud 控制台 — 个人开发者不信任,SaaS 又需要付费
- ❌ 做 AI 行为监控 — Helicone / Langfuse 已经在做,赛道拥挤
- ❌ 做新 MCP server — server 已是红海 (punkpeye/awesome-mcp-servers 90k★)
- ❌ 做 agent framework — Mcp-Agent 80p 已占位

## 数字目标 (基于 ChatGPT 预判指标)

| 阶段 | star | 含义 |
|---|---|---|
| v0.1 上线 | 0 | baseline |
| 第 1 周后 | <50 | 种子期,需重定位 |
| 第 4 周后 | 200-500 | 突破"种子",有自传播迹象 |
| 3 个月 | 1000-3000 | niche 工具站稳 |
| 6 个月 | 5000-10000 | **首次 10k 突破窗口** |

如果第 4 周后 <50 stars 且 star/view <1% → 改 README 标题和 wedge,不要硬推。

## 风险

| 风险 | 概率 | 缓解 |
|---|---|---|
| GitHub 官方出类似 gateway | 中 | 抢时间窗,占住 "policy-as-code" 心智 |
| MCPSec 加 enforce 功能 | 中 | 先做 gateway,他们是 scanner,定位不同 |
| MCP 协议 v2 大改 | 低 | 协议稳定期,不太可能 |
| 用户不买账 (觉得 server 治理不是问题) | 低 | HN / Reddit 已显示需求,不是空话 |

---

**Owner**: zhuke-ai
**Last updated**: 2026-07-19
**Status**: 选定赛道, 进入 alpha build