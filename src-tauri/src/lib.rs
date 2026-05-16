mod calendar;
mod clickup;
mod config;
mod notepad;
mod spotify;
mod terminal;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
// CARGO_MANIFEST_DIR is src-tauri/; .env lives one level up at the project root.
const ENV_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../.env");

pub fn run() {
    dotenvy::from_filename(ENV_PATH).ok();
    config::load();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(terminal::PtyManager::new())
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            clickup::get_today_tasks,
            clickup::get_completed_today_tasks,
            clickup::get_now_next,
            clickup::create_task,
            clickup::complete_task,
            clickup::uncheck_task,
            clickup::delete_task,
            clickup::update_task,
            clickup::get_task_stats,
            clickup::ensure_log_doc,
            clickup::start_day,
            calendar::get_calendar_events,
            notepad::get_notepad,
            notepad::save_notepad,
            notepad::save_to_path,
            terminal::pty_create,
            terminal::pty_write,
            terminal::pty_resize,
            terminal::pty_kill,
            spotify::spotify_is_authenticated,
            spotify::spotify_auth,
            spotify::spotify_get_playlists,
            spotify::spotify_get_playlist_tracks,
            spotify::spotify_play_context,
            spotify::spotify_set_shuffle,
            spotify::spotify_search,
            spotify::spotify_play_uri,
            spotify::spotify_get_token,
            spotify::spotify_transfer_playback,
            spotify::spotify_get_state,
            spotify::spotify_play,
            spotify::spotify_pause,
            spotify::spotify_next,
            spotify::spotify_prev,
            spotify::spotify_get_devices,
            spotify::spotify_get_beats,
            spotify::spotify_get_audio_features,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
