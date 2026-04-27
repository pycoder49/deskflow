use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE: &str = "https://api.clickup.com/api/v2";
const DOCS_BASE: &str = "https://api.clickup.com/api/v3";
const DAILY_LIST: &str = "901414961997";
const WORKSPACE_ID: &str = "90141074324";

fn token() -> String {
    std::env::var("CLICKUP_TOKEN").unwrap_or_default()
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

#[derive(Deserialize)]
struct ClickupState {
    today_doc_id: String,
    today_page_id: String,
    date: String,
}

fn read_clickup_state() -> Option<ClickupState> {
    let path = dirs::home_dir()?.join(".claude").join("clickup-state.json");
    let raw = std::fs::read_to_string(path).ok()?;
    let state: ClickupState = serde_json::from_str(&raw).ok()?;
    // Logical day: 4am rollover. See ~/.claude/CLAUDE.md → Logical Day.
    let today = (chrono::Local::now() - chrono::Duration::hours(4))
        .format("%Y-%m-%d")
        .to_string();
    if state.date != today {
        return None;
    }
    Some(state)
}

async fn append_to_today_doc(entry: &str) -> Result<(), String> {
    let state = match read_clickup_state() {
        Some(s) => s,
        None => return Ok(()),
    };

    let url = format!(
        "{DOCS_BASE}/workspaces/{WORKSPACE_ID}/docs/{}/pages/{}",
        state.today_doc_id, state.today_page_id
    );
    let body = serde_json::json!({
        "content": format!("\n{entry}"),
        "content_edit_mode": "append",
        "content_format": "text/md",
    });

    let resp = client()
        .put(&url)
        .header("Authorization", token())
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("doc append request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("doc append {status}: {text}"));
    }
    Ok(())
}

async fn fetch_today_tasks() -> Result<Vec<Task>, String> {
    let resp = client()
        .get(format!("{BASE}/list/{DAILY_LIST}/task"))
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

    if let Err(e) = append_to_today_doc(&format!("- **add** — {}", task.name)).await {
        eprintln!("[log] add append failed: {e}");
    }

    Ok(task)
}

#[tauri::command]
pub async fn complete_task(task_id: String, task_name: String) -> Result<(), String> {
    let body = serde_json::json!({ "status": "complete" });

    client()
        .put(format!("{BASE}/task/{task_id}"))
        .header("Authorization", token())
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if let Err(e) = append_to_today_doc(&format!("- **complete** — {task_name}")).await {
        eprintln!("[log] complete append failed: {e}");
    }

    Ok(())
}

#[tauri::command]
pub async fn uncheck_task(task_id: String, task_name: String) -> Result<(), String> {
    let body = serde_json::json!({ "status": "in progress" });

    client()
        .put(format!("{BASE}/task/{task_id}"))
        .header("Authorization", token())
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if let Err(e) = append_to_today_doc(&format!("- **uncheck** — {task_name}")).await {
        eprintln!("[log] uncheck append failed: {e}");
    }

    Ok(())
}
