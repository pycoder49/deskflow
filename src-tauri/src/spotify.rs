use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;

const SPOTIFY_BASE: &str = "https://api.spotify.com/v1";
const TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const REDIRECT_URI: &str = "http://127.0.0.1:8888/callback";
// Pre-encoded for the auth URL query string
const REDIRECT_URI_ENC: &str = "http%3A%2F%2F127.0.0.1%3A8888%2Fcallback";
const SCOPES_ENC: &str = "streaming%20user-read-email%20user-read-private\
    %20user-read-playback-state%20user-modify-playback-state%20user-read-currently-playing\
    %20user-read-recently-played\
    %20playlist-read-private%20playlist-read-collaborative";

fn client_id() -> String {
    std::env::var("SPOTIFY_CLIENT_ID").unwrap_or_default()
}

fn client_secret() -> String {
    std::env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_default()
}

fn tokens_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("spotify-tokens.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpotifyTokens {
    access_token: String,
    refresh_token: String,
    expires_at: i64, // unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyState {
    pub is_playing: bool,
    pub track_name: String,
    pub artist: String,
    pub album: String,
    pub album_art: Option<String>,
    pub progress_ms: u64,
    pub duration_ms: u64,
    pub device: Option<String>,
    pub track_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFeatures {
    pub energy: f32,
    pub tempo: f32,
    pub valence: f32,
}

fn load_tokens() -> Option<SpotifyTokens> {
    let data = std::fs::read_to_string(tokens_path()).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_tokens(tokens: &SpotifyTokens) {
    let path = tokens_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(tokens) {
        let _ = std::fs::write(path, json);
    }
}

async fn get_valid_token() -> Result<String, String> {
    let mut tokens = load_tokens().ok_or("not_authenticated")?;
    let now = chrono::Local::now().timestamp();
    if now < tokens.expires_at - 60 {
        return Ok(tokens.access_token);
    }
    // Refresh expired token
    let raw = Client::new()
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", tokens.refresh_token.as_str()),
            ("client_id", client_id().as_str()),
            ("client_secret", client_secret().as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("token refresh send: {e}"))?
        .text()
        .await
        .map_err(|e| format!("token refresh body: {e}"))?;
    let resp: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("token refresh parse: {e} — body: {}", raw.chars().take(200).collect::<String>()))?;

    if let Some(err) = resp["error"].as_str() {
        // Refresh token revoked — delete stale tokens
        let _ = std::fs::remove_file(tokens_path());
        return Err(format!("not_authenticated: {err}"));
    }

    tokens.access_token = resp["access_token"]
        .as_str()
        .ok_or("refresh returned no access_token")?
        .to_string();
    if let Some(r) = resp["refresh_token"].as_str() {
        tokens.refresh_token = r.to_string();
    }
    tokens.expires_at = now + resp["expires_in"].as_i64().unwrap_or(3600);
    save_tokens(&tokens);
    Ok(tokens.access_token)
}

async fn check_resp(resp: reqwest::Response) -> Result<(), String> {
    let status = resp.status();
    if status.is_success() {
        return Ok(());
    }
    let body = resp.text().await.unwrap_or_default();
    let msg = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v["error"]["message"].as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("HTTP {}", status.as_u16()));
    Err(msg)
}

// ─── Commands ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub uri: String,
    pub name: String,
    pub image: Option<String>,
    pub track_count: u32,
    pub owner: String,
    pub last_played_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub uri: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub album_art: Option<String>,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn spotify_get_playlists() -> Result<Vec<Playlist>, String> {
    let token = get_valid_token().await?;
    let client = Client::new();

    // Best-effort MRU lookup. Failures here just mean playlists keep API order.
    let mut mru: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    if let Ok(resp) = client
        .get(format!("{SPOTIFY_BASE}/me/player/recently-played"))
        .bearer_auth(&token)
        .query(&[("limit", "50")])
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                for item in data["items"].as_array().unwrap_or(&vec![]) {
                    let Some(ctx) = item.get("context").filter(|c| !c.is_null()) else { continue };
                    if ctx["type"].as_str() != Some("playlist") { continue; }
                    let Some(uri) = ctx["uri"].as_str() else { continue };
                    let Some(played_at) = item["played_at"].as_str() else { continue };
                    let Ok(ts) = chrono::DateTime::parse_from_rfc3339(played_at) else { continue };
                    // /recently-played returns newest first — first occurrence per uri wins
                    mru.entry(uri.to_string()).or_insert(ts.timestamp_millis());
                }
            }
        }
    }

    let resp = client
        .get(format!("{SPOTIFY_BASE}/me/playlists"))
        .bearer_auth(&token)
        .query(&[("limit", "50")])
        .send()
        .await
        .map_err(|e| format!("playlists send: {e}"))?;
    let status = resp.status();
    if status.as_u16() == 429 {
        let retry = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("a few");
        return Err(format!("Spotify rate-limited — retry in {retry}s"));
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "playlists HTTP {}: {}",
            status.as_u16(),
            body.chars().take(200).collect::<String>()
        ));
    }
    let resp: serde_json::Value = resp.json().await.map_err(|e| format!("playlists parse: {e}"))?;

    let mut playlists: Vec<Playlist> = resp["items"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|p| {
            let uri = p["uri"].as_str()?.to_string();
            let last_played_at = mru.get(&uri).copied();
            Some(Playlist {
                id: p["id"].as_str()?.to_string(),
                uri,
                name: p["name"].as_str()?.to_string(),
                image: p["images"]
                    .as_array()
                    .and_then(|imgs| imgs.first())
                    .and_then(|img| img["url"].as_str())
                    .map(|s| s.to_string()),
                track_count: p["tracks"]["total"].as_u64().unwrap_or(0) as u32,
                owner: p["owner"]["display_name"].as_str().unwrap_or("").to_string(),
                last_played_at,
            })
        })
        .collect();

    // Most-recently-played first; never-played tail keeps API order (stable sort).
    playlists.sort_by(|a, b| b.last_played_at.cmp(&a.last_played_at));

    Ok(playlists)
}

#[tauri::command]
pub async fn spotify_get_playlist_tracks(playlist_id: String) -> Result<Vec<PlaylistTrack>, String> {
    let token = get_valid_token().await?;
    let resp: serde_json::Value = Client::new()
        .get(format!("{SPOTIFY_BASE}/playlists/{playlist_id}/tracks"))
        .bearer_auth(&token)
        .query(&[
            ("limit", "100"),
            ("fields", "items(track(uri,name,duration_ms,artists(name),album(name,images)))"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(resp["items"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|item| {
            let t = item.get("track")?;
            if t.is_null() { return None; }
            let uri = t["uri"].as_str()?.to_string();
            if !uri.starts_with("spotify:track:") { return None; } // skip local files
            Some(PlaylistTrack {
                uri,
                name: t["name"].as_str().unwrap_or("").to_string(),
                artist: t["artists"]
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(|a| a["name"].as_str())
                    .unwrap_or("")
                    .to_string(),
                album: t["album"]["name"].as_str().unwrap_or("").to_string(),
                album_art: t["album"]["images"]
                    .as_array()
                    .and_then(|imgs| imgs.last()) // smallest image for the list
                    .and_then(|img| img["url"].as_str())
                    .map(|s| s.to_string()),
                duration_ms: t["duration_ms"].as_u64().unwrap_or(0),
            })
        })
        .collect())
}

#[tauri::command]
pub async fn spotify_play_context(
    context_uri: String,
    device_id: String,
    offset: usize,
) -> Result<(), String> {
    let token = get_valid_token().await?;
    let body = serde_json::json!({
        "context_uri": context_uri,
        "offset": { "position": offset },
    });
    let mut req = Client::new()
        .put(format!("{SPOTIFY_BASE}/me/player/play"))
        .bearer_auth(&token)
        .json(&body);
    if !device_id.is_empty() {
        req = req.query(&[("device_id", device_id.as_str())]);
    }
    check_resp(req.send().await.map_err(|e| e.to_string())?).await?;
    Ok(())
}

#[tauri::command]
pub async fn spotify_set_shuffle(state: bool, device_id: String) -> Result<(), String> {
    let token = get_valid_token().await?;
    let state_str = if state { "true" } else { "false" };
    let mut req = Client::new()
        .put(format!("{SPOTIFY_BASE}/me/player/shuffle"))
        .query(&[("state", state_str)])
        .bearer_auth(&token)
        .header("Content-Length", "0");
    if !device_id.is_empty() {
        req = req.query(&[("device_id", device_id.as_str())]);
    }
    check_resp(req.send().await.map_err(|e| e.to_string())?).await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTrack {
    pub uri: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub album_art: Option<String>,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn spotify_search(query: String) -> Result<Vec<SearchTrack>, String> {
    let token = get_valid_token().await?;
    let resp: serde_json::Value = Client::new()
        .get(format!("{SPOTIFY_BASE}/search"))
        .bearer_auth(&token)
        .query(&[("q", query.as_str()), ("type", "track"), ("limit", "10")])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let tracks = resp["tracks"]["items"]
        .as_array()
        .ok_or("unexpected response from Spotify search")?;

    Ok(tracks
        .iter()
        .map(|t| SearchTrack {
            uri: t["uri"].as_str().unwrap_or("").to_string(),
            name: t["name"].as_str().unwrap_or("").to_string(),
            artist: t["artists"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|a| a["name"].as_str())
                .unwrap_or("")
                .to_string(),
            album: t["album"]["name"].as_str().unwrap_or("").to_string(),
            // Use smallest image (last in the array) — usually 64px, fine for the list
            album_art: t["album"]["images"]
                .as_array()
                .and_then(|imgs| imgs.last())
                .and_then(|img| img["url"].as_str())
                .map(|s| s.to_string()),
            duration_ms: t["duration_ms"].as_u64().unwrap_or(0),
        })
        .collect())
}

#[tauri::command]
pub async fn spotify_play_uri(uri: String, device_id: String) -> Result<(), String> {
    let token = get_valid_token().await?;
    let body = serde_json::json!({ "uris": [uri] });
    Client::new()
        .put(format!("{SPOTIFY_BASE}/me/player/play"))
        .query(&[("device_id", device_id.as_str())])
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn spotify_get_token() -> Result<String, String> {
    get_valid_token().await
}

#[tauri::command]
pub async fn spotify_transfer_playback(device_id: String) -> Result<(), String> {
    let token = get_valid_token().await?;
    let body = serde_json::json!({ "device_ids": [device_id] });
    Client::new()
        .put(format!("{SPOTIFY_BASE}/me/player"))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn spotify_is_authenticated() -> bool {
    tokens_path().exists()
}

#[tauri::command]
pub async fn spotify_auth() -> Result<(), String> {
    let cid = client_id();
    if cid.is_empty() {
        return Err("SPOTIFY_CLIENT_ID not set in .env".to_string());
    }
    let auth_url = format!(
        "https://accounts.spotify.com/authorize?client_id={cid}&response_type=code\
         &redirect_uri={REDIRECT_URI_ENC}&scope={SCOPES_ENC}"
    );

    // Open system browser — raw_arg avoids Rust re-quoting so cmd.exe sees:
    // cmd /C start "" "url_with_&" (quoted URL prevents & from being a cmd separator)
    use std::os::windows::process::CommandExt;
    std::process::Command::new("cmd")
        .raw_arg(&format!("/C start \"\" \"{}\"", auth_url))
        .spawn()
        .map_err(|e| e.to_string())?;

    // Spawn a thread to catch the OAuth redirect on 127.0.0.1:8888
    let (tx, rx) = mpsc::channel::<Result<String, String>>();
    std::thread::spawn(move || {
        let _ = tx.send(capture_oauth_code());
    });

    // Wait up to 2 minutes for the code; on timeout/error, unblock the listener thread
    let code = tokio::task::spawn_blocking(move || {
        match rx.recv_timeout(Duration::from_secs(120)) {
            Ok(result) => result,
            Err(_) => {
                // Unblock the listener thread by connecting to it
                let _ = std::net::TcpStream::connect("127.0.0.1:8888");
                Err("OAuth timed out — please try again".to_string())
            }
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    // Exchange code for tokens
    let resp: serde_json::Value = Client::new()
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", REDIRECT_URI),
            ("client_id", client_id().as_str()),
            ("client_secret", client_secret().as_str()),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(err) = resp["error"].as_str() {
        let desc = resp["error_description"].as_str().unwrap_or(err);
        return Err(desc.to_string());
    }

    let access_token = resp["access_token"]
        .as_str()
        .ok_or("no access_token in response")?
        .to_string();
    let refresh_token = resp["refresh_token"]
        .as_str()
        .ok_or("no refresh_token in response")?
        .to_string();
    let expires_at = chrono::Local::now().timestamp()
        + resp["expires_in"].as_i64().unwrap_or(3600);

    save_tokens(&SpotifyTokens { access_token, refresh_token, expires_at });
    Ok(())
}

fn capture_oauth_code() -> Result<String, String> {
    let listener =
        TcpListener::bind("127.0.0.1:8888").map_err(|e| format!("port 8888 unavailable: {e}"))?;
    let (mut stream, _) = listener.accept().map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();

    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf).unwrap_or(0);

    let _ = stream.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n\
          <html><head><style>body{font-family:sans-serif;display:flex;align-items:center;\
          justify-content:center;height:100vh;margin:0;background:#191414;color:#1db954}\
          </style></head><body><h2>Spotify connected! You can close this tab.</h2>\
          </body></html>",
    );

    let req = String::from_utf8_lossy(&buf[..n]);
    // "GET /callback?code=xxx&state=... HTTP/1.1"
    let path = req
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .unwrap_or("");
    let qs = path.split('?').nth(1).unwrap_or("");

    for pair in qs.split('&') {
        if let Some(code) = pair.strip_prefix("code=") {
            if !code.is_empty() {
                return Ok(code.to_string());
            }
        }
        if let Some(err) = pair.strip_prefix("error=") {
            return Err(format!("Spotify denied: {err}"));
        }
    }
    Err("No auth code in callback".to_string())
}

#[tauri::command]
pub async fn spotify_get_state() -> Result<Option<SpotifyState>, String> {
    let token = get_valid_token().await?;

    let resp = Client::new()
        .get(format!("{SPOTIFY_BASE}/me/player"))
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // 204 = no active playback / no device
    if resp.status().as_u16() == 204 {
        return Ok(None);
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let item = data.get("item");

    Ok(Some(SpotifyState {
        is_playing: data["is_playing"].as_bool().unwrap_or(false),
        track_name: item
            .and_then(|i| i["name"].as_str())
            .unwrap_or("Unknown")
            .to_string(),
        artist: item
            .and_then(|i| i["artists"].as_array())
            .and_then(|a| a.first())
            .and_then(|a| a["name"].as_str())
            .unwrap_or("Unknown")
            .to_string(),
        album: item
            .and_then(|i| i["album"]["name"].as_str())
            .unwrap_or("")
            .to_string(),
        album_art: item
            .and_then(|i| i["album"]["images"].as_array())
            .and_then(|imgs| imgs.first())
            .and_then(|img| img["url"].as_str())
            .map(|s| s.to_string()),
        progress_ms: data["progress_ms"].as_u64().unwrap_or(0),
        duration_ms: item
            .and_then(|i| i["duration_ms"].as_u64())
            .unwrap_or(0),
        device: data["device"]["name"].as_str().map(|s| s.to_string()),
        track_id: item.and_then(|i| i["id"].as_str()).map(|s| s.to_string()),
    }))
}

#[tauri::command]
pub async fn spotify_play() -> Result<(), String> {
    let token = get_valid_token().await?;
    check_resp(Client::new()
        .put(format!("{SPOTIFY_BASE}/me/player/play"))
        .bearer_auth(&token)
        .header("Content-Length", "0")
        .send()
        .await
        .map_err(|e| e.to_string())?).await
}

#[tauri::command]
pub async fn spotify_pause() -> Result<(), String> {
    let token = get_valid_token().await?;
    check_resp(Client::new()
        .put(format!("{SPOTIFY_BASE}/me/player/pause"))
        .bearer_auth(&token)
        .header("Content-Length", "0")
        .send()
        .await
        .map_err(|e| e.to_string())?).await
}

#[tauri::command]
pub async fn spotify_next() -> Result<(), String> {
    let token = get_valid_token().await?;
    check_resp(Client::new()
        .post(format!("{SPOTIFY_BASE}/me/player/next"))
        .bearer_auth(&token)
        .header("Content-Length", "0")
        .send()
        .await
        .map_err(|e| e.to_string())?).await
}

#[tauri::command]
pub async fn spotify_prev() -> Result<(), String> {
    let token = get_valid_token().await?;
    check_resp(Client::new()
        .post(format!("{SPOTIFY_BASE}/me/player/previous"))
        .bearer_auth(&token)
        .header("Content-Length", "0")
        .send()
        .await
        .map_err(|e| e.to_string())?).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyDevice {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub device_type: String,
}

#[tauri::command]
pub async fn spotify_get_devices() -> Result<Vec<SpotifyDevice>, String> {
    let token = get_valid_token().await?;
    let resp: serde_json::Value = Client::new()
        .get(format!("{SPOTIFY_BASE}/me/player/devices"))
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(resp["devices"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|d| {
            let id = d["id"].as_str()?.to_string();
            Some(SpotifyDevice {
                id,
                name: d["name"].as_str().unwrap_or("Unknown").to_string(),
                is_active: d["is_active"].as_bool().unwrap_or(false),
                device_type: d["type"].as_str().unwrap_or("").to_string(),
            })
        })
        .collect())
}

#[tauri::command]
pub async fn spotify_get_beats(track_id: String) -> Result<Vec<f32>, String> {
    let token = get_valid_token().await?;
    let resp: serde_json::Value = Client::new()
        .get(format!("{SPOTIFY_BASE}/audio-analysis/{track_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if resp.is_null() || resp["error"].is_object() {
        return Err(resp["error"]["message"]
            .as_str()
            .unwrap_or("audio analysis unavailable")
            .to_string());
    }

    Ok(resp["beats"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|b| b["start"].as_f64().map(|s| s as f32))
        .collect())
}

#[tauri::command]
pub async fn spotify_get_audio_features(track_id: String) -> Result<AudioFeatures, String> {
    let token = get_valid_token().await?;
    let resp: serde_json::Value = Client::new()
        .get(format!("{SPOTIFY_BASE}/audio-features/{track_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if resp.is_null() || resp["error"].is_object() {
        return Err(resp["error"]["message"]
            .as_str()
            .unwrap_or("audio features unavailable")
            .to_string());
    }

    Ok(AudioFeatures {
        energy: resp["energy"].as_f64().unwrap_or(0.6) as f32,
        tempo: resp["tempo"].as_f64().unwrap_or(120.0) as f32,
        valence: resp["valence"].as_f64().unwrap_or(0.5) as f32,
    })
}
