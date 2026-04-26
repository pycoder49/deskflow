mod calendar;
mod clickup;
mod notepad;
mod vault;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
// CARGO_MANIFEST_DIR is src-tauri/; .env lives one level up at the project root.
const ENV_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../.env");

pub fn run() {
    dotenvy::from_filename(ENV_PATH).ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            clickup::get_today_tasks,
            clickup::get_now_next,
            clickup::create_task,
            clickup::complete_task,
            clickup::uncheck_task,
            calendar::get_calendar_events,
            vault::get_vault_pulse,
            notepad::get_notepad,
            notepad::save_notepad,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
