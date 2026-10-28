use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub accent_color: String,
    pub icon: String,
    pub last_play_formatted: String,
    pub last_play_relative: String,
    pub last_play_timestamp: Option<i64>,
    pub is_running: bool,
    pub is_detected: bool,
    pub is_custom_path: bool,
    pub install_path: Option<String>,
    pub exe_path: Option<String>,
    pub log_path: Option<String>,
}

fn get_config_file_path() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(app_data).join("hoyo-last-play");
    let _ = fs::create_dir_all(&dir);
    dir.join("custom_paths.json")
}

fn load_custom_paths() -> HashMap<String, String> {
    let path = get_config_file_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
            return map;
        }
    }
    HashMap::new()
}

fn save_custom_path(game_id: &str, install_path: &str) -> Result<(), String> {
    let mut map = load_custom_paths();
    map.insert(game_id.to_string(), install_path.to_string());
    let path = get_config_file_path();
    let json = serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn validate_and_find_exe(game_id: &str, folder: &Path) -> Option<(String, String)> {
    let exe_name = match game_id {
        "gi" => "GenshinImpact.exe",
        "hsr" => "StarRail.exe",
        "zzz" => "ZenlessZoneZero.exe",
        _ => return None,
    };

    // 1. Direct check inside folder
    let direct_exe = folder.join(exe_name);
    if direct_exe.exists() {
        return Some((
            folder.to_string_lossy().to_string(),
            direct_exe.to_string_lossy().to_string(),
        ));
    }

    // 2. Check common subdirectories
    let subdirs = match game_id {
        "gi" => vec!["Genshin Impact Game", "games/Genshin Impact Game", "Genshin Impact"],
        "hsr" => vec!["Star Rail Games", "games/Star Rail Games", "Star Rail"],
        "zzz" => vec!["ZenlessZoneZero Game", "games/ZenlessZoneZero Game", "ZenlessZoneZero"],
        _ => vec![],
    };

    for sub in subdirs {
        let candidate_dir = folder.join(sub);
        let candidate_exe = candidate_dir.join(exe_name);
        if candidate_exe.exists() {
            return Some((
                candidate_dir.to_string_lossy().to_string(),
                candidate_exe.to_string_lossy().to_string(),
            ));
        }
    }

    None
}

#[cfg(windows)]
fn is_process_running(target_exe: &str) -> bool {
    #[repr(C)]
    struct PROCESSENTRY32W {
        dw_size: u32,
        cnt_usage: u32,
        th32_process_id: u32,
        th32_default_heap_id: usize,
        th32_module_id: u32,
        cnt_threads: u32,
        th32_parent_process_id: u32,
        pc_pri_class_base: i32,
        dw_flags: u32,
        sz_exe_file: [u16; 260],
    }

    unsafe extern "system" {
        fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> isize;
        fn Process32FirstW(h_snapshot: isize, lppe: *mut PROCESSENTRY32W) -> i32;
        fn Process32NextW(h_snapshot: isize, lppe: *mut PROCESSENTRY32W) -> i32;
        fn CloseHandle(h_object: isize) -> i32;
    }

    const TH32CS_SNAPPROCESS: u32 = 0x00000002;

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == -1 || snap == 0 {
            return false;
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dw_size = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut found = false;
        if Process32FirstW(snap, &mut entry) != 0 {
            loop {
                let len = entry
                    .sz_exe_file
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.sz_exe_file.len());
                let name = OsString::from_wide(&entry.sz_exe_file[..len])
                    .to_string_lossy()
                    .to_string();

                if name.eq_ignore_ascii_case(target_exe) {
                    found = true;
                    break;
                }

                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
        found
    }
}

#[cfg(not(windows))]
fn is_process_running(_target_exe: &str) -> bool {
    false
}

fn get_game_install_path(reg_sub: &str) -> Option<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = format!(r"Software\Cognosphere\HYP\1_0\{}", reg_sub);
    if let Ok(key) = hkcu.open_subkey(&key_path) {
        if let Ok(val) = key.get_value::<String, _>("GameInstallPath") {
            let trimmed = val.trim().to_string();
            if !trimmed.is_empty() && Path::new(&trimmed).exists() {
                return Some(trimmed);
            }
        }
    }
    None
}

fn get_latest_file_time(path1: &Path, path2: &Path) -> (Option<chrono::DateTime<chrono::Local>>, Option<String>) {
    let mut latest: Option<(SystemTime, String)> = None;

    if let Ok(meta) = std::fs::metadata(path1) {
        if let Ok(m) = meta.modified() {
            latest = Some((m, path1.to_string_lossy().to_string()));
        }
    }

    if let Ok(meta) = std::fs::metadata(path2) {
        if let Ok(m) = meta.modified() {
            if latest.is_none() || m > latest.as_ref().unwrap().0 {
                latest = Some((m, path2.to_string_lossy().to_string()));
            }
        }
    }

    if let Some((time, path)) = latest {
        (Some(chrono::DateTime::<chrono::Local>::from(time)), Some(path))
    } else {
        (None, None)
    }
}

fn format_relative_time(dt: chrono::DateTime<chrono::Local>) -> String {
    let now = chrono::Local::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_seconds() < 0 || duration.num_seconds() < 60 {
        return "Baru saja".to_string();
    }
    if duration.num_minutes() < 60 {
        return format!("{} menit lalu", duration.num_minutes());
    }
    if duration.num_hours() < 24 {
        return format!("{} jam lalu", duration.num_hours());
    }
    if duration.num_days() == 1 {
        return "Kemarin".to_string();
    }
    if duration.num_days() < 30 {
        return format!("{} hari lalu", duration.num_days());
    }
    format!("{} bln lalu", duration.num_days() / 30)
}

fn format_datetime(dt: chrono::DateTime<chrono::Local>) -> String {
    let now = chrono::Local::now();
    let time_str = dt.format("%H:%M").to_string();

    if dt.date_naive() == now.date_naive() {
        format!("Hari ini, {}", time_str)
    } else if dt.date_naive() == now.date_naive().pred_opt().unwrap_or(now.date_naive()) {
        format!("Kemarin, {}", time_str)
    } else {
        dt.format("%d %b %Y, %H:%M").to_string()
    }
}

fn build_game_info(
    id: &str,
    name: &str,
    short_name: &str,
    accent_color: &str,
    reg_key: &str,
    exe_name: &str,
    log_rel1: &str,
    log_rel2: &str,
    fallback_paths: &[&str],
) -> GameInfo {
    let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
    let local_low = PathBuf::from(&user_profile).join("AppData").join("LocalLow");

    let log1 = local_low.join(log_rel1);
    let log2 = local_low.join(log_rel2);

    let (latest_time, used_log) = get_latest_file_time(&log1, &log2);

    // 1. Check custom relocated paths first
    let custom_paths = load_custom_paths();
    let mut install_path: Option<String> = None;
    let mut is_custom_path = false;

    if let Some(cp) = custom_paths.get(id) {
        if Path::new(cp).exists() {
            install_path = Some(cp.clone());
            is_custom_path = true;
        }
    }

    // 2. Check registry
    if install_path.is_none() {
        install_path = get_game_install_path(reg_key);
    }

    // 3. Check fallbacks
    if install_path.is_none() {
        for fb in fallback_paths {
            if Path::new(fb).exists() {
                install_path = Some(fb.to_string());
                break;
            }
        }
    }

    // Full executable path
    let exe_path = install_path.as_ref().and_then(|p| {
        let candidate = PathBuf::from(p).join(exe_name);
        if candidate.exists() {
            Some(candidate.to_string_lossy().to_string())
        } else {
            None
        }
    });

    let is_detected = exe_path.is_some();
    let is_running = is_process_running(exe_name);

    let (last_play_formatted, last_play_relative, last_play_timestamp) = if is_running {
        ("Sedang aktif sekarang".to_string(), "🟢 AKTIF".to_string(), None)
    } else if let Some(dt) = latest_time {
        (
            format_datetime(dt),
            format_relative_time(dt),
            Some(dt.timestamp_millis()),
        )
    } else {
        ("Belum terdeteksi".to_string(), "Tidak ada log".to_string(), None)
    };

    GameInfo {
        id: id.to_string(),
        name: name.to_string(),
        short_name: short_name.to_string(),
        accent_color: accent_color.to_string(),
        icon: format!("icons/{}.png", id),
        last_play_formatted,
        last_play_relative,
        last_play_timestamp,
        is_running,
        is_detected,
        is_custom_path,
        install_path,
        exe_path,
        log_path: used_log,
    }
}

mod commands {
    use super::*;

    #[tauri::command]
    pub fn get_games() -> Vec<GameInfo> {
        vec![
            build_game_info(
                "gi",
                "Genshin Impact",
                "GI",
                "#4EB9D0",
                "hk4e_global",
                "GenshinImpact.exe",
                r"miHoYo\Genshin Impact\output_log.txt",
                r"miHoYo\Genshin Impact\LocalLog.log",
                &[r"C:\Program Files\HoYoPlay\games\Genshin Impact Game"],
            ),
            build_game_info(
                "hsr",
                "Honkai: Star Rail",
                "HSR",
                "#D3A158",
                "hkrpg_global",
                "StarRail.exe",
                r"Cognosphere\Star Rail\Player.log",
                r"Cognosphere\Star Rail\output_log.txt",
                &[r"D:\HoYoPlay\Star Rail Games"],
            ),
            build_game_info(
                "zzz",
                "Zenless Zone Zero",
                "ZZZ",
                "#E7EF2D",
                "nap_global",
                "ZenlessZoneZero.exe",
                r"miHoYo\ZenlessZoneZero\Player.log",
                r"miHoYo\ZenlessZoneZero\output_log.txt",
                &[r"D:\HoYoPlay\ZenlessZoneZero Game"],
            ),
        ]
    }

    #[tauri::command]
    pub fn launch_game(game_id: String) -> Result<String, String> {
        let games = get_games();
        let game = games.into_iter().find(|g| g.id == game_id)
            .ok_or_else(|| format!("Game dengan ID '{}' tidak ditemukan.", game_id))?;

        let exe_path = game.exe_path
            .ok_or_else(|| format!("File executable untuk {} belum terpasang atau tidak ditemukan. Silakan gunakan fitur 'Relokasi Folder'.", game.name))?;

        let path_obj = Path::new(&exe_path);
        if !path_obj.exists() {
            return Err(format!("File '{}' tidak ditemukan di disk.", exe_path));
        }

        let dir_obj = path_obj.parent().unwrap_or_else(|| Path::new("."));

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x00000008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

            let mut cmd = std::process::Command::new(&exe_path);
            cmd.current_dir(dir_obj);
            cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);

            match cmd.spawn() {
                Ok(_) => Ok(format!("Berhasil membuka {}!", game.name)),
                Err(e) => Err(format!("Gagal meluncurkan {}: {}", game.name, e)),
            }
        }

        #[cfg(not(windows))]
        {
            let mut cmd = std::process::Command::new(&exe_path);
            cmd.current_dir(dir_obj);
            match cmd.spawn() {
                Ok(_) => Ok(format!("Berhasil membuka {}!", game.name)),
                Err(e) => Err(format!("Gagal meluncurkan {}: {}", game.name, e)),
            }
        }
    }

    #[tauri::command]
    pub fn relocate_game(game_id: String) -> Result<Option<String>, String> {
        let exe_name = match game_id.as_str() {
            "gi" => "GenshinImpact.exe",
            "hsr" => "StarRail.exe",
            "zzz" => "ZenlessZoneZero.exe",
            _ => return Err("Game ID tidak valid.".to_string()),
        };

        let title = format!("Pilih Folder Instalasi Game (Folder yang memuat {})", exe_name);

        let picked = rfd::FileDialog::new()
            .set_title(&title)
            .pick_folder();

        if let Some(folder_path) = picked {
            if let Some((valid_install_dir, _valid_exe_path)) = validate_and_find_exe(&game_id, &folder_path) {
                save_custom_path(&game_id, &valid_install_dir)?;
                Ok(Some(valid_install_dir))
            } else {
                Err(format!(
                    "File '{}' tidak ditemukan di dalam '{}'. Pastikan kamu memilih folder instalasi game yang sesuai.",
                    exe_name,
                    folder_path.display()
                ))
            }
        } else {
            Ok(None)
        }
    }

    #[tauri::command]
    pub fn reset_game_path(game_id: String) -> Result<(), String> {
        let mut map = load_custom_paths();
        map.remove(&game_id);
        let path = get_config_file_path();
        let json = serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?;
        let _ = fs::write(&path, json);
        Ok(())
    }

    #[tauri::command]
    pub fn open_folder(path: String) -> Result<(), String> {
        let p = Path::new(&path);
        if p.exists() {
            let _ = std::process::Command::new("explorer").arg(path).spawn();
            Ok(())
        } else {
            Err("Folder tidak ditemukan.".to_string())
        }
    }

    #[tauri::command]
    pub fn is_autostart_enabled() -> bool {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") {
            if let Ok(val) = key.get_value::<String, _>("HoyoLastPlayTauri") {
                return !val.is_empty();
            }
        }
        false
    }

    #[tauri::command]
    pub fn set_autostart(enable: bool) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu.open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            KEY_WRITE | KEY_READ,
        ).map_err(|e| e.to_string())?;

        if enable {
            if let Ok(exe_path) = std::env::current_exe() {
                key.set_value("HoyoLastPlayTauri", &exe_path.to_string_lossy().to_string())
                    .map_err(|e| e.to_string())?;
            }
        } else {
            let _ = key.delete_value("HoyoLastPlayTauri");
        }
        Ok(())
    }

    #[tauri::command]
    pub fn drag_window(window: tauri::Window) {
        let _ = window.start_dragging();
    }

    #[tauri::command]
    pub fn minimize_window(window: tauri::Window) {
        let _ = window.minimize();
    }

    #[tauri::command]
    pub fn close_window(window: tauri::Window) {
        let _ = window.close();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_games,
            commands::launch_game,
            commands::relocate_game,
            commands::reset_game_path,
            commands::open_folder,
            commands::is_autostart_enabled,
            commands::set_autostart,
            commands::drag_window,
            commands::minimize_window,
            commands::close_window
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
