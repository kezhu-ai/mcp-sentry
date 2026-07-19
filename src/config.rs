//! MCP config discovery across well-known client paths.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::Deserialize;
use tabled::{Table, Tabled};

#[derive(Debug, Clone, Deserialize)]
pub struct McpEntry {
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub name: String,
    pub client: String,
    pub source_file: PathBuf,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env_count: usize,
    pub kind: String,
    pub url: Option<String>,
}

/// Walk every well-known MCP config location and parse out server entries.
pub fn discover_all() -> Result<Vec<Server>> {
    let mut servers = Vec::new();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    let candidates: &[(&str, &str)] = &[
        ("claude-code", ".claude/.mcp.json"),
        ("claude-code", ".claude/mcp.json"),
        ("cursor", ".cursor/mcp.json"),
        ("codex-cli", ".codex/config.toml"),
        ("gemini-cli", ".gemini/config/mcp_config.json"),
        ("gemini-antigravity", ".gemini/antigravity/mcp_config.json"),
        ("lmstudio", ".lmstudio/mcp.json"),
        ("agent-reach", ".agent-reach/config/mcporter.json"),
        ("minimax", ".minimax/mcp/mcp.json"),
    ];

    for (client, rel) in candidates {
        let path = home.join(rel);
        if !path.exists() {
            continue;
        }
        if rel.ends_with(".toml") {
            match parse_toml(&path) {
                Ok(items) => servers.extend(items.into_iter().map(|(name, entry)| to_server(name, client, &path, entry))),
                Err(_) => continue,
            }
        } else {
            match parse_json(&path) {
                Ok(items) => servers.extend(items.into_iter().map(|(name, entry)| to_server(name, client, &path, entry))),
                Err(_) => continue,
            }
        }
    }

    servers.sort_by(|a, b| (a.client.as_str(), a.name.as_str()).cmp(&(b.client.as_str(), b.name.as_str())));
    Ok(servers)
}

fn parse_json(path: &PathBuf) -> Result<Vec<(String, McpEntry)>> {
    let raw = std::fs::read_to_string(path)?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    let obj = v.get("mcpServers")
        .or_else(|| v.get("servers"))
        .or_else(|| v.get("mcp_servers"))
        .and_then(|x| x.as_object())
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for (name, val) in obj {
        let entry: McpEntry = serde_json::from_value(val).unwrap_or(McpEntry {
            command: None,
            args: Vec::new(),
            env: BTreeMap::new(),
            kind: None,
            url: None,
        });
        out.push((name, entry));
    }
    Ok(out)
}

fn parse_toml(path: &PathBuf) -> Result<Vec<(String, McpEntry)>> {
    let raw = std::fs::read_to_string(path)?;
    let v: toml::Value = toml::from_str(&raw)?;
    let table = v.get("mcp_servers")
        .or_else(|| v.get("mcpServers"))
        .or_else(|| v.get("servers"))
        .and_then(|x| x.as_table())
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for (name, val) in table {
        let entry: McpEntry = match val.try_into() {
            Ok(e) => e,
            Err(_) => continue,
        };
        out.push((name, entry));
    }
    Ok(out)
}

fn to_server(name: String, client: &str, source: &Path, e: McpEntry) -> Server {
    Server {
        name,
        client: client.to_string(),
        source_file: source.to_path_buf(),
        command: e.command,
        args: e.args,
        env_count: e.env.len(),
        kind: e.kind.unwrap_or_else(|| "stdio".to_string()),
        url: e.url,
    }
}

#[derive(Tabled)]
struct Row {
    #[tabled(rename = "client")]
    client: String,
    #[tabled(rename = "name")]
    name: String,
    #[tabled(rename = "command")]
    command: String,
    #[tabled(rename = "args")]
    args: String,
    #[tabled(rename = "env")]
    env: String,
    #[tabled(rename = "kind")]
    kind: String,
}

pub fn print_table(servers: &[Server], full: bool) {
    if servers.is_empty() {
        println!("(no MCP servers discovered)");
        println!("checked: ~/.claude/.mcp.json, ~/.cursor/mcp.json, ~/.codex/config.toml, ~/.gemini/*, ~/.lmstudio/mcp.json");
        return;
    }
    let rows: Vec<Row> = servers.iter().map(|s| Row {
        client: s.client.clone(),
        name: s.name.clone(),
        command: s.command.clone().unwrap_or_else(|| "-".into()),
        args: if s.args.is_empty() { "-".into() } else { s.args.join(" ") },
        env: s.env_count.to_string(),
        kind: s.kind.clone(),
    }).collect();
    println!("{}", Table::new(rows));
    println!();
    println!("{} servers across {} client(s)", servers.len(), {
        let mut cs: Vec<&str> = servers.iter().map(|s| s.client.as_str()).collect();
        cs.sort(); cs.dedup();
        cs.len()
    });

    // Group by client
    let mut by_client: BTreeMap<&str, Vec<&Server>> = BTreeMap::new();
    for s in servers.iter() {
        by_client.entry(s.client.as_str()).or_default().push(s);
    }
    println!();
    println!("Per-client breakdown:");
    for (client, items) in by_client {
        println!("  {:<24} {} server(s)", client, items.len());
    }

    if full {
        println!();
        println!("Full config dump:");
        for s in servers {
            println!("--- {}:{} ---", s.client, s.name);
            println!("  source:   {}", s.source_file.display());
            if let Some(c) = &s.command { println!("  command:  {}", c); }
            if !s.args.is_empty() { println!("  args:     {}", s.args.join(" ")); }
            if let Some(u) = &s.url { println!("  url:      {}", u); }
            println!("  kind:     {}", s.kind);
        }
    }
}