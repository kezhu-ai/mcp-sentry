//! policy.yaml schema + default example.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Policy {
    pub version: u32,
    #[serde(default = "default_action")]
    pub default: Action,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Allow,
    Deny,
    Prompt,
}

fn default_action() -> Action { Action::Deny }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    /// Glob pattern matched against "<server>__<tool>", e.g. "filesystem__*",
    /// "*__delete_*", "github__create_issue", etc.
    pub tool: String,
    pub action: Action,
    /// Free-form reason, written into audit log when the rule fires.
    #[serde(default)]
    pub reason: Option<String>,
}

impl Policy {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let p: Policy = serde_yaml::from_str(&raw)?;
        Ok(p)
    }

    pub fn default_strict() -> Self {
        Policy {
            version: 1,
            default: Action::Deny,
            rules: vec![
                Rule { tool: "*__read_*".into(),  action: Action::Allow,  reason: Some("read is safe".into()) },
                Rule { tool: "*__list_*".into(),  action: Action::Allow,  reason: Some("list is safe".into()) },
                Rule { tool: "*__search_*".into(), action: Action::Allow, reason: Some("search is safe".into()) },
                Rule { tool: "*__get_*".into(),   action: Action::Allow,  reason: Some("get is safe".into()) },
                Rule { tool: "*__delete_*".into(), action: Action::Deny,  reason: Some("delete is destructive".into()) },
                Rule { tool: "*__write_*".into(),  action: Action::Prompt, reason: Some("write needs approval".into()) },
                Rule { tool: "*__exec_*".into(),   action: Action::Deny,  reason: Some("exec is high-risk".into()) },
                Rule { tool: "*__run_*".into(),    action: Action::Deny,  reason: Some("run is high-risk".into()) },
            ],
        }
    }

    pub fn decide(&self, server: &str, tool: &str) -> Decision {
        let key = format!("{}__{}", server, tool);
        for r in &self.rules {
            if glob_match(&r.tool, &key) {
                return Decision {
                    action: r.action,
                    matched_rule: r.tool.clone(),
                    reason: r.reason.clone().unwrap_or_default(),
                };
            }
        }
        Decision {
            action: self.default,
            matched_rule: "<default>".to_string(),
            reason: format!("no rule matched; default is {:?}", self.default),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub action: Action,
    pub matched_rule: String,
    pub reason: String,
}

fn glob_match(pattern: &str, text: &str) -> bool {
    // Minimal glob: only * wildcard, split on it, check substrings.
    if !pattern.contains('*') {
        return pattern == text;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.is_empty() {
        return true;
    }
    let mut idx = 0usize;
    // prefix
    if !parts[0].is_empty() {
        if !text.starts_with(parts[0]) {
            return false;
        }
        idx = parts[0].len();
    }
    // middle + suffix
    let last = parts.len() - 1;
    for (i, p) in parts.iter().enumerate() {
        if p.is_empty() { continue; }
        if i == 0 && !parts[0].is_empty() { continue; }
        if i == last {
            if !text[idx..].ends_with(p) { return false; }
            idx = text.len();
        } else {
            match text[idx..].find(p) {
                Some(pos) => { idx += pos + p.len(); }
                None => return false,
            }
        }
    }
    idx == text.len()
}

pub fn print_help() {
    println!("# Example policy.yaml — strict-by-default, read-only tools allowed");
    println!("# Usage: mcp-sentry wrap --policy ./policy.yaml -- <server-command>");
    println!();
    let yaml = serde_yaml::to_string(&Policy::default_strict()).unwrap();
    println!("{}", yaml);
}