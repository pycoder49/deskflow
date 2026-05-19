use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config;

const BASE: &str = "https://api.clickup.com/api/v2";

fn token() -> String {
    std::env::var("CLICKUP_TOKEN").unwrap_or_default()
}

fn daily_list_id() -> String {
    config::get().clickup.daily_list_id.clone()
}

fn client() -> Client {
    Client::new()
}

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub status: Status,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub time_estimate: Option<i64>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub tags: Vec<TaskTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTag {
    pub name: String,
    #[serde(default)]
    pub tag_fg: Option<String>,
    #[serde(default)]
    pub tag_bg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub status: String,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Priority {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowNextResult {
    pub now: Option<Task>,
    pub next: Vec<Task>,
}

static NOW_NEXT_CACHE: std::sync::Mutex<Option<(Vec<String>, NowNextResult)>> =
    std::sync::Mutex::new(None);

#[derive(Deserialize)]
struct ListResponse {
    tasks: Vec<Task>,
}

#[derive(Deserialize)]
struct ClaudeSelection {
    now: Option<String>,
    #[serde(default)]
    next: Vec<String>,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn is_done(t: &Task) -> bool {
    matches!(t.status.status.to_lowercase().as_str(), "complete" | "closed" | "done")
}

async fn parse_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, String> {
    let text = resp.text().await.map_err(|e| e.to_string())?;
    serde_json::from_str::<T>(&text).map_err(|e| {
        let preview = &text[..text.len().min(300)];
        format!("JSON parse error: {e}\nResponse: {preview}")
    })
}

fn extract_json(s: &str) -> &str {
    if let Some(start) = s.find('{') {
        if let Some(end) = s.rfind('}') {
            return &s[start..=end];
        }
    }
    s
}

fn format_calendar_context(events: &[crate::calendar::CalendarEvent]) -> String {
    use chrono::Timelike;
    let now = chrono::Local::now();
    let now_min = now.hour() * 60 + now.minute();

    let lines: Vec<String> = events
        .iter()
        .filter(|e| !e.all_day && !e.start_time.is_empty())
        .filter_map(|e| {
            let parts: Vec<u32> =
                e.start_time.split(':').filter_map(|x| x.parse().ok()).collect();
            if parts.len() < 2 {
                return None;
            }
            let event_min = parts[0] * 60 + parts[1];
            if event_min <= now_min {
                return None;
            }
            let until = event_min - now_min;
            Some(format!(
                "- {} – {} {} (in {}min)",
                e.start_time, e.end_time, e.title, until
            ))
        })
        .collect();

    if lines.is_empty() {
        "No upcoming timed events today.".to_string()
    } else {
        lines.join("\n")
    }
}

async fn fetch_today_calendar_context() -> String {
    let base = chrono::Local::now() - chrono::Duration::hours(4);
    let today = base.format("%Y-%m-%d").to_string();
    let tomorrow = (base + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    match crate::calendar::fetch_events(&today, &tomorrow).await {
        Ok(evs) => format_calendar_context(&evs),
        Err(_) => "Calendar unavailable.".to_string(),
    }
}

async fn call_claude_cli(prompt: &str) -> Result<String, String> {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    #[cfg(windows)]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", "claude", "-p", "--model", "claude-haiku-4-5-20251001"]);
        c
    };

    #[cfg(not(windows))]
    let mut cmd = {
        let mut c = Command::new("claude");
        c.args(["-p", "--model", "claude-haiku-4-5-20251001"]);
        c
    };

    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "claude CLI not found on PATH".to_string()
        } else {
            format!("failed to spawn claude: {e}")
        }
    })?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| "failed to open claude stdin".to_string())?;
        stdin
            .write_all(prompt.as_bytes())
            .await
            .map_err(|e| format!("failed to write prompt: {e}"))?;
        stdin
            .shutdown()
            .await
            .map_err(|e| format!("failed to close stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("failed to read claude output: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("claude -p exited with {}: {stderr}", output.status));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| format!("claude stdout was not valid UTF-8: {e}"))
}

async fn log_action(action: &str, task_name: &str, tags: &[TaskTag], details: Option<&str>) {
    let script = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../scripts/log_action.py");

    let tag_str = tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(",");

    #[cfg(windows)]
    let mut cmd = {
        let mut c = tokio::process::Command::new("cmd");
        c.args(["/C", "python"]);
        c.arg(script);
        c
    };
    #[cfg(not(windows))]
    let mut cmd = {
        let mut c = tokio::process::Command::new("python3");
        c.arg(script);
        c
    };

    cmd.arg("--action").arg(action)
        .arg("--task-name").arg(task_name);
    if !tag_str.is_empty() {
        cmd.arg("--tags").arg(&tag_str);
    }
    if let Some(d) = details {
        cmd.arg("--details").arg(d);
    }

    match cmd.output().await {
        Ok(out) if out.status.success() => {
            eprintln!("[log] {}", String::from_utf8_lossy(&out.stdout).trim());
        }
        Ok(out) => {
            eprintln!("[log] script failed: {}", String::from_utf8_lossy(&out.stderr).trim());
        }
        Err(e) => {
            eprintln!("[log] spawn failed: {e}");
        }
    }
}

async fn fetch_today_tasks() -> Result<Vec<Task>, String> {
    let resp = client()
        .get(format!("{BASE}/list/{}/task", daily_list_id()))
        .header("Authorization", token())
        .query(&[("include_closed", "false"), ("subtasks", "false")])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: ListResponse = parse_response(resp).await?;
    Ok(data.tasks.into_iter().filter(|t| !is_done(t)).collect())
}

// ─── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_today_tasks() -> Result<Vec<Task>, String> {
    fetch_today_tasks().await
}

#[tauri::command]
pub async fn get_completed_today_tasks() -> Result<Vec<Task>, String> {
    // Logical day starts at 4am — only return tasks completed since then.
    let day_start_ms = {
        let now = chrono::Local::now();
        let shifted = now - chrono::Duration::hours(4);
        let midnight = shifted
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let day_start = chrono::TimeZone::from_local_datetime(&chrono::Local, &midnight)
            .unwrap()
            + chrono::Duration::hours(4);
        day_start.timestamp_millis().to_string()
    };

    let resp = client()
        .get(format!("{BASE}/list/{}/task", daily_list_id()))
        .header("Authorization", token())
        .query(&[
            ("include_closed", "true"),
            ("subtasks", "false"),
            ("statuses[]", "complete"),
            ("date_updated_gt", &day_start_ms),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: ListResponse = parse_response(resp).await?;
    Ok(data.tasks.into_iter().filter(|t| is_done(t)).collect())
}

#[tauri::command]
pub async fn get_now_next(preserve_now: bool) -> Result<NowNextResult, String> {
    let tasks = fetch_today_tasks().await?;

    if tasks.is_empty() {
        *NOW_NEXT_CACHE.lock().unwrap() = None;
        return Ok(NowNextResult { now: None, next: vec![] });
    }

    let mut pool_key: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
    pool_key.sort();

    if let Some((cached_key, cached_result)) = NOW_NEXT_CACHE.lock().unwrap().as_ref() {
        if cached_key == &pool_key {
            eprintln!("[NowNext] cache HIT ({} tasks)", pool_key.len());
            return Ok(cached_result.clone());
        }
    }

    let cached_now: Option<Task> = {
        let guard = NOW_NEXT_CACHE.lock().unwrap();
        guard.as_ref().and_then(|(_, r)| r.now.clone())
    };

    let task_map: std::collections::HashMap<String, Task> =
        tasks.iter().cloned().map(|t| (t.id.clone(), t)).collect();

    let cal_context = fetch_today_calendar_context().await;

    if preserve_now {
        if let Some(now_task) = cached_now.as_ref() {
            if task_map.contains_key(&now_task.id) {
                eprintln!(
                    "[NowNext] preserve-Now — picking Next only ({} tasks)",
                    pool_key.len()
                );
                let remaining: Vec<Task> = tasks
                    .iter()
                    .filter(|t| t.id != now_task.id)
                    .cloned()
                    .collect();
                let next: Vec<Task> = if remaining.is_empty() {
                    vec![]
                } else {
                    let ids = claude_pick_next(&remaining, &now_task.name, &cal_context).await?;
                    ids.into_iter()
                        .filter_map(|id| task_map.get(&id).cloned())
                        .take(2)
                        .collect()
                };
                let result = NowNextResult { now: Some(now_task.clone()), next };
                *NOW_NEXT_CACHE.lock().unwrap() = Some((pool_key, result.clone()));
                return Ok(result);
            }
        }
    }

    eprintln!("[NowNext] cache MISS — full pick ({} tasks)", pool_key.len());
    let selection = claude_pick_full(&tasks, &cal_context).await?;

    let now = selection
        .now
        .as_deref()
        .filter(|id| *id != "null" && !id.is_empty())
        .and_then(|id| task_map.get(id).cloned());

    let next: Vec<Task> = selection
        .next
        .iter()
        .filter_map(|id| task_map.get(id).cloned())
        .take(2)
        .collect();

    let result = NowNextResult { now, next };
    *NOW_NEXT_CACHE.lock().unwrap() = Some((pool_key, result.clone()));
    Ok(result)
}

fn format_task_line(t: &Task) -> String {
    let priority = t
        .priority
        .as_ref()
        .and_then(|p| p.id.as_deref())
        .unwrap_or("none");
    let estimate = t
        .time_estimate
        .map(|e| format!("{}min", e / 60000))
        .unwrap_or_else(|| "?min".to_string());
    format!("{} | p{} | {} | {}", t.id, priority, estimate, t.name)
}

async fn claude_pick_full(tasks: &[Task], cal_context: &str) -> Result<ClaudeSelection, String> {
    use chrono::Timelike;
    let now = chrono::Local::now();
    let now_str = format!("{:02}:{:02}", now.hour(), now.minute());

    let task_lines: Vec<String> = tasks.iter().map(format_task_line).collect();
    let prompt = format!(
        "Pick tasks to focus on right now.\n\nCurrent time: {now_str}\nUpcoming calendar events:\n{cal_context}\n\nTasks (id | priority(1=urgent,4=low) | estimate | name):\n{}\n\nIMPORTANT: Do not pick a task whose estimate would run past the next meeting. Prefer shorter tasks when a meeting is soon.\nReturn ONLY a JSON object, no explanation:\n{{\"now\":\"task_id\",\"next\":[\"id1\",\"id2\"]}}\n\nPick 1 for \"now\" (most urgent/important, fits before next meeting). Up to 2 for \"next\". Use null for \"now\" only if all tasks are trivial.",
        task_lines.join("\n")
    );
    let text = call_claude_cli(&prompt).await?;
    serde_json::from_str(extract_json(&text))
        .map_err(|e| format!("Claude returned invalid JSON: {e}\nRaw: {text}"))
}

async fn claude_pick_next(pool: &[Task], now_name: &str, cal_context: &str) -> Result<Vec<String>, String> {
    use chrono::Timelike;
    let now = chrono::Local::now();
    let now_str = format!("{:02}:{:02}", now.hour(), now.minute());

    let task_lines: Vec<String> = pool.iter().map(format_task_line).collect();
    let prompt = format!(
        "Current time: {now_str}\nUpcoming calendar events:\n{cal_context}\n\nThe user is currently focused on: \"{now_name}\". Pick up to 2 tasks to do NEXT from this list. Prefer tasks whose estimates fit in available time before the next meeting.\n\nTasks (id | priority(1=urgent,4=low) | estimate | name):\n{}\n\nReturn ONLY a JSON object, no explanation:\n{{\"next\":[\"id1\",\"id2\"]}}",
        task_lines.join("\n")
    );
    let text = call_claude_cli(&prompt).await?;

    #[derive(Deserialize)]
    struct NextOnly {
        #[serde(default)]
        next: Vec<String>,
    }

    let parsed: NextOnly = serde_json::from_str(extract_json(&text))
        .map_err(|e| format!("Claude returned invalid JSON: {e}\nRaw: {text}"))?;
    Ok(parsed.next)
}

#[tauri::command]
pub async fn create_task(
    name: String,
    list_id: String,
    priority: u8,
    tags: Vec<String>,
) -> Result<Task, String> {
    let body = serde_json::json!({ "name": name, "priority": priority, "tags": tags });

    let resp = client()
        .post(format!("{BASE}/list/{list_id}/task"))
        .header("Authorization", token())
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let task: Task = parse_response(resp).await?;
    log_action("create", &task.name, &task.tags, None).await;
    Ok(task)
}

#[tauri::command]
pub async fn complete_task(task_id: String, task_name: String, tags: Vec<TaskTag>) -> Result<(), String> {
    let body = serde_json::json!({ "status": "complete" });

    client()
        .put(format!("{BASE}/task/{task_id}"))
        .header("Authorization", token())
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    log_action("complete", &task_name, &tags, None).await;
    Ok(())
}

#[tauri::command]
pub async fn uncheck_task(task_id: String, task_name: String, tags: Vec<TaskTag>) -> Result<(), String> {
    let body = serde_json::json!({ "status": "in progress" });

    client()
        .put(format!("{BASE}/task/{task_id}"))
        .header("Authorization", token())
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    log_action("uncheck", &task_name, &tags, None).await;
    Ok(())
}

#[tauri::command]
pub async fn delete_task(task_id: String, task_name: String, tags: Vec<TaskTag>) -> Result<(), String> {
    let resp = client()
        .delete(format!("{BASE}/task/{task_id}"))
        .header("Authorization", token())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("delete failed {status}: {text}"));
    }

    log_action("delete", &task_name, &tags, None).await;
    Ok(())
}

#[tauri::command]
pub async fn update_task(
    task_id: String,
    task_name: String,
    tags: Vec<TaskTag>,
    new_name: Option<String>,
    new_priority: Option<u8>,
    new_due_date: Option<i64>,      // ms timestamp; 0 = clear
    new_time_estimate: Option<i64>, // ms; 0 = clear
    details: Option<String>,
) -> Result<(), String> {
    let mut body = serde_json::Map::new();
    if let Some(ref name) = new_name {
        body.insert("name".into(), serde_json::json!(name));
    }
    if let Some(priority) = new_priority {
        body.insert("priority".into(), serde_json::json!(priority));
    }
    if let Some(due) = new_due_date {
        if due == 0 {
            body.insert("due_date".into(), serde_json::Value::Null);
        } else {
            body.insert("due_date".into(), serde_json::json!(due));
        }
    }
    if let Some(est) = new_time_estimate {
        body.insert("time_estimate".into(), serde_json::json!(est));
    }

    let resp = client()
        .put(format!("{BASE}/task/{task_id}"))
        .header("Authorization", token())
        .json(&serde_json::Value::Object(body))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("update failed {status}: {text}"));
    }

    let display_name = new_name.as_deref().unwrap_or(&task_name);
    log_action("update", display_name, &tags, details.as_deref()).await;
    Ok(())
}

// Append-only log of every Start Day spawn — both the deterministic Python
// bootstrap and the AI skill run. Without this, claude -p failures were
// invisible (stderr was dropped) and the dashboard silently regressed.
fn append_start_day_log(project_root: &std::path::Path, phase: &str, stdout: &[u8], stderr: &[u8], exit: Option<i32>) {
    use std::io::Write;
    let log_dir = project_root.join("logs");
    if std::fs::create_dir_all(&log_dir).is_err() {
        return;
    }
    let log_path = log_dir.join("check-in.log");
    let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) else {
        return;
    };
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let code = exit.map_or_else(|| "?".to_string(), |c| c.to_string());
    let _ = writeln!(f, "─── [{ts}] {phase} (exit={code}) ───");
    if !stdout.is_empty() {
        let _ = writeln!(f, "STDOUT:\n{}", String::from_utf8_lossy(stdout));
    }
    if !stderr.is_empty() {
        let _ = writeln!(f, "STDERR:\n{}", String::from_utf8_lossy(stderr));
    }
    let _ = writeln!(f);
}

#[tauri::command]
pub async fn start_day() -> Result<String, String> {
    let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

    // Gate: skip everything if already checked in today (4am logical day).
    // The AI skill is responsible for writing this — so a failed prior run
    // leaves the gate open and the next click retries.
    let shifted = chrono::Local::now() - chrono::Duration::hours(4);
    let today_str = shifted.format("%Y-%m-%d").to_string();
    let state_path = project_root.join("clickup-state.json");
    if state_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&state_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if json.get("date").and_then(|d| d.as_str()) == Some(today_str.as_str()) {
                    return Ok("already".to_string());
                }
            }
        }
    }

    // Phase 1 — Deterministic bootstrap. Backfills task-stats.json via direct
    // ClickUp HTTP. Idempotent. If this fails, the whole click fails so the
    // user can retry; the chart's correctness can't depend on AI succeeding.
    let bootstrap_script = project_root.join("scripts").join("start_day.py");

    #[cfg(windows)]
    let mut boot_cmd = {
        let mut c = tokio::process::Command::new("cmd");
        c.args(["/C", "python"]);
        c.arg(&bootstrap_script);
        c
    };
    #[cfg(not(windows))]
    let mut boot_cmd = {
        let mut c = tokio::process::Command::new("python3");
        c.arg(&bootstrap_script);
        c
    };

    boot_cmd
        .arg("--bootstrap")
        .current_dir(&project_root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let boot_out = boot_cmd
        .output()
        .await
        .map_err(|e| format!("bootstrap spawn failed: {e}"))?;

    append_start_day_log(&project_root, "bootstrap", &boot_out.stdout, &boot_out.stderr, boot_out.status.code());

    if !boot_out.status.success() {
        let stderr = String::from_utf8_lossy(&boot_out.stderr);
        return Err(format!("bootstrap exited {}: {stderr}", boot_out.status));
    }

    // Gate owned by Rust: mark today checked-in before the skill fires.
    // A failed skill does not reopen the gate.
    let state_json = format!(r#"{{"date": "{}"}}"#, today_str);
    std::fs::write(&state_path, &state_json)
        .map_err(|e| format!("failed to write clickup-state.json: {e}"))?;

    // Phase 2 — AI skill: move decisions + retro log entries.
    let skill = config::get().commands.start_day_skill.clone();
    let arg = format!("/{skill}");

    #[cfg(windows)]
    let mut cmd = {
        let mut c = tokio::process::Command::new("cmd");
        c.args(["/C", "claude", "-p", &arg, "--output-format", "json"]);
        c
    };
    #[cfg(not(windows))]
    let mut cmd = {
        let mut c = tokio::process::Command::new("claude");
        c.args(["-p", &arg, "--output-format", "json"]);
        c
    };

    cmd.current_dir(&project_root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let child = cmd.spawn().map_err(|e| {
        let msg = format!("failed to spawn claude: {e}");
        append_start_day_log(&project_root, &format!("skill:{skill}"), msg.as_bytes(), b"", None);
        msg
    })?;
    let output = child
        .wait_with_output()
        .await
        .map_err(|e| {
            let msg = format!("start-day wait failed: {e}");
            append_start_day_log(&project_root, &format!("skill:{skill}"), msg.as_bytes(), b"", None);
            msg
        })?;

    append_start_day_log(&project_root, &format!("skill:{skill}"), &output.stdout, &output.stderr, output.status.code());

    // Parse token usage from JSON output and append a summary line.
    if let Ok(stdout_str) = std::str::from_utf8(&output.stdout) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(stdout_str) {
            let cost = json.get("total_cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let duration_ms = json.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0);
            let usage = json.get("usage");
            let input = usage.and_then(|u| u.get("input_tokens")).and_then(|v| v.as_u64()).unwrap_or(0);
            let output_tok = usage.and_then(|u| u.get("output_tokens")).and_then(|v| v.as_u64()).unwrap_or(0);
            let cache_read = usage.and_then(|u| u.get("cache_read_input_tokens")).and_then(|v| v.as_u64()).unwrap_or(0);
            let cache_create = usage.and_then(|u| u.get("cache_creation_input_tokens")).and_then(|v| v.as_u64()).unwrap_or(0);
            let total = input + output_tok + cache_read + cache_create;
            let summary = format!(
                "tokens: total={total} in={input} out={output_tok} cache_read={cache_read} cache_write={cache_create} | cost=${cost:.4} | {duration_ms}ms\n"
            );
            append_start_day_log(&project_root, &format!("skill:{skill} usage"), summary.as_bytes(), b"", None);
        }
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("start-day exited {}: {stderr}", output.status));
    }

    Ok("ok".to_string())
}

#[derive(Serialize)]
pub struct StatEntry {
    pub date: String,
    pub count: i64,
}

#[tauri::command]
pub async fn get_task_stats() -> Result<Vec<StatEntry>, String> {
    let stats_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../task-stats.json");

    let map: std::collections::HashMap<String, i64> = if stats_path.exists() {
        let content = tokio::fs::read_to_string(&stats_path)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        std::collections::HashMap::new()
    };

    let logical_today = chrono::Local::now() - chrono::Duration::hours(4);
    let mut entries = Vec::new();
    for days_ago in (0..14i64).rev() {
        let day = logical_today - chrono::Duration::days(days_ago);
        let date = day.date_naive().to_string();
        let count = map.get(&date).copied().unwrap_or(0);
        entries.push(StatEntry { date, count });
    }
    Ok(entries)
}

#[tauri::command]
pub async fn ensure_log_doc() -> Result<(), String> {
    let script = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../scripts/log_action.py");

    #[cfg(windows)]
    let mut cmd = {
        let mut c = tokio::process::Command::new("cmd");
        c.args(["/C", "python"]);
        c.arg(script);
        c
    };
    #[cfg(not(windows))]
    let mut cmd = {
        let mut c = tokio::process::Command::new("python3");
        c.arg(script);
        c
    };

    cmd.arg("--ensure");

    let out = cmd.output().await.map_err(|e| format!("spawn failed: {e}"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("ensure_log_doc failed: {stderr}"));
    }
    Ok(())
}
