//! audit subcommand: inspect a single MCP server.
//!
//! For now we do a simple static inspection:
//!   - If the target is a server name we know from `list`, show its config.
//!   - If the target is a path to an executable / script, show its file info.
//!   - If the target is a directory, look for package.json / pyproject.toml.

use std::path::Path;
use anyhow::{Context, Result};

pub fn run(target: &str) -> Result<()> {
    let path = std::path::PathBuf::from(target);

    if path.exists() {
        return audit_path(&path);
    }

    // Treat as a server name from list
    let servers = crate::config::discover_all().unwrap_or_default();
    let matches: Vec<_> = servers.iter().filter(|s| s.name.eq_ignore_ascii_case(target)).collect();
    if !matches.is_empty() {
        for s in &matches {
            println!("name:    {}", s.name);
            println!("client:  {}", s.client);
            println!("source:  {}", s.source_file.display());
            if let Some(c) = &s.command { println!("command: {}", c); }
            if !s.args.is_empty() { println!("args:    {}", s.args.join(" ")); }
            if let Some(u) = &s.url { println!("url:     {}", u); }
            println!("kind:    {}", s.kind);
            println!("env:     {} var(s) configured", s.env_count);
            println!();
        }
        return Ok(());
    }

    anyhow::bail!("target {:?} is neither a known server nor a path", target);
}

fn audit_path(path: &Path) -> Result<()> {
    let meta = std::fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    println!("path:    {}", path.display());
    println!("size:    {} bytes", meta.len());
    println!("is_dir:  {}", meta.is_dir());

    if meta.is_dir() {
        // Look for typical MCP server source markers
        for marker in &["package.json", "pyproject.toml", "Cargo.toml", "go.mod", "main.py", "index.js"] {
            let p = path.join(marker);
            if p.exists() {
                println!("found:   {} ({} bytes)", p.display(), std::fs::metadata(&p)?.len());
                if marker == &"package.json" {
                    if let Ok(raw) = std::fs::read_to_string(&p) {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                            if let Some(name) = v.get("name").and_then(|x| x.as_str()) {
                                println!("package: {}", name);
                            }
                        }
                    }
                }
            }
        }
    } else {
        println!("note:    static scan only; live `tools/list` requires spawning the server");
    }
    Ok(())
}