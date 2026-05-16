use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

// portable-pty's MasterPty/SlavePty traits aren't Send, but the Win32 ConPTY
// impl wraps Windows HANDLEs which are safe to use across threads.
struct MasterHandle(Box<dyn portable_pty::MasterPty>);
unsafe impl Send for MasterHandle {}

#[allow(dead_code)] // field kept alive for Drop; dropping kills the child process
struct SlaveHandle(Box<dyn portable_pty::SlavePty>);
unsafe impl Send for SlaveHandle {}

pub(crate) struct PtyInner {
    master: MasterHandle,
    writer: Box<dyn Write + Send>,
    _slave: SlaveHandle,
}

pub struct PtyManager(pub Mutex<Option<PtyInner>>);

impl PtyManager {
    pub fn new() -> Self {
        PtyManager(Mutex::new(None))
    }
}

#[tauri::command]
pub fn pty_create(
    app: AppHandle,
    state: State<PtyManager>,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    // Drop any existing session before starting a new one
    *state.0.lock().unwrap() = None;

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| e.to_string())?;

    let mut cmd = CommandBuilder::new("C:/Program Files/Git/bin/bash.exe");
    cmd.args(["--login", "-i"]);
    let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    cmd.cwd(project_root.canonicalize().unwrap_or(project_root));
    pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    // Stream PTY output → frontend via Tauri events
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let s = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let _ = app.emit("pty-data", s);
                }
            }
        }
    });

    *state.0.lock().unwrap() = Some(PtyInner {
        master: MasterHandle(pair.master),
        writer,
        _slave: SlaveHandle(pair.slave),
    });
    Ok(())
}

#[tauri::command]
pub fn pty_write(state: State<PtyManager>, data: String) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    if let Some(pty) = guard.as_mut() {
        pty.writer.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn pty_resize(state: State<PtyManager>, rows: u16, cols: u16) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    if let Some(pty) = guard.as_ref() {
        pty.master
            .0
            .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn pty_kill(state: State<PtyManager>) -> Result<(), String> {
    *state.0.lock().unwrap() = None;
    Ok(())
}
