use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::config;

// gcalcli display names (the `summary` field). Personal calendar comes from
// `os-config.json`; secondary calendars (holidays, birthdays) are
// edited in code below. `--calendar` flags must precede the subcommand.
const EXTRA_CALENDARS: &[&str] = &[
    "Holidays in United States",
    "Birthdays",
];

fn calendars() -> Vec<String> {
    let mut cals = vec![config::get().calendar.personal_email.clone()];
    cals.extend(EXTRA_CALENDARS.iter().map(|s| s.to_string()));
    cals.into_iter().filter(|c| !c.is_empty()).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub title: String,
    pub calendar: String,
    pub all_day: bool,
}

async fn run_gcalcli(start: &str, end: &str) -> Result<String, String> {
    // PYTHONIOENCODING=utf-8 forces Python to use UTF-8 for stdout regardless
    // of the Windows console code page (cp1252 by default). Without this,
    // events with emoji or non-Latin characters crash gcalcli's print().
    let mut cmd = Command::new("gcalcli");
    cmd.env("PYTHONIOENCODING", "utf-8");
    for cal in &calendars() {
        cmd.args(["--calendar", cal]);
    }
    cmd.args(["agenda", start, end, "--tsv", "--details=calendar"]);

    let output = cmd
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "gcalcli not found on PATH".to_string()
            } else {
                format!("failed to spawn gcalcli: {e}")
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gcalcli exited with {}: {stderr}", output.status));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| format!("gcalcli stdout was not valid UTF-8: {e}"))
}

fn parse_tsv(raw: &str) -> Vec<CalendarEvent> {
    raw.lines()
        .skip(1) // header row
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 6 {
                return None;
            }
            let start_time = fields[1].trim().to_string();
            let end_time = fields[3].trim().to_string();
            let all_day = start_time.is_empty() && end_time.is_empty();
            Some(CalendarEvent {
                start_date: fields[0].trim().to_string(),
                start_time,
                end_date: fields[2].trim().to_string(),
                end_time,
                title: fields[4].trim().to_string(),
                calendar: fields[5].trim().to_string(),
                all_day,
            })
        })
        .collect()
}

pub async fn fetch_events(start: &str, end: &str) -> Result<Vec<CalendarEvent>, String> {
    let raw = run_gcalcli(start, end).await?;
    Ok(parse_tsv(&raw))
}

#[tauri::command]
pub async fn get_calendar_events(
    start: String,
    end: String,
) -> Result<Vec<CalendarEvent>, String> {
    fetch_events(&start, &end).await
}
