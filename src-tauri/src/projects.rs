use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProjectEntry {
    id: String,
    name: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    id: String,
    name: String,
    path: String,
    context: Option<String>,
}

fn projects_json_path() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("../projects.json")
}

fn load_entries() -> Vec<ProjectEntry> {
    let path = projects_json_path();
    if !path.exists() {
        return vec![];
    }
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&text).unwrap_or_default()
}

fn save_entries(entries: &[ProjectEntry]) -> Result<(), String> {
    let path = projects_json_path();
    let text = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())
}

fn read_context(project_path: &str) -> Option<String> {
    std::fs::read_to_string(PathBuf::from(project_path).join("context.md")).ok()
}

fn entries_to_projects(entries: Vec<ProjectEntry>) -> Vec<Project> {
    entries
        .into_iter()
        .map(|e| {
            let context = read_context(&e.path);
            Project { id: e.id, name: e.name, path: e.path, context }
        })
        .collect()
}

#[tauri::command]
pub fn get_projects() -> Result<Vec<Project>, String> {
    Ok(entries_to_projects(load_entries()))
}

#[tauri::command]
pub fn add_project(path: String) -> Result<Vec<Project>, String> {
    let mut entries = load_entries();
    if entries.iter().any(|e| e.id == path) {
        return Err(format!("Already added: {}", path));
    }
    let name = PathBuf::from(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unnamed")
        .to_string();
    entries.push(ProjectEntry { id: path.clone(), name, path });
    save_entries(&entries)?;
    Ok(entries_to_projects(entries))
}

#[tauri::command]
pub fn remove_project(id: String) -> Result<Vec<Project>, String> {
    let mut entries = load_entries();
    entries.retain(|e| e.id != id);
    save_entries(&entries)?;
    Ok(entries_to_projects(entries))
}
