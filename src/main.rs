//! mcp-sentry — local-first policy-as-code firewall for MCP servers.
//!
//! Subcommands:
//!   list               List MCP servers found in well-known client config files
//!   audit <server>     Inspect a single MCP server (parse stdio or read source)
//!   wrap --policy=P -- Spawn a child MCP server, intercept tool calls per policy
//!   policy             Print the default policy.yaml schema + example

use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand};

mod config;
mod policy;
mod audit;
mod wrap;

#[derive(Parser, Debug)]
#[command(name = "mcp-sentry", version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// List all MCP servers discovered in Claude Code / Cursor / Codex / LM Studio config files.
    List {
        /// Show raw JSON of each entry too
        #[arg(long)]
        full: bool,
    },

    /// Inspect one MCP server: list its tools and inferred permissions.
    Audit {
        /// Server name (from `mcp-sentry list`) or its command path
        target: String,
    },

    /// Wrap an MCP server process. Spawns it as a child, intercepts JSON-RPC
    /// tool calls on stdio, decides allow/deny/prompt per the policy file,
    /// writes an audit log line per decision.
    Wrap {
        /// Path to policy YAML
        #[arg(long)]
        policy: PathBuf,

        /// Logical server name used to match policy rules (e.g. "filesystem").
        /// Policy rule format: `<server>__<tool>`.
        #[arg(long, default_value = "server")]
        server: String,

        /// Path to audit log (default: ~/.mcp-sentry/audit.log)
        #[arg(long)]
        audit_log: Option<PathBuf>,

        /// Command to run (everything after `--`)
        #[arg(last = true, required = true)]
        cmd: Vec<String>,
    },

    /// Print an example policy.yaml and the rule grammar.
    Policy,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::List { full } => {
            let servers = config::discover_all()?;
            config::print_table(&servers, full);
        }
        Cmd::Audit { target } => {
            audit::run(&target)?;
        }
        Cmd::Wrap { policy, server, audit_log, cmd } => {
            if cmd.is_empty() {
                anyhow::bail!("wrap: no command provided (use `--` separator)");
            }
            let log_path = audit_log.unwrap_or_else(|| {
                let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                p.push(".mcp-sentry");
                p.push("audit.log");
                p
            });
            wrap::run(&policy, &server, &log_path, cmd)?;
        }
        Cmd::Policy => {
            policy::print_help();
        }
    }
    Ok(())
}