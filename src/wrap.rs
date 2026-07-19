//! wrap subcommand: spawn an MCP server as a child process, intercept its
//! JSON-RPC traffic on stdio, decide allow/deny/prompt per policy, append
//! audit log lines.
//!
//! This is intentionally minimal: we read line-delimited JSON from the child,
//! pattern-match `tools/call`, write a decision to the audit log, and either
//! pass the call through unchanged (allow/prompt) or rewrite the response to
//! an error (deny). For prompt, we currently log + allow (UI not in scope
//! for v0.1).

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;

use crate::policy::Action;
use crate::policy::Policy;

pub fn run(policy_path: &Path, server: &str, audit_log: &PathBuf, cmd: Vec<String>) -> Result<()> {
    if let Some(parent) = audit_log.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let policy = Policy::load(policy_path)
        .with_context(|| format!("failed to load policy {}", policy_path.display()))?;

    let (program, args) = cmd.split_first().unwrap();
    eprintln!(
        "[mcp-sentry] server={} · policy={} · audit={} · child={} {}",
        server,
        policy_path.display(),
        audit_log.display(),
        program,
        args.join(" ")
    );

    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("failed to spawn {}", program))?;

    let mut child_stdin = child.stdin.take().context("no child stdin")?;
    let child_stdout = child.stdout.take().context("no child stdout")?;

    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        let reader = std::io::BufReader::new(child_stdout);
        for line in reader.lines().map_while(Result::ok) {
            if tx.send(line).is_err() { break; }
        }
    });

    // Read from our own stdin and forward to child, intercepting JSON-RPC.
    let our_stdin = std::io::stdin();
    let mut our_input = our_stdin.lock();
    let mut buf = String::new();
    loop {
        buf.clear();
        let n = match our_input.read_line(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                eprintln!("[mcp-sentry] stdin read error: {}", e);
                break;
            }
        };
        let line = buf.trim_end_matches(['\r', '\n']).to_string();
        if line.is_empty() { continue; }

        // Intercept JSON-RPC: if it's a tools/call, evaluate policy first.
        let intercept = parse_tools_call(&line);
        let (forward_line, decision_note) = match intercept {
            Some((server, tool)) => {
                let d = policy.decide(&server, &tool);
                log_decision(audit_log, &server, &tool, &d)?;
                if d.action == Action::Deny {
                    eprintln!(
                        "[mcp-sentry] DENY {}__{} (matched: {})",
                        server, tool, d.matched_rule
                    );
                    // Rewrite: return JSON-RPC error to the client.
                    let err = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": parse_id(&line).unwrap_or(serde_json::Value::Null),
                        "error": {
                            "code": -32000,
                            "message": format!(
                                "denied by mcp-sentry policy (rule: {}, reason: {})",
                                d.matched_rule, d.reason
                            )
                        }
                    });
                    (serde_json::to_string(&err).unwrap(), Some(d))
                } else {
                    if d.action == Action::Prompt {
                        eprintln!(
                            "[mcp-sentry] PROMPT {}__{} (matched: {}) — passing through",
                            server, tool, d.matched_rule
                        );
                    } else {
                        eprintln!("[mcp-sentry] ALLOW {}__{}", server, tool);
                    }
                    (line.clone(), Some(d))
                }
            }
            None => (line.clone(), None),
        };

        child_stdin.write_all(forward_line.as_bytes()).ok();
        child_stdin.write_all(b"\n").ok();
        child_stdin.flush().ok();
        let _ = n;
    drop(decision_note); // suppress unused warning on last iteration
    }

    // Drain any pending child responses and forward to our stdout, but
    // for short-lived demos we just wait a bit then exit.
    thread::sleep(Duration::from_millis(50));
    while let Ok(line) = rx.try_recv() {
        println!("{}", line);
    }
    let _ = child.kill();
    Ok(())
}

fn parse_tools_call(line: &str) -> Option<(String, String)> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let method = v.get("method")?.as_str()?;
    if method != "tools/call" { return None; }
    let params = v.get("params")?;
    // MCP spec: params.name is the tool name; the server is the connection.
    // We expose it via a `server` arg on `wrap` so policy can match per-server.
    let tool = params.get("name")?.as_str()
        .or_else(|| params.get("tool")?.as_str())?
        .to_string();
    Some(("server".to_string(), tool))
}

fn parse_id(line: &str) -> Option<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(line).ok()
        .and_then(|v| v.get("id").cloned())
}

fn log_decision(path: &PathBuf, server: &str, tool: &str, d: &crate::policy::Decision) -> Result<()> {
    let entry = serde_json::json!({
        "ts": Utc::now().to_rfc3339(),
        "server": server,
        "tool": tool,
        "action": format!("{:?}", d.action).to_lowercase(),
        "rule": d.matched_rule,
        "reason": d.reason,
    });
    let mut f = std::fs::OpenOptions::new()
        .create(true).append(true).open(path)?;
    writeln!(f, "{}", entry)?;
    Ok(())
}