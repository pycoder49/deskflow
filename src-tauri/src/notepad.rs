use std::path::PathBuf;

fn notepad_path() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("../notepad.md")
}

#[tauri::command]
pub fn get_notepad() -> Result<String, String> {
    let path = notepad_path();
    if !path.exists() {
        return Ok(String::new());
    }
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_notepad(content: String) -> Result<(), String> {
    let path = notepad_path();
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_to_path(content: String, path: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| e.to_string())
}
