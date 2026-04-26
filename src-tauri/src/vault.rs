use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

const DEFAULT_VAULT: &str = "C:/Users/aryan/Desktop/AryanOS";
const RAW_DIRS: &[&str] = &["raw/notes", "raw/clips"];

fn vault_path() -> PathBuf {
    std::env::var("VAULT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_VAULT))
}

fn vault_name() -> String {
    vault_path()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("vault")
        .to_string()
}

// ─── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultCounts {
    pub sources: u32,
    pub topics: u32,
    pub entities: u32,
    pub queries: u32,
    pub study_guides: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InboxItem {
    pub path: String,     // forward-slash relative path: raw/notes/foo.md
    pub name: String,     // basename
    pub modified: String, // YYYY-MM-DD
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogDay {
    pub date: String,         // "2026-04-21"
    pub bullets: Vec<String>, // top-level "- " bullets, leading dash stripped
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HotSection {
    pub title: String, // "Where We Left Off", "Next Session — Start Here", etc.
    pub body: String,  // raw markdown body (bullets or prose)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultPulse {
    pub vault_name: String,
    pub hot_sections: Vec<HotSection>,
    pub hot_updated: String,
    pub hot_session: String,
    pub counts: VaultCounts,
    pub inbox: Vec<InboxItem>,
    pub recent_log: Vec<LogDay>,
}

// ─── Parsers ────────────────────────────────────────────────────────────────

fn read_rel(rel: &str) -> String {
    std::fs::read_to_string(vault_path().join(rel)).unwrap_or_default()
}

// Walk hot.md and split into all `## ` sections in document order.
// Skips empty sections.
fn parse_hot_sections(md: &str) -> Vec<HotSection> {
    let mut out: Vec<HotSection> = Vec::new();
    let mut current: Option<HotSection> = None;
    for line in md.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(s) = current.take() {
                let trimmed = s.body.trim().to_string();
                if !trimmed.is_empty() {
                    out.push(HotSection { title: s.title, body: trimmed });
                }
            }
            current = Some(HotSection {
                title: rest.trim().to_string(),
                body: String::new(),
            });
            continue;
        }
        if let Some(ref mut s) = current {
            s.body.push_str(line);
            s.body.push('\n');
        }
    }
    if let Some(s) = current {
        let trimmed = s.body.trim().to_string();
        if !trimmed.is_empty() {
            out.push(HotSection { title: s.title, body: trimmed });
        }
    }
    out
}

fn parse_hot_meta(md: &str) -> (String, String) {
    // Returns (updated, session) from "**Last updated:** ... | **Session:** ..."
    let mut updated = String::new();
    let mut session = String::new();
    for line in md.lines() {
        let Some(rest) = line.strip_prefix("**Last updated:**") else {
            continue;
        };
        for (i, seg) in rest.split('|').enumerate() {
            let s = seg.trim();
            if let Some(v) = s.strip_prefix("**Session:**") {
                session = v.trim().to_string();
            } else if i == 0 {
                updated = s.to_string();
            }
        }
        break;
    }
    (updated, session)
}

fn parse_counts(md: &str) -> VaultCounts {
    // First line of body: "Last updated: ... | Sources: N | Entities: N | Topics: N | ..."
    let header = md
        .lines()
        .find(|l| l.starts_with("Last updated:"))
        .unwrap_or("");
    let mut c = VaultCounts {
        sources: 0,
        topics: 0,
        entities: 0,
        queries: 0,
        study_guides: 0,
    };
    for seg in header.split('|') {
        let mut parts = seg.splitn(2, ':');
        let key = parts.next().unwrap_or("").trim().to_lowercase();
        let val = parts.next().unwrap_or("").trim();
        let n: u32 = val.parse().unwrap_or(0);
        match key.as_str() {
            "sources" => c.sources = n,
            "topics" => c.topics = n,
            "entities" => c.entities = n,
            "queries" => c.queries = n,
            "study guides" => c.study_guides = n,
            _ => {}
        }
    }
    c
}

fn parse_log(md: &str, max_days: usize) -> Vec<LogDay> {
    let mut days: Vec<LogDay> = Vec::new();
    let mut current: Option<LogDay> = None;
    for line in md.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(d) = current.take() {
                days.push(d);
                if days.len() >= max_days {
                    return days;
                }
            }
            current = Some(LogDay {
                date: rest.trim().to_string(),
                bullets: Vec::new(),
            });
            continue;
        }
        if let Some(ref mut day) = current {
            // Top-level bullets only — skip indented sub-bullets
            if let Some(rest) = line.strip_prefix("- ") {
                day.bullets.push(rest.trim().to_string());
            }
        }
    }
    if let Some(d) = current {
        if days.len() < max_days {
            days.push(d);
        }
    }
    days
}

// Walk wiki/sources/*.md, extract `**File:** \`raw/...\`` references.
// Returns set of vault-relative raw paths that have already been ingested.
fn ingested_raw_paths() -> HashSet<String> {
    let mut set = HashSet::new();
    let dir = vault_path().join("wiki/sources");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return set;
    };
    for entry in entries.flatten() {
        if entry.path().extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        for line in content.lines() {
            let Some((_, after)) = line.split_once("**File:**") else {
                continue;
            };
            // Extract path between the first pair of backticks on this line.
            let Some(start) = after.find('`') else { break };
            let rest = &after[start + 1..];
            let Some(end) = rest.find('`') else { break };
            set.insert(rest[..end].trim().replace('\\', "/"));
            break;
        }
    }
    set
}

fn build_inbox() -> Vec<InboxItem> {
    let ingested = ingested_raw_paths();
    let mut items: Vec<(InboxItem, SystemTime)> = Vec::new();
    for rel_dir in RAW_DIRS {
        let dir = vault_path().join(rel_dir);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            if !ft.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let rel_path = format!("{rel_dir}/{name}");
            if ingested.contains(&rel_path) {
                continue;
            }
            let Ok(meta) = entry.metadata() else { continue };
            let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let modified = DateTime::<Local>::from(mtime).format("%Y-%m-%d").to_string();
            items.push((
                InboxItem {
                    path: rel_path,
                    name,
                    modified,
                },
                mtime,
            ));
        }
    }
    items.sort_by(|a, b| b.1.cmp(&a.1));
    items.into_iter().map(|(i, _)| i).collect()
}

// ─── Command ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_vault_pulse() -> Result<VaultPulse, String> {
    let hot = read_rel("wiki/hot.md");
    let index = read_rel("wiki/index.md");
    let log = read_rel("wiki/log.md");

    let (hot_updated, hot_session) = parse_hot_meta(&hot);
    Ok(VaultPulse {
        vault_name: vault_name(),
        hot_sections: parse_hot_sections(&hot),
        hot_updated,
        hot_session,
        counts: parse_counts(&index),
        inbox: build_inbox(),
        recent_log: parse_log(&log, 5),
    })
}
